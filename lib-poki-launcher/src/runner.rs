use failure::Error;
use nix::unistd::{getpid, setpgid};
use poki_launcher_x11::forground;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};

use super::App;

fn parse_exec<'a>(exec: &'a str) -> (&'a str, Vec<&'a str>) {
    let mut iter = exec.split(" ");
    let cmd = iter.next().expect("Empty Exec");
    let args = iter.collect();
    (cmd, args)
}

impl App {
    pub fn run(&self) -> Result<(), Error> {
        let (cmd, args) = parse_exec(&self.exec);
        let mut command = Command::new(&cmd);
        command
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            command.pre_exec(|| {
                let pid = getpid();
                // TODO Hanle error here
                setpgid(pid, pid).expect("Failed to set pgid");
                Ok(())
            });
        }
        let _child = command.spawn()?;
        forground(cmd);
        Ok(())
    }
}
