use crate::{
    comment_generator,
    committer::Committer,
    config,
    confirmor::Confirmor,
    prompt_translator::{ClaudeTranslator, OpenAITranslator, PromptTranslator},
};

pub struct App {}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(App {})
    }

    pub async fn commit(&self, base_message: Option<String>, assume_yes: bool) -> anyhow::Result<()> {
        let confirmor = Confirmor::new(assume_yes)?;

        // FIXME: This is a bit of a mess. We should probably have a separate
        let model_provider = config::get(config::Item::ModelProvider)?;
        match model_provider {
            Some(model_provider) => {
                if model_provider == "OpenAI" {
                    let openai_api_key = config::get(config::Item::OpenaiApiKey)?;
                    if openai_api_key.is_none() {
                        anyhow::bail!("OpenAI API key is required");
                    }

                    let prompt_translator = OpenAITranslator::new(
                        config::get(config::Item::OpenaiModel)?.unwrap_or("gpt-4-turbo-preview".to_string()),
                    );

                    let comment_generator = comment_generator::CommentGenerator::new(PromptTranslator::OpenAI(prompt_translator), base_message);
                    let committer = Committer::new(confirmor, comment_generator)?;

                    committer.commit().await?;
                } else if model_provider == "Anthropic" {
                    let anthropic_api_key = config::get(config::Item::AnthropicApiKey)?;
                    if anthropic_api_key.is_none() {
                        anyhow::bail!("Anthropic API key is required");
                    }

                    let prompt_translator = ClaudeTranslator::new(
                        config::get(config::Item::AnthropicModel)?.unwrap_or("claude-3-opus-20240229".to_string()),
                    );

                    let comment_generator = comment_generator::CommentGenerator::new(PromptTranslator::Claude(prompt_translator), base_message);
                    let committer = Committer::new(confirmor, comment_generator)?;

                    committer.commit().await?;
                } else {
                    anyhow::bail!("Invalid model provider: {}", model_provider);
                }
            }
            None => {
                anyhow::bail!("Model provider is required");
            }
        }

        Ok(())
    }
}
