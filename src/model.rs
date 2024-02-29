use crate::config;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
}

pub async fn list() -> anyhow::Result<Vec<Model>> {
    let api_key = config::get(config::Item::OpenaiApiKey)?.expect("OpenAI API key not set");

    let response = reqwest::Client::new()
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await?;

    let response = serde_json::de::from_str::<ModelListResponse>(&response.text().await?)?;

    Ok(response.data)
}

#[derive(serde::Deserialize, Debug)]
pub struct ModelListResponse {
    pub data: Vec<Model>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Model {
    pub created: Option<i64>,
    pub id: String,
    pub object: String,
    pub owned_by: String,
}
