use failure::Error;

use std::fs::read_dir;
use std::path::PathBuf;

fn search_locations() -> Vec<&'static str> {
    vec!["/usr/share/applications"]
}

pub fn desktop_entries() -> Result<Vec<PathBuf>, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan() {
        let entries = desktop_entries();
        dbg!(entries);
    }
}