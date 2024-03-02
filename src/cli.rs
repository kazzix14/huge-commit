use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    #[clap(short = 'm', long, help = "The base message to use for the commit.", num_args(1..))]
    pub base_message: Option<Vec<String>>,

    #[clap(
        short = 'y',
        long,
        default_value = "false",
        help = "Assume yes to all prompts."
    )]
    pub assume_yes: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "Commit changes.")]
    Commit,

    #[clap(subcommand, about = "Get or set configuration.")]
    Config(crate::config::Command),

    #[clap(subcommand, about = "models.")]
    Model(crate::model::Command),
}
