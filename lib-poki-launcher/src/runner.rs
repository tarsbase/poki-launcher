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
use failure::{Error, Fail};
use nix::unistd::{getpid, setpgid};
use poki_launcher_x11::forground;
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

fn parse_exec<'a>(exec: &'a str) -> (&'a str, Vec<&'a str>) {
    let mut iter = exec.split(' ');
    let cmd = iter.next().expect("Empty Exec");
    let args = iter.collect();
    (cmd, args)
}

impl App {
    /// Run the app.
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
        let _child = command.spawn().map_err(|e| RunError {
            exec: self.exec.clone(),
            err: e.into(),
        })?;
        forground(cmd);
        Ok(())
    }
}
