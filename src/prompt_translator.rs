use openai::chat::{
    ChatCompletionChoiceDelta, ChatCompletionDelta, ChatCompletionGeneric, ChatCompletionMessage,
};
use tokio::sync::mpsc::Receiver;

use crate::config;

pub struct PromptTranslator {
    model: String,
}

impl PromptTranslator {
    pub fn new(model: String) -> Self {
        Self { model }
    }

    pub async fn translate(
        &self,
        prompt: String,
    ) -> anyhow::Result<Receiver<ChatCompletionGeneric<ChatCompletionChoiceDelta>>> {
        let api_key = config::get(config::Item::OpenaiApiKey)?.expect("openai-api-key not set");

        openai::set_key(api_key);

        Ok(ChatCompletionDelta::builder(
            &self.model,
            [ChatCompletionMessage {
                role: openai::chat::ChatCompletionMessageRole::Assistant,
                content: Some(prompt),
                name: None,
                function_call: None,
            }],
        )
        .create_stream()
        .await?)
    }
}
