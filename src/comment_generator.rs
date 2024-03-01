use crate::config;
use git2::DiffFormat;
use openai::chat::{ChatCompletionDelta, ChatCompletionMessage};

use std::fmt::Write;
use std::io::Read;

pub struct CommentGenerator {}

impl CommentGenerator {
    pub fn new() -> anyhow::Result<Self> {
        Ok(CommentGenerator {})
    }

    pub async fn gen_commit_message<'a>(
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
If the base message looks like a command, that means a user executed it on the codebase. put the command inside your message.
e.g. `rubocop -a` -> "Run `rubocop -a`"
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
- Do not write things that aren't related to the changes. Meaning, upgrading version of program itself does not means features is added or changed
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
