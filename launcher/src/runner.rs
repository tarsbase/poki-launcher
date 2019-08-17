use failure::Error;
use std::process::{Command, Stdio};

use super::App;

impl App {
    #[allow(dead_code)]
    pub fn run(&self) -> Result<(), Error> {
        Command::new("bash")
            .args(vec!["-c", &self.exec])
            .stdout(Stdio::null())
            .spawn()?;
        Ok(())
    }
}
