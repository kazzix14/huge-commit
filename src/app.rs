use crate::{cli, config};

pub struct App {
  config: config::Config,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
      Ok(App {
        config: config::read_config()?,
      })
    }

    pub fn get_config(&self, key: &config::Item) -> Option<String> {
      self.config.get(key)
    }

    pub fn set_config(&self, key: &config::Item, value: Option<String>) {
      self.config.set(key, value)
    }
}

