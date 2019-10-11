use failure::Error;
use poki_launcher_x11::forground;
use std::process::{Command, Stdio};

use super::App;

fn parse_exec<'a>(exec: &'a str) -> (&'a str, Vec<&'a str>) {
    let mut iter = exec.split(" ");
    let cmd = iter.next().expect("Empty Exec");
    let args = iter.filter(|item| !item.starts_with("%")).collect();
    (cmd, args)
}

impl App {
    #[allow(dead_code)]
    pub fn run(&self) -> Result<(), Error> {
        let (cmd, args) = parse_exec(&self.exec);
        let child = Command::new(&cmd)
            .args(&args)
            .stdout(Stdio::null())
            .spawn()?;
        forground(cmd);
        Ok(())
    }
}
