use super::Plugin;
use crate::config::Config;
use anyhow::Result;

pub struct Files {}

impl Files {
    pub fn init(_: &Config) -> Result<Self> {
        Ok(Files {})
    }
}

impl Plugin for Files {
    fn matcher(&self, _config: &Config, input: &str) -> bool {
        match input.get(0..1) {
            Some(":") => true,
            _ => false,
        }
    }
    fn search(
        &self,
        config: &Config,
        input: &str,
        num_items: usize,
    ) -> Result<Vec<crate::ListItem>> {
        unimplemented!()
    }
    fn run(&mut self, config: &Config, id: &str) -> Result<()> {
        unimplemented!()
    }
}
