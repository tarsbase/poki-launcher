use super::App;
use failure::{Error, Fail};
use ini::Ini;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Fail)]
pub enum EntryParseError {
    #[fail(display = "Desktop file {} is missing 'Desktop Entry' section", file)]
    MissingSection { file: String },
    #[fail(display = "Desktop file {} is missing the 'Name' parameter", file)]
    MissingName { file: String },
    #[fail(display = "Desktop file {} is missing the 'Exec' parameter", file)]
    MissingExec { file: String },
    #[fail(display = "Desktop file {} is missing the 'Icon' parameter", file)]
    MissingIcon { file: String },
    #[fail(display = "Failed to parse deskop file {}: {}", file, err)]
    InvalidIni { file: String, err: Error },
    #[fail(
        display = "In entry {} property {} has an invalid value {}",
        file, name, value
    )]
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
    iter.filter(|item| !item.starts_with('%')).collect()
}

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
    Ok(Some(App::new(name, icon, exec)))
}
