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
    fn test_branches_format_columns() {
        let mut input = String::new();
        for _ in (0..49) {
            input = input + "branch\n"
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
branch";

        assert_eq!(expected, branches.format_columns());

        let mut input = String::new();
        for _ in (0..51) {
            input = input + "branch\n"
        }

        let branches = Branches::new(&input);

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

        assert_eq!(expected, branches.format_columns());

        let mut input = String::new();
        for _ in (0..101) {
            input = input + "branch\n"
        }

        let branches = Branches::new(&input);

        let expected =
"\
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch                                   branch
branch                                   branch\
";

        assert_eq!(expected, branches.format_columns());
    }
}
