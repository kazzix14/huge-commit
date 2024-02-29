use clap::Subcommand;
use directories;
use serde::Deserialize;
use std::{
    fs::File,
    io::{Read, Write},
};

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
}

pub fn get(key: Item) -> anyhow::Result<Option<String>> {
    let config = read_config()?;

    let value = match key {
        Item::OpenaiApiKey => config.openai_api_key,
        Item::OpenaiModel => config.openai_model,
    };

    Ok(value)
}

pub fn set(key: Item, value: String) -> anyhow::Result<()> {
    let mut config = read_config()?;

    match key {
        Item::OpenaiApiKey => config.openai_api_key = Some(value),
        Item::OpenaiModel => config.openai_model = Some(value),
    };

    write_config(&config)?;

    Ok(())
}

fn config_path() -> anyhow::Result<std::path::PathBuf> {
    let base_dir = directories::BaseDirs::new().expect("Failed to get base directories");
    let cache_dir = base_dir.cache_dir();
    let config_path = cache_dir.join("huge-commit/config.toml");

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
    let mut file = File::create(&config_path()?)?;

    file.write_all(toml::to_string(config)?.as_bytes())?;

    Ok(())
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Config {
    openai_api_key: Option<String>,
    openai_model: Option<String>,
}
