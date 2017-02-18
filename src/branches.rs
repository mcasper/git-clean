pub const COLUMN_SPACER_LENGTH: usize = 30;

#[derive(Debug)]
pub struct Branches {
    pub string: String,
    pub vec: Vec<String>,
}

impl Branches {
    pub fn new(branches: Vec<String>) -> Branches {
        let trimmed_string = branches.join("\n").trim_right_matches('\n').into();

        Branches {
            string: trimmed_string,
            vec: branches,
        }
    }

    pub fn format_columns(&self) -> String {
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
            let largest_col_member = chunks.clone()
                .map(|chunk| if let Some(branch) = chunk.get(index) {
                    branch.len()
                } else {
                    0
                })
                .max()
                .unwrap();
            let next_col_start = largest_col_member + COLUMN_SPACER_LENGTH;
            col_indices[i - 1] = next_col_start;
        }

        let rows: Vec<String> = self.vec
            .chunks(col_count)
            .map(|chunk| make_row(chunk, &col_indices))
            .collect();

        rows.join("\n").trim().to_owned()
    }
}

fn make_row(chunks: &[String], col_indices: &[usize]) -> String {
    match chunks.len() {
        1 => chunks[0].clone(),
        2 => {
            format!("{b1:0$}{b2}",
                    col_indices[0],
                    b1 = chunks[0],
                    b2 = chunks[1])
        }
        3 => {
            format!("{b1:0$}{b2:1$}{b3}",
                    col_indices[0],
                    col_indices[1],
                    b1 = chunks[0],
                    b2 = chunks[1],
                    b3 = chunks[2])
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
        assert_eq!(vec!["branch1".to_owned(), "branch2".to_owned()],
                   branches.vec);
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

        let expected =
            "\
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
        let mut input = vec!["really_long_branch_name".to_owned(),
                             "branch".to_owned(),
                             "branch".to_owned(),
                             "branch".to_owned(),
                             "really_long_middle_col".to_owned(),
                             "branch".to_owned()];
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
