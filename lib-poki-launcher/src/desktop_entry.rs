use super::App;
use failure::{Error, Fail};
use ini::Ini;
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

fn remove_if_true(item: Option<&String>, app: App) -> Option<App> {
    match item {
        Some(text) => match text.parse() {
            Ok(item) => {
                if item {
                    None
                } else {
                    Some(app)
                }
            }
            Err(_) => {
                eprintln!("NoDisplay has a invalid value: {}", text);
                Some(app)
            }
        },
        None => Some(app),
    }
}

fn strip_entry_args<'a>(exec: &'a str) -> String {
    let iter = exec.split(" ");
    let args = iter.filter(|item| !item.starts_with("%")).collect();
    args
}

pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<Option<App>, Error> {
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
    let exec = strip_entry_args(&exec);
    let icon = entry
        .get("Icon")
        .ok_or(DesktopEntryParseError::MissingIcon)?
        .clone();
    let app = App::new(name, icon, exec);
    Ok(remove_if_true(entry.get("NoDisplay"), app.clone())
        .or(remove_if_true(entry.get("Hidden"), app)))
}
