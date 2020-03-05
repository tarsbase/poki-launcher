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
use anyhow::{Context as _, Error, Result};
use nix::unistd::{getpid, setpgid};
use std::os::unix::process::CommandExt as _;
use std::process::{Child, Command, Stdio};

pub fn parse_command_string<'a>(exec: &'a str) -> (String, Vec<&'a str>) {
    let mut iter = exec.split(' ');
    let cmd = iter.next().expect("Empty Exec").to_owned();
    let args = iter.collect();
    (cmd, args)
}

/// Run a command in the background, moved out of poki launcher's process group
pub fn run_bg(mut command: Command) -> Result<Child> {
    command.stdout(Stdio::null()).stderr(Stdio::null());
    unsafe {
        command.pre_exec(|| {
            let pid = getpid();
            if let Err(e) = setpgid(pid, pid) {
                log::error!(
                    "{:?}",
                    Error::new(e).context(format!(
                        "Failed to set pgid of child process with pid {}",
                        pid
                    ))
                );
            }
            Ok(())
        });
    }
    Ok(command
        .spawn()
        .with_context(|| format!("Execution of command`{:?}`", command))?)
}
