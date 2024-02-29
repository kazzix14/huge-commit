use clap::Subcommand;

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

pub fn get(key: Item) -> () {
    println!("get {:?}", key);
}

pub fn set(key: Item, value: String) -> () {
    println!("set {:?} {:?}", key, value);
}
