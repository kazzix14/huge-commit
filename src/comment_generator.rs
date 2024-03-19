use crate::prompt_translator::PromptTranslator;
use futures::StreamExt;
use tokio::pin;

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
- return commit message in only 1 line.
- Explain the reason behind the changes.
- Only provide the commit message without starting with "Commit message:".
- If you can't fit everything in 10 words, prioritize the most important information.
- Use present tense verbs, e.g., "Add feature" instead of "Added feature".
- Do not write things that aren't related to the changes. Meaning, upgrading version of program itself does not means features is added or changed

basic comment message format is `verb` + subject + details`. you should not start with `action:` or `action(..):`. use normal sentence.
you may choose action from following list. if you can't find suitable action, you can use other action or write your own action.
- `feat` new feature
- `fix` bug fix
- `docs` documentation
- `refactor` A code change that neither fixes a bug nor adds a feature. also includes white space, formatting, missing semi-colons. never include new feature or bug fix. this is not preferred action. use other if possible.
- `enhance` A code change that improves UX. like performance, messages. never include new feature or bug fix. this is not preferred action. use other if possible.
- `test` Adding missing tests or correcting existing tests
- `build` Changes that affect the build system, includes ci
- `deps` dependency updates
- or you can add your own action, if you can't find suitable action from above list.

{base_message_prompt}

```diff
{diff}
```
"#
        );

        let response_rx = self.prompt_translator.translate(prompt).await?;
        pin!(response_rx);

        let mut commit_message = String::new();
        while let Some(chunk) = response_rx.next().await {
            commit_message.push_str(&chunk.to_string());
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        println!();

        Ok(commit_message)
    }
}
