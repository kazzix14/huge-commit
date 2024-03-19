use crate::{
    comment_generator,
    committer::Committer,
    config::{self, ModelProvider},
    confirmor::Confirmor,
    prompt_translator::{AnthropicTranslator, OpenAITranslator, PromptTranslator},
    UserError,
};

pub struct App {}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(App {})
    }

    pub async fn commit(&self, base_message: Option<String>, assume_yes: bool) -> anyhow::Result<()> {
        let confirmor = Confirmor::new(assume_yes)?;

        let prompt_translator = match config::get_model_provider()?.ok_or(UserError::ModelProviderNotSet)? {
            ModelProvider::OpenAI => PromptTranslator::OpenAI(OpenAITranslator::new(
                config::get(config::Item::OpenaiModel)?.unwrap_or("gpt-4-turbo-preview".to_string()),
            )),
            ModelProvider::Anthropic => PromptTranslator::Anthropic(AnthropicTranslator::new(
                config::get(config::Item::AnthropicModel)?.unwrap_or("claude-3-opus-20240229".to_string()),
            )),
        };

        let comment_generator = comment_generator::CommentGenerator::new(prompt_translator, base_message);
        let committer = Committer::new(confirmor, comment_generator)?;

        committer.commit().await?;

        Ok(())
    }
}
