pub struct Branches {
    pub string: String,
    pub vec: Vec<String>
}

impl Branches {
    pub fn new(branches: &String) -> Branches {
        let split = branches.split("\n");
        let vec: Vec<&str> = split.collect();
        let trimmed_vec: Vec<String> = vec.iter().map(|s| s.trim().to_owned()).collect();
        let trimmed_string = trimmed_vec.join("\n").trim_right_matches("\n").to_owned();

        Branches {
            string: trimmed_string,
            vec: trimmed_vec,
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
        let mut col_indices = vec![];

        // TODO: Make this code generic with some crazy loops
        match col_count {
            2 => {
                let mut largest_col1 = 0;
                for chunk in chunks {
                    if chunk[0].len() > largest_col1 {
                        largest_col1 = chunk[0].len()
                    }
                }
                let col2_start = largest_col1 + 30;
                col_indices.push(col2_start);
            },
            3 => {
                let mut largest_col1 = 0;
                let mut largest_col2 = 0;
                for chunk in chunks {
                    if chunk[0].len() > largest_col1 {
                        largest_col1 = chunk[0].len()
                    };
                    if let Some(branch) = chunk.get(1) {
                        if branch.len() > largest_col2 {
                            largest_col2 = chunk[1].len()
                        }
                    } else {
                    };
                }
                let col2_start = largest_col1 + 30;
                let col3_start = largest_col2 + 30;
                col_indices.push(col2_start);
                col_indices.push(col3_start);
            },
            _ => unreachable!(),
        }

        let rows: Vec<String> = self.vec.chunks(col_count)
            .map(|chunk| make_row(chunk, &col_indices)).collect();

        rows.join("\n").trim().to_owned()
    }
}

fn make_row(chunks: &[String], col_indices: &Vec<usize>) -> String {
    let mut result = chunks[0].clone();
    for (i, chunk) in chunks[1..].iter().enumerate() {
        let spacer_len = col_indices[i] - chunks[i].len();
        let mut spacer = String::new();
        let _: Vec<_> = (0..spacer_len).map(|_| spacer.push_str(" ")).collect();
        result.push_str(&spacer);
        result.push_str(chunk);
    }
    result
}

#[cfg(test)]
mod test {
    use super::Branches;

    #[test]
    fn test_branches_new() {
        let input = " branch1\n branch2 ";
        let branches = Branches::new(&input.to_string());

        assert_eq!("branch1\nbranch2".to_owned(), branches.string);
        assert_eq!(vec!["branch1".to_owned(), "branch2".to_owned()], branches.vec);
    }

    #[test]
    fn test_format_single_column() {
        let mut input = String::new();
        for _ in 0..24 {
            input.push_str("branch\n")
        }

        let branches = Branches::new(&input);

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
branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_format_two_columns() {
        let mut input = String::new();
        for _ in 0..26 {
            input.push_str("branch\n")
        }

        let branches = Branches::new(&input);

        let expected =
"\
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch
branch                              branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_format_three_columns() {
        let mut input = String::new();
        for _ in 0..51 {
            input.push_str("branch\n")
        }

        let branches = Branches::new(&input);

        let expected =
"\
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_format_maxes_at_three_columns() {
        let mut input = String::new();
        for _ in 0..76 {
            input.push_str("branch\n")
        }

        let branches = Branches::new(&input);

        let expected =
"\
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch                              branch                              branch
branch\
";

        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_branches_of_different_lengths() {
        let mut input = String::new();
        for (i, _) in (0..26).enumerate() {
            input.push_str("branch");
            input.push_str(&i.to_string());
            input.push_str("\n");
        }

        let branches = Branches::new(&input);

        let expected =
"\
branch0                               branch1
branch2                               branch3
branch4                               branch5
branch6                               branch7
branch8                               branch9
branch10                              branch11
branch12                              branch13
branch14                              branch15
branch16                              branch17
branch18                              branch19
branch20                              branch21
branch22                              branch23
branch24                              branch25";
        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_branches_of_bigger_lengths() {
        let mut input = "really_long_branch_name\nbranch-1\n".to_owned();
        for (i, _) in (0..26).enumerate() {
            input.push_str("branch");
            input.push_str(&i.to_string());
            input.push_str("\n");
        }

        let branches = Branches::new(&input);

        let expected =
"\
really_long_branch_name                              branch-1
branch0                                              branch1
branch2                                              branch3
branch4                                              branch5
branch6                                              branch7
branch8                                              branch9
branch10                                             branch11
branch12                                             branch13
branch14                                             branch15
branch16                                             branch17
branch18                                             branch19
branch20                                             branch21
branch22                                             branch23
branch24                                             branch25";
        assert_eq!(expected, branches.format_columns());
    }

    #[test]
    fn test_long_branches_with_three_columns() {
        let mut input = "really_long_branch_name\nbranch\nbranch\nbranch\nreally_long_middle_col\nbranch\n".to_owned();
        for i in 0..45 {
            input.push_str("branch");
            input.push_str(&i.to_string());
            input.push_str("\n");
        }

        let branches = Branches::new(&input);

        let expected =
"\
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
