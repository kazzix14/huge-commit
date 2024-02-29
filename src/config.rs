use clap::Subcommand;

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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Item {
    OpenaiApiKey,
}

fn list() -> Vec<String> {
    clap::ValueEnum::value_variants()
        .iter()
        .map(|config| {
            <Item as clap::ValueEnum>::to_possible_value(&config)
                .unwrap()
                .get_name()
                .to_string()
        })
        .collect()
}
