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
use anyhow::{Context as _, Error, Result};
use log::debug;
use nix::unistd::{getpid, setpgid};
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};

use super::App;

fn parse_exec<'a>(exec: &'a str) -> (String, Vec<&'a str>) {
    let mut iter = exec.split(' ');
    let cmd = iter.next().expect("Empty Exec").to_owned();
    let args = iter.collect();
    (cmd, args)
}

fn with_term<'a>(
    config: &Config,
    exec: &'a str,
) -> Result<(String, Vec<&'a str>)> {
    let term = if let Some(term) = &config.file_options.term_cmd {
        term.clone()
    } else {
        std::env::var("TERM").context(
            "Tried to start a terminal app but the \
        TERM environment variable is not set so I don't know what terminal\
        program to use.  To fix this either set the TERM variable or set\
        term_cmd in the config file with the command you want to use\
        to start your terminal.",
        )?
    };
    let mut args: Vec<&str> = exec.split(' ').collect();
    args.insert(0, "-e");
    Ok((term, args))
}

impl App {
    /// Run the app.
    pub fn run(&self, config: &Config) -> Result<()> {
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
                if let Err(e) = setpgid(pid, pid) {
                    log::error!(
                        "{}",
                        Error::new(e).context(format!(
                            "Failed to set pgid of child process with pid {}",
                            pid
                        ))
                    );
                }
                Ok(())
            });
        }
        let _child = command.spawn().with_context(|| {
            format!(
                "Execution failed with Exec line {}.\n\
            If I'm trying to start your terminal emulator with\
            the wrong options please set term_cmd in the config\
            file with the correct command",
                self.exec
            )
        })?;
        Ok(())
    }
}
