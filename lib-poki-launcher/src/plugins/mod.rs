mod apps;
mod files;

use crate::config::Config;
use crate::event::Event;
use crate::ListItem;
use anyhow::Result;
use log::{error, info, warn};
use std::sync::mpsc::Sender;

pub fn init_plugins(config: &Config) -> Vec<Box<dyn Plugin>> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
    if config.file_options.plugin_load_order.is_empty() {
        warn!(
            "No plugins loading, launcher will do nothing. \
                You probably want to enable some plugins in the config file."
        )
    }
    for plugin_name in &config.file_options.plugin_load_order {
        match plugin_name.as_str() {
            "apps" => match self::apps::Apps::init(&config) {
                Ok(apps) => {
                    info!("Loading plugin: `apps`");
                    plugins.push(Box::new(apps))
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            },
            "files" => match self::files::Files::init(&config) {
                Ok(apps) => {
                    info!("Loading plugin: `files`");
                    plugins.push(Box::new(apps))
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            },
            _ => error!("Unknown plugin: `{}`", plugin_name),
        }
    }
    plugins
}

pub trait Plugin: Send + Sync {
    // fn init(config: &Config) -> Result<Box<Self>>;
    fn matcher(&self, config: &Config, input: &str) -> bool;
    fn search(
        &self,
        config: &Config,
        input: &str,
        num_items: usize,
    ) -> Result<Vec<ListItem>>;
    fn run(&mut self, config: &Config, id: &str) -> Result<()>;
    #[allow(unused_variables)]
    fn reload(&mut self, config: &Config) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn register_event_handlers(
        &mut self,
        config: &Config,
        event_tx: Sender<Event>,
    ) {
    }
}
