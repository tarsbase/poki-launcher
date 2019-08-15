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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run_xterm() {
        let app = App::new("xTerm".to_owned(), "xterm".to_owned());
        app.run().unwrap();
    }

    #[test]
    fn run_terminator() {
        let app = App::new("Terminator".to_owned(), "terminator".to_owned());
        app.run().unwrap();
    }

    #[test]
    fn run_vim() {
        let app = App::new("Vim".to_owned(), "vim %F".to_owned());
        app.run().unwrap();
    }

    #[test]
    fn run_error() {
        // TODO This should probably produce an error
        let app = App::new("Error".to_owned(), "awdawdawdawdawd".to_owned());
        app.run().unwrap();
    }
}