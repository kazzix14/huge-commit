use crate::config;

use git2::{DiffFormat, Repository};
use openai::chat::{ChatCompletionDelta, ChatCompletionMessage};

use std::fmt::Write;
use std::io::Read;

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

    pub fn set_config(&mut self, key: &config::Item, value: Option<String>) {
        self.config.set(key, value)
    }

    pub async fn commit(
        &self,
        base_message: Option<String>,
        assume_yes: bool,
    ) -> anyhow::Result<()> {
        let repo = Repository::open(".")?;
        let diff = self.get_diff(&repo)?;

        if !self.diff_has_change(&diff)? {
            self.stage_all_files(&repo, assume_yes)?;
        }

        let diff = self.get_diff(&repo)?;
        if !self.diff_has_change(&diff)? {
            Err(crate::UserError::NoChangesToCommit.into())
        } else {
            let commit_message = self.gen_commit_message(base_message, &diff).await?;

            self.commit_changes(&repo, &commit_message, assume_yes)?;
            Ok(())
        }
    }

    fn confirm(&self, message: &'static str, default: bool) -> bool {
        let confirm = inquire::Confirm::new(message)
            .with_default(default)
            .prompt()
            .expect("Failed to get user input");
        println!();

        confirm
    }

    fn get_diff<'a>(&self, repo: &'a git2::Repository) -> anyhow::Result<git2::Diff<'a>> {
        let index = repo.index()?;
        let head_commit = repo.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), None)?;

        Ok(diff)
    }

    fn diff_has_change(&self, diff: &git2::Diff) -> anyhow::Result<bool> {
        Ok(0 < diff.stats()?.files_changed())
    }

    fn stage_all_files(&self, repo: &Repository, assume_yes: bool) -> anyhow::Result<()> {
        let mut index = repo.index()?;

        let stage = if assume_yes {
            true
        } else {
            self.confirm("No changes to commit. stage all changes?", true)
        };

        println!();

        if stage {
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .expect("Failed to stage changes");
            index.write().expect("Failed to write index");
            Ok(())
        } else {
            Err(crate::UserError::NoChangesToCommit.into())
        }
    }

    fn commit_changes(
        &self,
        repo: &Repository,
        commit_message: &str,
        assume_yes: bool,
    ) -> anyhow::Result<()> {
        let mut index = repo.index()?;

        let commit = if assume_yes {
            true
        } else {
            self.confirm("commit with this message?", true)
        };

        if commit {
            let sig = repo.signature()?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            let head = repo.head()?.peel_to_commit()?;
            repo.commit(Some("HEAD"), &sig, &sig, commit_message, &tree, &[&head])?;
        };

        Ok(())
    }

    async fn gen_commit_message<'a>(
        &self,
        base_message: Option<String>,
        diff: &git2::Diff<'a>,
    ) -> anyhow::Result<String> {
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

        let api_key = config::get(config::Item::OpenaiApiKey)?.expect("openai-api-key not set");

        openai::set_key(api_key);

        let base_message_prompt = base_message
            .map(|message| {
                format!(
                    r#"
I'll put rough comment message, you should write commit message based on it.
```rough commit message
{}
```
"#,
                    message
                )
            })
            .unwrap_or("".to_string());

        let prompt = format!(
            r#"
Write a commit message for the changes I will write at the end of this message.
- Limit the first line to a maximum of 10 words.
- Explain the reason behind the changes, not the specific details of what was changed.
- Focus on the big picture, rather than small details.
- Only provide the commit message without starting with "Commit message:".
- If you can't fit everything in 10 words, prioritize the most important information.
- Use present tense verbs, e.g., "Add feature" instead of "Added feature".
- Do not write things that aren't related to the changes. e.g., 'upgrading version of program itself does not means features is added or changed'
{base_message_prompt}
```diff
{diff}
```
"#,
            base_message_prompt = base_message_prompt,
            diff = diff_buf,
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
}
