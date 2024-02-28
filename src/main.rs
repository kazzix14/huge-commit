use git2::{DiffFormat, IndexAddOption, Repository};
use openai::chat::{ChatCompletion, ChatCompletionBuilder, ChatCompletionMessage};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{default::Default, env};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let repo_path = ".";
    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;

    // Stage changes
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Diff to summarize changes
    let diff = repo.diff_index_to_workdir(Some(&index), None)?;
    let stats = diff.stats()?;
    let summary = format!(
        "Changes: {} files, {} insertions(+), {} deletions(-)",
        stats.files_changed(),
        stats.insertions(),
        stats.deletions()
    );

    let _ = &diff
        .print(DiffFormat::Patch, |delta, hunk, line| {
            println!("delta: {:?}", delta);
            println!("hunk: {:?}", hunk);
            println!("line: {:?}", line);
            true
        })
        .expect("Failed to print diff");
    dbg!(&stats);
    dbg!(&summary);
    panic!();

    // Authenticate with OpenAI and generate commit message
    let api_key = env::var("OPENAI_API_KEY")?;
    openai::set_key(api_key);

    let prompt = format!(
        "Summarize these changes for a git commit message: {}",
        summary
    );

    let request = ChatCompletionBuilder::default()
        .model("gpt-4")
        .messages([ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::Assistant,
            content: Some(prompt),
            name: None,
            function_call: None,
        }])
        .build()
        .expect("Failed to build chat completion");

    let response = ChatCompletion::create(&request).await?;
    dbg!(response);

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
