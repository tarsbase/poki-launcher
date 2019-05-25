use ini::Ini;
use failure::{Error, Fail};

#[derive(Debug, Clone)]
pub (super) struct DesktopEntry {
    name: String,
    exec: String,
}

#[derive(Debug, Fail)]
pub (super) enum DesktopEntryParseError {
    #[fail(display = "Desktop file is missing 'Desktop Entry' section")]
    MissingSection,
}

pub (super) fn parse_desktop_file(path: &str) -> Result<DesktopEntry, Error> {
    // TODO Finish implementation
    let file = Ini::load_from_file(path)?;
    let entry = file.section(Some("Desktop Entry".to_owned())).ok_or(DesktopEntryParseError::MissingSection)?;
    Ok(DesktopEntry {
        name: entry.get("Name").unwrap().clone(),
        exec: entry.get("Exec").unwrap().clone(),
    })
}