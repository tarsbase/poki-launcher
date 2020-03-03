pub mod apps;

use crate::config::Config;
use crate::event::Event;
use crate::ListItem;
use anyhow::Result;
use log::error;
use std::sync::mpsc::Sender;

pub fn init_plugins(config: &Config) -> Vec<Box<dyn Plugin>> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
    match self::apps::Apps::init(&config) {
        Ok(apps) => plugins.push(Box::new(apps)),
        Err(e) => {
            error!("{}", e);
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
