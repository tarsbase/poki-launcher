use failure::Error;

use std::fs::read_dir;
use std::path::PathBuf;
use crate::App;
use std::fs::File;
use std::io::Read;
use crate::desktop_entry::parse_desktop_file;

fn search_locations() -> Vec<&'static str> {
    vec!["/usr/share/applications"]
}

pub fn desktop_files() -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for loc in search_locations() {
        for entry in read_dir(&loc)? {
            let entry = entry?;
            if entry.file_name().to_str().unwrap().contains(".desktop") {
                files.push(entry.path())
            }
        }
    }
    Ok(files)
}

pub fn desktop_parse_entries(files: Vec<PathBuf>) -> Result<Vec<App>, Error> {
    files.iter().map(|path| {
        let mut file = File::open(&path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        parse_desktop_file(&buf)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan() {
        let entries = desktop_files();
        #[allow(unused_must_use)]
        dbg!(entries);
    }
}