use std::path::{PathBuf, Path};
use std::process::{Command, Output};
use std::{env, str};
use tempdir::TempDir;

pub fn project(name: &str) -> ProjectBuilder {
    ProjectBuilder::new(name)
}

pub struct ProjectBuilder {
    pub name: String,
}

impl ProjectBuilder {
    fn new(name: &str) -> Self {
        ProjectBuilder {
            name: name.into(),
        }
    }

    pub fn build(self) -> Project {
        let work_dir = TempDir::new(&self.name).unwrap();
        let remote_dir = TempDir::new(&format!("{}_remote", &self.name)).unwrap();

        let project = Project {
            directory: work_dir,
            name: self.name,
            remote: remote_dir,
        };

        project.batch_setup_commands(
            &[
                "git init",
                "git config push.default matching",
                "git remote add origin remote",
                "touch test_file.txt",
                "git add .",
                "git commit -am Init",
            ]
        );

        project
    }
}

pub struct Project {
    directory: TempDir,
    pub name: String,
    remote: TempDir,
}

impl Project {
    pub fn setup_command(&self, command: &str) -> TestCommandResult {
        let command_pieces = command.split(' ').collect::<Vec<&str>>();
        let result = TestCommand::new(
            &self.path(),
            command_pieces[1..].to_vec(),
            command_pieces[0]
            ).run();

        if !result.is_success() {
            panic!(result.failure_message("setup command to succeed"))
        }

        result
    }

    pub fn remote_setup_command(&self, command: &str) -> TestCommandResult {
        let command_pieces = command.split(' ').collect::<Vec<&str>>();
        let result = TestCommand::new(
            &self.remote_path(),
            command_pieces[1..].to_vec(),
            command_pieces[0]
            ).run();

        if !result.is_success() {
            panic!(result.failure_message("remote setup command to succeed"))
        }

        result
    }

    pub fn batch_setup_commands(&self, commands: &[&str]) {
        for command in commands.iter() {
            self.setup_command(command);
        };
    }

    pub fn git_clean_command(&self, command: &str) -> TestCommand {
        let command_pieces = command.split(' ').collect::<Vec<&str>>();
        TestCommand::new(&self.path(), command_pieces, path_to_git_clean())
    }

    fn path(&self) -> PathBuf {
        self.directory.path().into()
    }

    fn remote_path(&self) -> PathBuf {
        self.remote.path().into()
    }

    pub fn setup_remote(self) -> Project {
        self.remote_setup_command("git init");
        self.remote_setup_command("git checkout -b other");

        self.setup_command(&format!("git remote set-url origin {}", self.remote_path().display()));
        self.setup_command("git push origin HEAD");

        self
    }
}

pub struct TestCommand {
    pub path: PathBuf,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    top_level_command: String,
}

impl TestCommand {
    fn new<S: Into<String>>(path: &Path, args: Vec<&str>, top_level_command: S) -> Self {
        let owned_args = args.iter().map(|arg| arg.to_owned().to_owned()).collect::<Vec<String>>();

        TestCommand {
            path: path.into(),
            args: owned_args,
            envs: vec![],
            top_level_command: top_level_command.into(),
        }
    }

    pub fn env(mut self, key: &str, value: &str) -> TestCommand {
        self.envs.push((key.into(), value.into()));
        self
    }

    pub fn run(&self) -> TestCommandResult {
        let mut command = Command::new(&self.top_level_command);
        for &(ref k, ref v) in &self.envs {
            command.env(&k, &v);
        }
        let output = command
            .args(&self.args)
            .current_dir(&self.path)
            .output()
            .unwrap();

        TestCommandResult {
            output: output,
        }
    }
}

pub struct TestCommandResult {
    output: Output,
}

impl TestCommandResult {
    pub fn is_success(&self) -> bool {
        self.output.status.success()
    }

    pub fn stdout(&self) -> &str {
        str::from_utf8(&self.output.stdout).unwrap()
    }

    pub fn stderr(&self) -> &str {
        str::from_utf8(&self.output.stderr).unwrap()
    }

    pub fn failure_message(&self, expectation: &str) -> String {
        format!("Expected {}, instead found\nstdout: {}\nstderr: {}\n",
                expectation,
                self.stdout(),
                self.stderr())
    }
}

fn path_to_git_clean() -> String {
    let path = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join("debug")
        .join("git-clean")
        .to_str().unwrap()
        .to_owned();
    println!("Path is: {:?}", path);
    path
}
