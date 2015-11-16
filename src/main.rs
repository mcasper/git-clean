extern crate getopts;

use std::process::{Command, Stdio, Child};
use std::io;
use std::io::{Read, Write};
use std::env;

use getopts::{Options, Matches};

enum DeleteOption {
    Local,
    Remote,
    Both,
}

impl DeleteOption {
    fn new(opts: Matches) -> DeleteOption {
        return if opts.opt_present("l") {
            DeleteOption::Local
        } else if opts.opt_present("r") {
            DeleteOption::Remote
        } else {
            DeleteOption::Both
        };
    }

    fn warning_message(&self) -> String {
        let source = match self {
            &DeleteOption::Local => "locally:",
            &DeleteOption::Remote => "remotely:",
            &DeleteOption::Both => "locally and remotely:",
        };
        "The following branches will be deleted ".to_owned() + source
    }
}

struct Branches {
    string: String,
    vec: Vec<String>
}

impl Branches {
    fn new(branches: &String) -> Branches {
        let split = branches.split("\n");
        let vec: Vec<&str> = split.collect();
        let trimmed_vec: Vec<String> = vec.iter().map(|s| s.trim().to_owned()).collect();
        let trimmed_string = trimmed_vec.join("\n").trim_right_matches("\n").to_owned();

        Branches {
            string: trimmed_string,
            vec: trimmed_vec,
        }
    }

    fn format_columns(&self) -> String {
        if self.vec.len() < 51 {
            return self.string.clone();
        }

        let col_count = self.vec.len() / 50 + 1;
        let spacer = "                                   ";

        let rows = self.vec.chunks(col_count)
            .map(|chunk| chunk.join(spacer)).collect::<Vec<String>>();

        rows.join("\n").trim().to_owned()
    }
}

struct GitOptions {
    remote: String,
    base_branch: String
}

impl GitOptions {
    fn new(opts: &Matches) -> GitOptions {
        let remote = match opts.opt_str("R") {
            Some(remote) => remote,
            None => "origin".to_owned(),
        };
        let base_branch = match opts.opt_str("b") {
            Some(branch) => branch,
            None => "master".to_owned(),
        };

        GitOptions {
            remote: remote,
            base_branch: base_branch,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("l", "locals", "only delete local branches");
    opts.optflag("r", "remotes", "only delete remote branches");
    opts.optopt("R", "", "changes the git remote used (default is origin)", "REMOTE");
    opts.optopt("b", "", "changes the base for merged branches (default is master)", "BRANCH");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => {
            println!("{}", e);
            print_usage(opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    let git_options = GitOptions::new(&matches);

    let branches = merged_branches(&git_options);

    // Early return if there's nothing to delete
    if branches.string.len() == 0 {
        println!("No branches to delete, you're clean!");
        return;
    }

    let del_opt = DeleteOption::new(matches);

    print_warning(&branches, &del_opt);

    // Read the user's response on continuing
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.to_lowercase().as_ref() {
        "y\n" => (),
        "yes\n" => (),
        _ => return,
    }

    ensure_base_branch(&git_options);

    match delete_branches(&branches, del_opt, &git_options) {
        Ok(ref msg) => println!("\n{}", msg),
        Err(ref msg) => println!("\n{}", msg),
    }
}

fn print_usage(opts: Options) {
    print!("{}", opts.usage("Usage: git-clean [options]"));
}

fn print_warning(branches: &Branches, del_opt: &DeleteOption) {
    println!("{}", del_opt.warning_message());
    println!("{}", branches.format_columns());
    print!("Continue? (yN) ");
    io::stdout().flush().unwrap();
}

fn merged_branches(git_options: &GitOptions) -> Branches {
    let base_branch = &git_options.base_branch;
    let regex = "(\\* ".to_owned() + base_branch + "|\\s" + base_branch + ")";
    let grep = spawn_piped(vec!["grep", "-vE", &regex]);

    let gbranch = Command::new("git")
        .args(&["branch", "--merged", base_branch])
        .output()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    {
        grep.stdin.unwrap().write_all(&gbranch.stdout).unwrap();
    }

    let mut s = String::new();
    grep.stdout.unwrap().read_to_string(&mut s).unwrap();

    Branches::new(&s)
}

fn delete_branches(branches: &Branches, options: DeleteOption, git_options: &GitOptions) -> Result<String, String> {
    let output = match options {
        DeleteOption::Local => delete_local_branches(&branches.string).unwrap(),
        DeleteOption::Remote => delete_remote_branches(&branches.string, git_options).unwrap(),
        DeleteOption::Both => {
            let out1 = delete_remote_branches(&branches.string, git_options).unwrap();
            let out2 = delete_local_branches(&branches.string).unwrap();
            ["Remote:".to_owned(), out1, "Local:".to_owned(), out2].join("\n")
        },
    };

    Ok(output)
}

fn delete_local_branches(branches: &String) -> Result<String, String> {
    let xargs = spawn_piped(vec!["xargs", "git", "branch", "-d"]);

    {
        xargs.stdin.unwrap().write_all(branches.as_bytes()).unwrap()
    }

    let mut s = String::new();
    xargs.stdout.unwrap().read_to_string(&mut s).unwrap();
    Ok(s)
}

fn delete_remote_branches(branches: &String, git_options: &GitOptions) -> Result<String, String> {
    let xargs = spawn_piped(vec!["xargs", "git", "push", &git_options.remote, "--delete"]);

    {
        xargs.stdin.unwrap().write_all(branches.as_bytes()).unwrap()
    }

    let mut stdout = String::new();
    xargs.stdout.unwrap().read_to_string(&mut stdout).unwrap();

    let mut stderr = String::new();
    xargs.stderr.unwrap().read_to_string(&mut stderr).unwrap();

    let split = stderr.split("\n");
    let vec: Vec<&str> = split.collect();
    let mut failed_remotes = vec![];
    for s in vec {
        if s.contains("error: unable to delete '") {
            let branch = s.trim_left_matches("error: unable to delete '")
                .trim_right_matches("': remote ref does not exist");

            failed_remotes.push(branch.to_owned() + " was already deleted in the remote.");
        }
    };
    let output = failed_remotes.join("\n") + &stdout;

    Ok(output)
}

fn ensure_base_branch(git_options: &GitOptions) {
    let current_branch_command = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    let current_branch = String::from_utf8(current_branch_command.stdout).unwrap();

    if current_branch.trim() != git_options.base_branch {
        panic!("Please run this command from the branch: ".to_owned() + &git_options.base_branch);
    }
}

fn spawn_piped(args: Vec<&str>) -> Child {
    let cmd = args[0];
    Command::new(cmd)
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) })
}

#[cfg(test)]
mod test {
    use super::trim_entries;
    use super::format_columns;
    use super::print_warning;
    use super::DeleteOption;
    use super::spawn_piped;

    use std::io::{Read, Write};

    use getopts::{Options};

    #[test]
    fn test_trim_entries() {
        assert_eq!("branch", trim_entries(" branch ".to_owned()));
        assert_eq!("branch", trim_entries(" branch\n".to_owned()));
        assert_eq!("branch1\nbranch2\nbranch3", trim_entries(" branch1 \n branch2 \n branch3 ".to_owned()));
    }

    #[test]
    fn test_format_columns() {
        let mut input = String::new();
        for _ in (0..49) {
            input = input + "branch\n"
        }

        let expected =
"\
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
";

        assert_eq!(expected, format_columns(&input));

        let mut input = String::new();
        for _ in (0..51) {
            input = input + "branch\n"
        }

        let expected =
"\
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch                                   branch
branch\
";

        assert_eq!(expected, format_columns(&input));
    }

    #[test]
    fn test_print_warning() {
        print_warning(&"branch".to_owned(), &DeleteOption::Both);
    }

    #[test]
    fn test_delete_option_new() {
        let mut opts = Options::new();
        opts.optflag("l", "local", "only delete local branches");
        opts.optflag("r", "remote", "only delete remote branches");
        opts.optflag("h", "help", "print this help menu");

        // opts throws away the first elem, because it expects it to be the
        // path of the executable
        let args = vec!["./target/debug/git-clean", "-l"];

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return }
        };

        match DeleteOption::new(matches) {
            DeleteOption::Local => (),
            other @ _ => panic!("Expected a DeleteOption::Local, but found: {:?}", other),
        };

        let args = vec!["./target/debug/git-clean", "-r"];

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return }
        };

        match DeleteOption::new(matches) {
            DeleteOption::Remote => (),
            other @ _ => panic!("Expected a DeleteOption::Remote, but found: {:?}", other),
        };

        let args = vec!["./target/debug/git-clean"];

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return }
        };

        match DeleteOption::new(matches) {
            DeleteOption::Both => (),
            other @ _ => panic!("Expected a DeleteOption::Both, but found: {:?}", other),
        };
    }

    #[test]
    fn test_spawn_piped() {
        let echo = spawn_piped(vec!["grep", "foo"]);

        {
            echo.stdin.unwrap().write_all("foo\nbar\nbaz".as_bytes()).unwrap()
        }

        let mut stdout = String::new();
        echo.stdout.unwrap().read_to_string(&mut stdout).unwrap();

        assert_eq!(stdout, "foo\n");
    }

    #[test]
    fn test_warning_message() {
        assert_eq!("The following branches will be deleted locally:", DeleteOption::Local.warning_message());
        assert_eq!("The following branches will be deleted remotely:", DeleteOption::Remote.warning_message());
        assert_eq!("The following branches will be deleted locally and remotely:", DeleteOption::Both.warning_message());
    }
}
