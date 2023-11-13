use commands::*;
use error::Error;
use options::*;
use regex::Regex;
use std::io::{stdin, stdout, Write};

pub const COLUMN_SPACER_LENGTH: usize = 30;

#[derive(Debug)]
pub struct Branches {
    pub string: String,
    pub vec: Vec<String>,
}

impl Branches {
    pub fn new(branches: Vec<String>) -> Branches {
        let trimmed_string = branches.join("\n").trim_end_matches('\n').into();

        Branches {
            string: trimmed_string,
            vec: branches,
        }
    }

    pub fn print_warning_and_prompt(&self, delete_mode: &DeleteMode) -> Result<(), Error> {
        println!("{}", delete_mode.warning_message());
        println!("{}", self.format_columns());
        print!("Continue? (Y/n) ");
        stdout().flush()?;

        // Read the user's response on continuing
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        match input.to_lowercase().as_ref() {
            "y\n" | "y\r\n" | "yes\n" | "yes\r\n" | "\n" | "\r\n" => Ok(()),
            _ => Err(Error::ExitEarly),
        }
    }

    pub fn merged(options: &Options) -> Branches {
        let mut branches: Vec<String> = vec![];
        println!("Updating remote {}", options.remote);
        run_command_with_no_output(&["git", "remote", "update", &options.remote, "--prune"]);

        let merged_branches_regex = format!("^\\*?\\s*{}$", options.base_branch);
        let merged_branches_filter = Regex::new(&merged_branches_regex).unwrap();
        let merged_branches_cmd = run_command(&["git", "branch", "--merged"]);
        let merged_branches_output = std::str::from_utf8(&merged_branches_cmd.stdout).unwrap();

        let merged_branches =
            merged_branches_output
                .lines()
                .fold(Vec::<String>::new(), |mut acc, line| {
                    if !merged_branches_filter.is_match(line) {
                        acc.push(line.trim().to_string());
                    }
                    acc
                });

        let local_branches_regex = format!("^\\*?\\s*{}$", options.base_branch);
        let local_branches_filter = Regex::new(&local_branches_regex).unwrap();
        let local_branches_cmd = run_command(&["git", "branch"]);
        let local_branches_output = std::str::from_utf8(&local_branches_cmd.stdout).unwrap();

        let local_branches = local_branches_output
            .lines()
            .fold(Vec::<String>::new(), |mut acc, line| {
                if !local_branches_filter.is_match(line) {
                    acc.push(line.trim().to_string());
                }
                acc
            })
            .iter()
            .filter(|branch| !options.ignored_branches.contains(branch))
            .cloned()
            .collect::<Vec<String>>();

        let remote_branches_regex = format!("\\b(HEAD|{})\\b", &options.base_branch);
        let remote_branches_filter = Regex::new(&remote_branches_regex).unwrap();
        let remote_branches_cmd = run_command(&["git", "branch", "-r"]);
        let remote_branches_output = std::str::from_utf8(&remote_branches_cmd.stdout).unwrap();

        let remote_branches =
            remote_branches_output
                .lines()
                .fold(Vec::<String>::new(), |mut acc, line| {
                    if !remote_branches_filter.is_match(line) {
                        acc.push(line.trim().to_string());
                    }
                    acc
                });

        for branch in local_branches {
            // First check if the local branch doesn't exist in the remote, it's the cheapest and easiest
            // way to determine if we want to suggest to delete it.
            if options.delete_unpushed_branches
                && !remote_branches
                    .iter()
                    .any(|b: &String| *b == format!("{}/{}", &options.remote, branch))
            {
                branches.push(branch.to_owned());
                continue;
            }

            // If it does exist in the remote, check to see if it's listed in git branches --merged. If
            // it is, that means it wasn't merged using Github squashes, and we can suggest it.
            if merged_branches.iter().any(|b: &String| *b == branch) {
                branches.push(branch.to_owned());
                continue;
            }

            // If neither of the above matched, merge main into the branch and see if it succeeds.
            // If it can't cleanly merge, then it has likely been merged with Github squashes, and we
            // can suggest it.
            if options.squashes {
                run_command(&["git", "checkout", &branch]);
                match run_command_with_status(&[
                    "git",
                    "pull",
                    "--ff-only",
                    &options.remote,
                    &options.base_branch,
                ]) {
                    Ok(status) => {
                        if !status.success() {
                            println!("why");
                            branches.push(branch);
                        }
                    }
                    Err(err) => {
                        println!(
                            "Encountered error trying to update branch {} with branch {}: {}",
                            branch, options.base_branch, err
                        );
                        continue;
                    }
                }

                run_command(&["git", "reset", "--hard"]);
                run_command(&["git", "checkout", &options.base_branch]);
            }
        }

        // if deleted in remote, list
        //
        // g branch -d -r <remote>/<branch>
        // g branch -d <branch>

        Branches::new(branches)
    }

    fn format_columns(&self) -> String {
        // Covers the single column case
        if self.vec.len() < 26 {
            return self.string.clone();
        }

        let col_count = {
            let total_cols = self.vec.len() / 25 + 1;
            ::std::cmp::min(total_cols, 3)
        };

        let chunks = self.vec.chunks(col_count);
        let mut col_indices = [0; 3];

        for i in 1..col_count {
            let index = i - 1;
            let largest_col_member = chunks
                .clone()
                .map(|chunk| {
                    if let Some(branch) = chunk.get(index) {
                        branch.len()
                    } else {
                        0
                    }
                })
                .max()
                .unwrap();
            let next_col_start = largest_col_member + COLUMN_SPACER_LENGTH;
            col_indices[i - 1] = next_col_start;
        }

        let rows: Vec<String> = self
            .vec
            .chunks(col_count)
            .map(|chunk| make_row(chunk, &col_indices))
            .collect();

        rows.join("\n").trim().to_owned()
    }

    pub fn delete(&self, options: &Options) -> String {
        match options.delete_mode {
            DeleteMode::Local => delete_local_branches(self),
            DeleteMode::Remote => delete_remote_branches(self, options),
            DeleteMode::Both => {
                let local_output = delete_local_branches(self);
                let remote_output = delete_remote_branches(self, options);
                [
                    "Remote:".to_owned(),
                    remote_output,
                    "\nLocal:".to_owned(),
                    local_output,
                ]
                .join("\n")
            }
        }
    }
}

fn make_row(chunks: &[String], col_indices: &[usize]) -> String {
    match chunks.len() {
        1 => chunks[0].clone(),
        2 => {
            format!(
                "{b1:0$}{b2}",
                col_indices[0],
                b1 = chunks[0],
                b2 = chunks[1]
            )
        }
        3 => {
            format!(
                "{b1:0$}{b2:1$}{b3}",
                col_indices[0],
                col_indices[1],
                b1 = chunks[0],
                b2 = chunks[1],
                b3 = chunks[2]
            )
        }
        _ => unreachable!("This code should never be reached!"),
    }
}

#[cfg(test)]
mod test {
    use super::Branches;

    #[test]
    fn test_branches_new() {
        let input = vec!["branch1".to_owned(), "branch2".to_owned()];
        let branches = Branches::new(input);

        assert_eq!("branch1\nbranch2".to_owned(), branches.string);
        assert_eq!(
            vec!["branch1".to_owned(), "branch2".to_owned()],
            branches.vec
        );
    }

    #[test]
    fn test_format_single_column() {
        let mut input = vec![];
        for _ in 0..24 {
            input.push("branch".to_owned())
        }

        let branches = Branches::new(input);

        let expected = "\
branch
branch
branch
branch
branch
branch
branch
branch
branch
branch
\
                        branch
branch
branch
branch
branch
branch
branch
branch
branch
branch
\
                        branch
branch
branch
branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_format_two_columns() {
        let mut input = vec![];
        for _ in 0..26 {
            input.push("branch".to_owned())
        }

        let branches = Branches::new(input);

        let expected = "\
branch                              branch
branch                              \
branch
branch                              branch
branch                              \
branch
branch                              branch
branch                              \
branch
branch                              branch
branch                              \
branch
branch                              branch
branch                              \
branch
branch                              branch
branch                              \
branch
branch                              branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_format_three_columns() {
        let mut input = vec![];
        for _ in 0..51 {
            input.push("branch".to_owned())
        }

        let branches = Branches::new(input);

        let expected = "\
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_format_maxes_at_three_columns() {
        let mut input = vec![];
        for _ in 0..76 {
            input.push("branch".to_owned())
        }

        let branches = Branches::new(input);

        let expected = "\
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch                              branch                              \
branch
branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_branches_of_different_lengths() {
        let mut input = vec![];
        for (i, _) in (0..26).enumerate() {
            input.push(format!("branch{}", i))
        }

        let branches = Branches::new(input);

        let expected = "\
branch0                               branch1
branch2                               \
branch3
branch4                               branch5
branch6                               \
branch7
branch8                               branch9
branch10                              \
branch11
branch12                              branch13
branch14                              \
branch15
branch16                              branch17
branch18                              \
branch19
branch20                              branch21
branch22                              \
branch23
branch24                              branch25";
        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_branches_of_bigger_lengths() {
        let mut input = vec!["really_long_branch_name".to_owned(), "branch-1".to_owned()];
        for (i, _) in (0..26).enumerate() {
            input.push(format!("branch{}", i));
        }

        let branches = Branches::new(input);

        let expected = "\
really_long_branch_name                              branch-1
branch0                                              \
branch1
branch2                                              branch3
branch4                                              \
branch5
branch6                                              branch7
branch8                                              \
branch9
branch10                                             branch11
branch12                                             \
branch13
branch14                                             branch15
branch16                                             \
branch17
branch18                                             branch19
branch20                                             \
branch21
branch22                                             branch23
branch24                                             \
branch25";
        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_long_branches_with_three_columns() {
        let mut input = vec![
            "really_long_branch_name".to_owned(),
            "branch".to_owned(),
            "branch".to_owned(),
            "branch".to_owned(),
            "really_long_middle_col".to_owned(),
            "branch".to_owned(),
        ];
        for i in 0..45 {
            input.push(format!("branch{}", i));
        }

        let branches = Branches::new(input);

        let expected = "\
really_long_branch_name                              branch                                              branch
branch                                               really_long_middle_col                              branch
branch0                                              branch1                                             branch2
branch3                                              branch4                                             branch5
branch6                                              branch7                                             branch8
branch9                                              branch10                                            branch11
branch12                                             branch13                                            branch14
branch15                                             branch16                                            branch17
branch18                                             branch19                                            branch20
branch21                                             branch22                                            branch23
branch24                                             branch25                                            branch26
branch27                                             branch28                                            branch29
branch30                                             branch31                                            branch32
branch33                                             branch34                                            branch35
branch36                                             branch37                                            branch38
branch39                                             branch40                                            branch41
branch42                                             branch43                                            branch44";
        assert_eq!(expected, branches.format_columns());
    }
}
