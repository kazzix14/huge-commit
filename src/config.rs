use clap::Subcommand;

use std::{borrow::Borrow, fs::File, io::Write};

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(arg_required_else_help = true, flatten_help = true)]
    Get {
        #[clap(index = 1)]
        key: Item,
    },
    #[clap(arg_required_else_help = true, flatten_help = true)]
    Set {
        #[clap(index = 1)]
        key: Item,

        #[clap(index = 2)]
        value: String,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Item {
    OpenaiApiKey,
    OpenaiModel,
    ConfigPath,
}

pub fn get<K: Borrow<Item>>(key: K) -> anyhow::Result<Option<String>> {
    let config = read_config()?;

    let value = match key.borrow() {
        Item::OpenaiApiKey => config.openai_api_key,
        Item::OpenaiModel => config.openai_model,
        Item::ConfigPath => Some(config_path()?.to_string_lossy().to_string()),
    };

    Ok(value)
}

pub fn set<K: Borrow<Item>>(key: K, value: Option<String>) -> anyhow::Result<()> {
    let mut config = read_config()?;

    match key.borrow() {
        Item::OpenaiApiKey => config.openai_api_key = value,
        Item::OpenaiModel => config.openai_model = value,
        Item::ConfigPath => unimplemented!("Setting config path is currently not supported."),
    };

    write_config(&config)?;

    Ok(())
}

fn config_path() -> anyhow::Result<std::path::PathBuf> {
    let base_dir = directories::BaseDirs::new().expect("Failed to get base directories");
    let cache_dir = base_dir.cache_dir();
    let config_path = cache_dir.join("huge-commit/config.toml");

    std::fs::create_dir_all(config_path.parent().expect("Failed to get parent"))?;

    Ok(config_path)
}

fn read_config() -> anyhow::Result<Config> {
    if !config_path()?.exists() {
        std::fs::File::create(config_path()?)?;
    }

    let config = std::fs::read_to_string(&mut config_path()?)?;

    Ok(toml::from_str::<Config>(&config).expect("Failed to parse config file"))
}

fn write_config(config: &Config) -> anyhow::Result<()> {
    let mut file = File::create(config_path()?)?;

    file.write_all(toml::to_string(config)?.as_bytes())?;

    Ok(())
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub openai_model: Option<String>,
}
