use super::App;
use failure::{Error, Fail};
use ini::Ini;
use itertools::Itertools;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Fail)]
pub enum DesktopEntryParseError {
    #[fail(display = "Desktop file is missing 'Desktop Entry' section")]
    MissingSection,
    #[fail(display = "Desktop file is missing the 'Name' parameter")]
    MissingName,
    #[fail(display = "Desktop file is missing the 'Exec' parameter")]
    MissingExec,
    #[fail(display = "Desktop file is missing the 'Icon' parameter")]
    MissingIcon,
}

fn strip_args(exec: &str) -> String {
    exec.split(" ")
        .filter(|item| !item.starts_with("%"))
        .join(" ")
}

pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<App, Error> {
    // TODO Finish implementation
    let file = Ini::load_from_file(path)?;
    let entry = file
        .section(Some("Desktop Entry".to_owned()))
        .ok_or(DesktopEntryParseError::MissingSection)?;
    let name = entry
        .get("Name")
        .ok_or(DesktopEntryParseError::MissingName)?
        .clone();
    let exec = entry
        .get("Exec")
        .ok_or(DesktopEntryParseError::MissingExec)?
        .clone();
    let icon = entry
        .get("Icon")
        .ok_or(DesktopEntryParseError::MissingIcon)?
        .clone();
    let exec = strip_args(&exec);
    Ok(App::new(name, icon, exec))
}
