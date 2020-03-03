pub mod apps;

use crate::config::Config;
use crate::event::Event;
use crate::ListItem;
use anyhow::Result;
use log::{error, warn};
use serde_json::Value;
use std::sync::mpsc::Sender;

fn get_plugin_list(plugins: &Value) -> Vec<&String> {
    if let Some(map) = plugins.as_object() {
        map.keys().collect()
    } else {
        vec![]
    }
}

pub fn init_plugins(config: &Config) -> Vec<Box<dyn Plugin>> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
    let plugin_list = get_plugin_list(&config.file_options.plugins);
    if plugin_list.is_empty() {
        warn!(
            "No plugins loading, launcher will do nothing. \
                You probably want to enable some plugins in the config file."
        )
    }
    for plugin_name in plugin_list {
        match plugin_name.as_str() {
            "apps" => match self::apps::Apps::init(&config) {
                Ok(apps) => plugins.push(Box::new(apps)),
                Err(e) => {
                    error!("{:?}", e);
                }
            },
            _ => error!("Unknown plugin: {}", plugin_name),
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
