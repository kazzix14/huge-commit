mod app;
mod cli;
mod committer;
mod config;
mod confirmor;
mod model;
mod comment_generator;
mod prompt_translator;

use app::App;
use chrono::TimeZone;
use clap::Parser;

use std::error::Error;

#[derive(Debug, thiserror::Error)]
enum UserError {
    #[error("No changes to commit.")]
    NoChangesToCommit,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let app = App::new()?;

    let base_message = args.base_message.map(|message| message.join(" "));

    match args.command {
        None | Some(cli::Command::Commit) => app.commit(base_message, args.assume_yes).await?,
        Some(cli::Command::Config(config::Command::Get { key })) => {
            if let Some(value) = config::get(&key)? {
                println!("{}", value);
            } else {
                println!("not set");
            }
        }
        Some(cli::Command::Config(config::Command::Set { key, value })) => {
            config::set(&key, Some(value))?
        }
        Some(cli::Command::Model(model::Command::List)) => {
            let models = model::list().await?;

            models.iter().for_each(|model| {
                let created_at = model
                    .created
                    .and_then(|created_at| {
                        chrono::Local
                            .timestamp_opt(created_at, 0)
                            .single()
                            .map(|datetime| datetime.to_rfc2822())
                    })
                    .unwrap_or("n/a".to_string());

                println!(
                    r#"{}
  created_at: {}
  owned_by: {}
                "#,
                    model.id, created_at, model.owned_by
                );
            });
        }
    };

    Ok(())
}
