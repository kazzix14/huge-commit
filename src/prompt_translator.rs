use futures::Stream;
use futures::StreamExt;
use openai::chat::{
    ChatCompletionChoiceDelta, ChatCompletionDelta, ChatCompletionGeneric, ChatCompletionMessage,
};

use crate::config;

pub trait PromptTranslator {
    async fn translate(&self, prompt: String) -> anyhow::Result<impl Stream<Item = String>>;
}

pub struct OpenAITranslator {
    model: String,
}

impl OpenAITranslator {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl PromptTranslator for OpenAITranslator {
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
        //Ok(stream)
    }
}

// while let Some(response) = response_rx.recv().await {
//     response.choices.iter().for_each(|choice| {
//         if let Some(content) = &choice.delta.content {
//             commit_message.push_str(content);
//             print!("{}", content);
//             std::io::Write::flush(&mut std::io::stdout()).unwrap();
//         }
//     });
// }
