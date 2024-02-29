use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    #[clap(short, long)]
    pub message: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Commit,

    #[clap(subcommand)]
    Config(crate::config::Command),

    #[clap(subcommand)]
    Model(crate::model::Command),
}
