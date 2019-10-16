use crate::DIRS;
use failure::Error;
use serde_derive::Deserialize;
use std::default::Default;
use std::fs::create_dir;

#[derive(Deserialize)]
pub struct Config {
    pub app_paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_paths: vec!["/usr/share/applications".to_owned()],
        }
    }
}

impl Config {
    pub fn load() -> Result<Config, Error> {
        let mut cfg = config::Config::default();
        let config_dir = DIRS.config_dir();
        let mut file_path = None;
        if !config_dir.exists() {
            create_dir(&config_dir)?;
        }
        for entry in config_dir.read_dir()? {
            if let Ok(entry) = entry {
                if entry
                    .file_name()
                    .into_string()
                    .unwrap()
                    .starts_with("poki-launcher")
                {
                    file_path = Some(entry.path());
                }
            }
        }
        let file_path = match file_path {
            Some(p) => p,
            None => {
                return Ok(Self::default());
            }
        };
        cfg.merge(config::File::with_name(file_path.to_str().unwrap()))?;
        Ok(cfg.try_into()?)
    }
}
