mod cli;
mod config;
mod model;

use chrono::TimeZone;
use clap::Parser;

use git2::{DiffFormat, Repository};
use openai::chat::{ChatCompletionDelta, ChatCompletionMessage};

use std::error::Error;
use std::fmt::Write;
use std::io::Read;

#[derive(Debug, thiserror::Error)]
enum UserError {
    #[error("No changes to commit.")]
    NoChangesToCommit,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    match args.command {
        None | Some(cli::Command::Commit) => commit().await?,
        Some(cli::Command::Config(config::Command::Get { key })) => {
            if let Some(value) = config::get(key)? {
                println!("{}", value);
            } else {
                println!("not set");
            }
        }
        Some(cli::Command::Config(config::Command::Set { key, value })) => config::set(key, value)?,
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

async fn commit() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let diff = get_diff(&repo)?;

    if !diff_has_change(&diff)? {
        stage_all_files(&repo)?;
    }

    let diff = get_diff(&repo)?;
    if !diff_has_change(&diff)? {
        Err(UserError::NoChangesToCommit.into())
    } else {
        let commit_message = gen_commit_message(&diff).await?;

        commit_changes(&repo, &commit_message)?;
        Ok(())
    }
}

fn commit_changes(repo: &Repository, commit_message: &str) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    let commit = inquire::Confirm::new("commit with this message?")
        .with_default(true)
        .prompt()
        .expect("Failed to get user input");
    println!();

    if commit {
        let sig = repo.signature()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head()?.peel_to_commit()?;
        repo.commit(Some("HEAD"), &sig, &sig, commit_message, &tree, &[&head])?;
    };

    Ok(())
}

async fn gen_commit_message<'a>(diff: &git2::Diff<'a>) -> anyhow::Result<String> {
    let mut diff_buf = String::new();

    let _ = &diff
        .print(DiffFormat::Patch, |_delta, _hunk, line| {
            let mut buf = String::new();

            line.content()
                .read_to_string(&mut buf)
                .expect("Failed to read line");

            diff_buf
                .write_fmt(format_args!("{} {}", line.origin(), buf))
                .expect("Failed to write diff");

            true
        })
        .expect("Failed to print diff");

    let api_key = config::get(config::Item::OpenaiApiKey)?.expect("open-api-key not set");

    openai::set_key(api_key);

    let prompt = format!(
        r#"
write a commit message for the changes I will write at the end of this message.
- NEVER WRITE MORE THAN 10 WORDS AT THE FIRST LINE.
- you must write WHY you made the changes, not WHAT you changed.
- YOU MUST NOT POINT TO SMALL DETAILS, BUT TO THE BIG PICTURE.
- ONLY RETURN COMMIT MESSAGE. DO NOT STATE WITH `Commit message:`
- priority is feature, then fix, then refactor, then style. if you feel you can not fit everything in 10 words, then you can skip where priority is low.

diff:
```
{}
```
"#,
        diff_buf
    );

    let mut response_rx = ChatCompletionDelta::builder(
        &config::get(config::Item::OpenaiModel)?.unwrap_or("gpt-4-turbo-preview".to_string()),
        [ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::Assistant,
            content: Some(prompt),
            name: None,
            function_call: None,
        }],
    )
    .create_stream()
    .await?;

    let mut commit_message = String::new();
    while let Some(response) = response_rx.recv().await {
        response.choices.iter().for_each(|choice| {
            if let Some(content) = &choice.delta.content {
                commit_message.push_str(content);
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        });
    }
    println!();

    Ok(commit_message)
}

fn diff_has_change(diff: &git2::Diff) -> anyhow::Result<bool> {
    Ok(0 < diff.stats()?.files_changed())
}

fn stage_all_files(repo: &Repository) -> anyhow::Result<()> {
    let mut index = repo.index()?;
    let stage = inquire::Confirm::new("No changes to commit. stage all changes?")
        .with_default(true)
        .prompt()
        .expect("Failed to get user input");

    println!();

    if stage {
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .expect("Failed to stage changes");
        index.write().expect("Failed to write index");
        Ok(())
    } else {
        Err(UserError::NoChangesToCommit.into())
    }
}

fn get_diff(repo: &git2::Repository) -> anyhow::Result<git2::Diff<'_>> {
    let index = repo.index()?;
    let head_commit = repo.head()?.peel_to_commit()?;
    let head_tree = head_commit.tree()?;
    let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), None)?;

    Ok(diff)
}
