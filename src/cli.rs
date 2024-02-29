use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Commit,
    #[clap(subcommand)]
    Config(config::Command),
}

pub mod config {
    use clap::{Subcommand, ValueEnum};

    #[derive(Debug, Subcommand)]
    pub enum Command {
        List,
        Get {
            #[clap(index = 1)]
            key: Item,
        },
        Set {
            #[clap(index = 1)]
            key: Item,

            #[clap(index = 2)]
            value: String,
        },
    }

    #[derive(Debug, Clone, Copy, ValueEnum)]
    pub enum Item {
        OpenaiApiKey,
    }
}
