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
use crate::run::*;
use anyhow::{Context as _, Result};
use log::debug;
use std::process::Command;

use super::App;

fn with_term<'a>(
    term_cmd: &'a Option<String>,
    exec: &'a str,
) -> Result<(String, Vec<&'a str>)> {
    if let Some(term) = term_cmd {
        let mut args: Vec<&str> =
            term.split(' ').chain(exec.split(' ')).collect();
        let term = args.remove(0).to_owned();
        Ok((term, args))
    } else {
        let term = std::env::var("TERM").context(
            "Tried to start a terminal app but the \
        TERM environment variable is not set so I don't know what terminal\
        program to use.  To fix this either set the TERM variable or set\
        term_cmd in the config file with the command you want to use\
        to start your terminal.",
        )?;
        let mut args: Vec<&str> = exec.split(' ').collect();
        args.insert(0, "-e");
        Ok((term, args))
    }
}

impl App {
    /// Run the app.
    pub fn run(&self, term_cmd: &Option<String>) -> Result<()> {
        debug!("Exec: `{}`", self.exec);
        let (cmd, args) = if self.terminal {
            with_term(&term_cmd, &self.exec)?
        } else {
            parse_command_string(&self.exec)
        };
        debug!("Running `{} {}`", cmd, args.join(" "));
        let mut command = Command::new(&cmd);
        command.args(&args);
        let _ = run_bg(command).with_context(|| {
            format!(
                "Execution failed with Exec line: `{}` `{}`.\n\
            If I'm trying to start your terminal emulator with \
            the wrong options please set term_cmd in the config \
            file with the correct command",
                cmd,
                args.join(" ")
            )
        })?;
        Ok(())
    }
}
