use std::pin::Pin;

use futures::Stream;
use futures::StreamExt;
use openai::chat::{ChatCompletionChoiceDelta, ChatCompletionDelta, ChatCompletionGeneric, ChatCompletionMessage};

use crate::config;

pub enum PromptTranslator {
    OpenAI(OpenAITranslator),
    Claude(ClaudeTranslator),
}

impl PromptTranslator {
    pub async fn translate(&self, prompt: String) -> anyhow::Result<Pin<Box<dyn Stream<Item = String>>>> {
        match self {
            Self::OpenAI(translator) => Ok(Box::pin(translator.translate(prompt).await?)),
            Self::Claude(translator) => Ok(Box::pin(translator.translate(prompt).await?)),
        }
    }
}

pub struct OpenAITranslator {
    model: String,
}

impl OpenAITranslator {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl OpenAITranslator {
    async fn translate(&self, prompt: String) -> anyhow::Result<impl Stream<Item = String>> {
        let api_key = config::get(config::Item::OpenaiApiKey)?.expect("openai-api-key not set");

        openai::set_key(api_key);

        let translated = ChatCompletionDelta::builder(
            &self.model,
            [ChatCompletionMessage {
                role: openai::chat::ChatCompletionMessageRole::Assistant,
                content: Some(prompt),
                name: None,
                function_call: None,
            }],
        )
        .create_stream()
        .await?;

        let stream = tokio_stream::wrappers::ReceiverStream::new(translated);
        let stream = stream.map(|data: ChatCompletionGeneric<ChatCompletionChoiceDelta>| {
            data.choices
                .iter()
                .map(|c| c.delta.clone().content)
                .filter(|c| c.is_some())
                .map(|f| f.unwrap())
                .collect::<Vec<String>>()
                .join(" ")
        });

        Ok(stream)
    }
}

pub struct ClaudeTranslator {
    model: String,
}

impl ClaudeTranslator {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl ClaudeTranslator {
    async fn translate(&self, prompt: String) -> anyhow::Result<impl Stream<Item = String>> {
        let api_key = config::get(config::Item::AnthropicApiKey)?.expect("anthropic-api-key not set");
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("X-API-Key", api_key)
            .json(&serde_json::json!({
                "messages": serde_json::json!([{"role": "user", "content": prompt}]),
                "model": self.model,
                "stream": true,
                "max_tokens": 400,
                "temperature": 0.7,
            }))
            .send()
            .await?;

        let stream = response
            .bytes_stream()
            .map(|chunk| {
                chunk
                    .map(|bytes| String::from_utf8_lossy(&bytes).trim().to_string())
                    .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            })
            .filter_map(|result| async move { result.ok() })
            .filter(|data| futures::future::ready(!data.is_empty()))
            .map(|data| {
                //println!("{}", &data);
                let lines: Vec<&str> = data.split('\n').collect();
                let json_data = lines
                    .iter()
                    .find(|line| line.find("{").is_some())
                    .and_then(|line| serde_json::from_str::<serde_json::Value>(line.trim_start_matches("data: ")).ok());

                json_data
                    //.and_then(|value| value.get("data").cloned())
                    //.and_then(|data| serde_json::from_value::<serde_json::Value>(data).ok())
                    .and_then(|value| value.get("delta").cloned())
                    .and_then(|delta| delta.get("text").cloned())
                    .and_then(|text| text.as_str().map(|s| s.to_string()))
                    .unwrap_or_default()
            });

        Ok(stream)
    }
}
