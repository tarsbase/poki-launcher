/***
 * This file is part of Poki Launcher.
 *
 * Poki Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Poki Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Poki Launcher.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::config::Config;
use failure::{Error, Fail};
use log::debug;
use nix::unistd::{getpid, setpgid};
use poki_launcher_x11::foreground;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};

use super::App;

/// An error from running the app.
#[derive(Debug, Fail)]
#[fail(display = "Execution failed with Exec line {}: {}", exec, err)]
pub struct RunError {
    /// The exec string from the app
    exec: String,
    /// The error to propagate.
    err: Error,
}

fn parse_exec<'a>(exec: &'a str) -> (String, Vec<&'a str>) {
    let mut iter = exec.split(' ');
    let cmd = iter.next().expect("Empty Exec").to_owned();
    let args = iter.collect();
    (cmd, args)
}

fn with_term<'a>(config: &Config, exec: &'a str) -> Result<(String, Vec<&'a str>), Error> {
    let term = if let Some(term) = &config.term_cmd {
        term.clone()
    } else {
        std::env::var("TERM")?
    };
    let mut args: Vec<&str> = exec.split(' ').collect();
    args.insert(0, "-e");
    Ok((term, args))
}

impl App {
    /// Run the app.
    pub fn run(&self, config: &Config) -> Result<(), Error> {
        debug!("Exec: `{}`", self.exec);
        let (cmd, args) = if self.terminal {
            with_term(&config, &self.exec)?
        } else {
            parse_exec(&self.exec)
        };
        debug!("Running `{} {}`", cmd, args.join(" "));
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
        let _child = command.spawn().map_err(|e| RunError {
            exec: self.exec.clone(),
            err: e.into(),
        })?;
        foreground(&cmd);
        Ok(())
    }
}
