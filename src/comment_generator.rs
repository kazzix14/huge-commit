use crate::prompt_translator::PromptTranslator;

pub struct CommentGenerator {
    prompt_translator: PromptTranslator,
    base_message: Option<String>,
}

impl CommentGenerator {
    pub fn new(prompt_translator: PromptTranslator, base_message: Option<String>) -> Self {
        CommentGenerator {
            prompt_translator,
            base_message,
        }
    }

    pub async fn gen_commit_message<'a>(&self, diff: String) -> anyhow::Result<String> {
        let base_message_prompt = self
            .base_message
            .as_ref()
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
"#
        );

        let mut response_rx = self.prompt_translator.translate(prompt).await?;
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
