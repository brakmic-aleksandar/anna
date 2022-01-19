use std::path::{
    Path,
    PathBuf
};
use std::process::Command;

pub struct Repo {
    path: PathBuf
}

impl Repo {
    pub fn new(path: &Path) -> Repo {
        Repo {
            path: path.to_path_buf()
        }
    }

    fn cmd_builder(&self, git_cmd: &str) -> Command {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(self.path.to_str().unwrap()).arg(git_cmd);

        cmd
    }

    fn perform_cmd(cmd: &mut Command) -> Result<(), ()> {
        match cmd.status() {
            Ok(status) if status.code().unwrap() == 0 => Ok(()),
            _ => Err(())
        }
    }

    pub fn pull(&self) -> Result<(), ()> {
        let mut cmd = self.cmd_builder("pull");

        Repo::perform_cmd(&mut cmd)
    }

    pub fn add(&self, file_path: &Path) -> Result<(), ()> {
        let mut cmd = self.cmd_builder("add");
        cmd.arg(file_path);

        Repo::perform_cmd(&mut cmd)
    }

    pub fn commit(&self, msg: &str) -> Result<(), ()> {
        let mut cmd = self.cmd_builder("commit");
        cmd.arg("--message")
           .arg(msg);

        Repo::perform_cmd(&mut cmd)
    }

    pub fn push(&self) -> Result<(), ()> {
        let mut cmd = self.cmd_builder("push");

        Repo::perform_cmd(&mut cmd)
    }
}
