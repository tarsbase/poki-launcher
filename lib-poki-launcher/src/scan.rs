use crate::db::AppsDB;
use crate::desktop_entry::{parse_desktop_file, DesktopEntryParseError};
use crate::App;
use failure::{Error, Fail};
use std::fs::read_dir;
use std::path::PathBuf;

#[derive(Debug, Fail)]
pub enum ScanError {
    #[fail(
        display = "Failed to scan directory {} for desktop entries: {}",
        dir, err
    )]
    ScanDirectory { dir: String, err: Error },
    #[fail(display = "Parse error: {}", err)]
    ParseEntry { err: DesktopEntryParseError },
}

pub fn desktop_entires(paths: &[String]) -> (Vec<PathBuf>, Vec<Error>) {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for loc in paths {
        match read_dir(&loc) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry.file_name().to_str().unwrap().contains(".desktop") {
                                files.push(entry.path())
                            }
                        }
                        Err(e) => {
                            errors.push(
                                ScanError::ScanDirectory {
                                    dir: loc.clone(),
                                    err: e.into(),
                                }
                                .into(),
                            );
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(
                    ScanError::ScanDirectory {
                        dir: loc.clone(),
                        err: e.into(),
                    }
                    .into(),
                );
            }
        }
    }
    (files, errors)
}

fn scan_desktop_entries(paths: &[String]) -> (Vec<App>, Vec<Error>) {
    let (entries, mut errors) = desktop_entires(&paths);
    let (apps, errs): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .map(|path| parse_desktop_file(&path))
        .partition(Result::is_ok);
    let mut apps: Vec<_> = apps
        .into_iter()
        .map(Result::unwrap)
        .filter_map(|x| x)
        .collect();
    apps.sort_unstable();
    apps.dedup();
    // TODO Don't ignore errors
    errors.extend(errs.into_iter().map(Result::unwrap_err).collect::<Vec<_>>());
    (apps, errors)
}

impl AppsDB {
    pub fn from_desktop_entries(paths: &[String]) -> (AppsDB, Vec<Error>) {
        let (apps, errors) = scan_desktop_entries(paths);
        (AppsDB::new(apps), errors)
    }

    pub fn rescan_desktop_entries(&mut self, paths: &[String]) -> Vec<Error> {
        let (apps, errors) = scan_desktop_entries(paths);
        self.merge(apps);
        errors
    }
}
