use serde_derive::Deserialize;
use crate::DIRS;
use failure::{Error, err_msg};

#[derive(Deserialize)]
pub struct Config {
    pub app_paths: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Config, Error> {
        let config_dir = DIRS.config_dir();
        let mut file_path = None;
        for entry in config_dir.read_dir()? {
            if let Ok(entry) = entry {
                if entry.file_name().into_string().unwrap().starts_with("poki-launcher") {
                    file_path = Some(entry.path());
                }
            }
        }
        let file_path = match file_path {
            Some(p) => p,
            None => {
                return Err(err_msg("Config file not found"));
            }
        };
        let mut cfg = config::Config::default();
        cfg.merge(config::File::with_name(file_path.to_str().unwrap()))?;
        Ok(cfg.try_into()?)
    }
}
