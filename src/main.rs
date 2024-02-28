use git2::{DiffFormat, IndexAddOption, Repository};
use openai::chat::{ChatCompletion, ChatCompletionBuilder, ChatCompletionMessage};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Write;
use std::{default::Default, env};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let repo_path = ".";
    let repo = Repository::open(repo_path)?;

    // Stage changes
    //index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    //index.write()?;

    let head_commit = repo.head()?.peel_to_commit()?;
    let head_tree = head_commit.tree()?;

    let mut index = repo.index()?;
    let index_tree = index.write_tree_to(&repo)?;
    // Diff to summarize changes
    let diff =
        repo.diff_tree_to_tree(Some(&head_tree), Some(&repo.find_tree(index_tree)?), None)?;
    let stats = diff.stats()?;
    let summary = format!(
        "Changes: {} files, {} insertions(+), {} deletions(-)",
        stats.files_changed(),
        stats.insertions(),
        stats.deletions()
    );

    let mut diff_buf = String::new();

    let _ = &diff
        .print(DiffFormat::Patch, |delta, hunk, line| {
            diff_buf
                .write_str("--------------------------\n")
                .expect("Failed to write diff");
            diff_buf
                .write_fmt(format_args!("{:?}\n", delta))
                .expect("Failed to write diff");
            diff_buf
                .write_fmt(format_args!("{:?}\n", hunk))
                .expect("Failed to write diff");
            diff_buf
                .write_fmt(format_args!("{:?}\n", line))
                .expect("Failed to write diff");
            true
        })
        .expect("Failed to print diff");
    println!("{}", diff_buf);

    // Authenticate with OpenAI and generate commit message
    let api_key = env::var("OPENAI_API_KEY")?;
    openai::set_key(api_key);

    let prompt = format!(
        r#"
write a commit message for the changes I will write at the end of this message.
- NEVER WRITE MORE THAN 10 WORDS AT THE FIRST LINE.
- you must write WHY you made the changes, not WHAT you changed.
- YOU MUST NOT POINT TO SMALL DETAILS, BUT TO THE BIG PICTURE.
- ONLY RETURN COMMIT MESSAGE. DO NOT STATE WITH `Commit message:`
- priority is feature, then fix, then refactor, then style. if you feel you can not fit everything in 10 words, then you can skip where priority is low.

I will pass the diff below
```
{}
```
"#,
        diff_buf
    );

    let request = ChatCompletionBuilder::default()
        .model("gpt-4-turbo-preview")
        .messages([ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::Assistant,
            content: Some(prompt),
            name: None,
            function_call: None,
        }])
        .build()
        .expect("Failed to build chat completion");

    let response = ChatCompletion::create(&request).await?;

    let message = response
        .choices
        .iter()
        .map(|choice| choice.message.content.as_ref().expect("No content"))
        .cloned()
        .collect::<String>();

    dbg!("{}", message);

    //let commit_message = response.choices.get(0).map_or_else(
    //    || Err("No completion found".into()),
    //    |choice| Ok(choice.text.trim().to_string()),
    //)?;

    //// Commit changes
    //let sig = repo.signature()?;
    //let tree_id = index.write_tree()?;
    //let tree = repo.find_tree(tree_id)?;
    //let head = repo.head()?.peel_to_commit()?;
    //repo.commit(Some("HEAD"), &sig, &sig, &commit_message, &tree, &[&head])?;

    //println!("Committed with message: {}", commit_message);
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct OpenAiPrompt {
    prompt: String,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    text: String,
}

//async fn generate_commit_message(
//    client: &OpenAiClient,
//    summary: &str,
//) -> Result<String, Box<dyn Error>> {
//    let prompt = OpenAiPrompt {
//        prompt: format!(
//            "Summarize these changes for a git commit message: {}",
//            summary
//        ),
//        max_tokens: 60, // Adjust based on desired length
//    };
//
//    let response: OpenAiResponse = client.completions("text-davinci-003", &prompt).await?;
//    if let Some(choice) = response.choices.get(0) {
//        Ok(choice.text.trim().to_string())
//    } else {
//        Err("Failed to generate commit message".into())
//    }
//}
