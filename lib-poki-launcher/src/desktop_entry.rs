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
use super::App;
use failure::{Error, Fail};
use ini::Ini;
use itertools::Itertools as _;
use std::path::Path;

/// Error from paring a desktop entry
#[derive(Debug, Fail)]
pub enum EntryParseError {
    /// Desktop file is missing the 'Desktop Entry' section.
    #[fail(display = "Desktop file {} is missing 'Desktop Entry' section", file)]
    MissingSection { file: String },
    /// Desktop file is missing the 'Name' parameter.
    #[fail(display = "Desktop file {} is missing the 'Name' parameter", file)]
    MissingName { file: String },
    /// Desktop file is missing the 'Exec' parameter.
    #[fail(display = "Desktop file {} is missing the 'Exec' parameter", file)]
    MissingExec { file: String },
    /// Desktop file is missing the 'Icon' parameter.
    #[fail(display = "Desktop file {} is missing the 'Icon' parameter", file)]
    MissingIcon { file: String },
    /// Failed to parse deskop file.
    #[fail(display = "Failed to parse desktop file {}: {}", file, err)]
    InvalidIni { file: String, err: Error },
    #[fail(
        display = "In entry {} property {} has an invalid value {}",
        file, name, value
    )]
    /// A property had an invalid value.
    /// This is returned if NoDisplay or Hidden are set ti a value that isn't
    /// `true` or `false`.
    InvalidPropVal {
        file: String,
        name: String,
        value: String,
    },
}

fn prop_is_true(item: Option<&String>) -> Result<bool, Error> {
    match item {
        Some(text) => Ok(text.parse()?),
        None => Ok(false),
    }
}

fn strip_entry_args(exec: &str) -> String {
    let iter = exec.split(' ');
    iter.filter(|item| !item.starts_with('%')).join(" ")
}

/// Parse a desktop entry
///
/// # Arguments
///
/// * `path` - Path to the desktop entry
///
/// # Return
///
/// Returns `Ok(None)` if the app should not be listed.
///
/// # Example
///
/// Parse a list of desktop entries, separating successes from failures, then removing apps
/// that shouldn't be displayed (None) from the successes.
/// ```no_run
/// use lib_poki_launcher::desktop_entry::parse_desktop_file;
/// use std::path::Path;
///
/// let entries = vec![Path::new("./firefox.desktop"), Path::new("./chrome.desktop")];
/// let (apps, errors): (Vec<_>, Vec<_>) = entries
///     .into_iter()
///     .map(|path| parse_desktop_file(&path))
///     .partition(Result::is_ok);
/// let mut apps: Vec<_> = apps
///     .into_iter()
///     .map(Result::unwrap)
///     .filter_map(|x| x)
///     .collect();
/// ```
pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<Option<App>, Error> {
    let path_str = path.as_ref().to_string_lossy().into_owned();
    // TODO Finish implementation
    let file = Ini::load_from_file(path).map_err(|e| EntryParseError::InvalidIni {
        file: path_str.clone(),
        err: e.into(),
    })?;
    let entry =
        file.section(Some("Desktop Entry".to_owned()))
            .ok_or(EntryParseError::MissingSection {
                file: path_str.clone(),
            })?;
    if prop_is_true(entry.get("NoDisplay")).map_err(|_| EntryParseError::InvalidPropVal {
        file: path_str.clone(),
        name: "NoDisplay".into(),
        value: entry.get("NoDisplay").unwrap().clone(),
    })? || prop_is_true(entry.get("Hidden")).map_err(|_| EntryParseError::InvalidPropVal {
        file: path_str.clone(),
        name: "Hidden".into(),
        value: entry.get("Hidden").unwrap().clone(),
    })? {
        return Ok(None);
    }
    let name = entry
        .get("Name")
        .ok_or(EntryParseError::MissingName {
            file: path_str.clone(),
        })?
        .clone();
    let exec = entry
        .get("Exec")
        .ok_or(EntryParseError::MissingExec {
            file: path_str.clone(),
        })?
        .clone();
    let exec = strip_entry_args(&exec);
    let icon = entry
        .get("Icon")
        .ok_or(EntryParseError::MissingIcon {
            file: path_str.clone(),
        })?
        .clone();
    let terminal = {
        if let Some(value) = entry.get("Terminal") {
            value.parse()?
        } else {
            false
        }
    };
    Ok(Some(App::new(name, icon, exec, terminal)))
}

#[cfg(test)]
mod test {
    use super::*;

    mod strip_entry_args {
        use super::*;

        #[test]
        fn no_args() {
            let exec = "/usr/bin/cat --flag".to_owned();
            assert_eq!(strip_entry_args(&exec), exec);
        }

        #[test]
        fn has_args() {
            let exec = "/usr/bin/cat %f --flag";
            let exec_no_args = "/usr/bin/cat --flag".to_owned();
            assert_eq!(strip_entry_args(&exec), exec_no_args);
        }
    }

    mod parse_desktop_file {
        use super::*;

        #[test]
        fn vaild_file_exist() {
            use crate::App;
            use std::fs::{remove_file, File};
            use std::io::prelude::*;
            use std::path::Path;

            let path = Path::new("./test.desktop");
            let mut file = File::create(&path).unwrap();
            file.write_all(
                b"[Desktop Entry]
 Name=Test
 Icon=testicon
 Exec=/usr/bin/test --with-flag %f",
            )
            .unwrap();
            let app = parse_desktop_file(&path).unwrap().unwrap();
            let other_app = App::new(
                "Test".to_owned(),
                "testicon".to_owned(),
                "/usr/bin/test --with-flag".to_owned(),
                false,
            );
            // Note, apps will have different uuids but Eq doesn't consider them
            assert_eq!(app, other_app);
            remove_file(&path).unwrap();
        }
    }
}
