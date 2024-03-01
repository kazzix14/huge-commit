use openai::chat::{
    ChatCompletionChoiceDelta, ChatCompletionDelta, ChatCompletionGeneric, ChatCompletionMessage,
};
use tokio::sync::mpsc::Receiver;

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
