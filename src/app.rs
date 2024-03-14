use crate::{
    comment_generator, committer::Committer, config, confirmor::Confirmor,
    prompt_translator::OpenAITranslator,
};

pub struct App {}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(App {})
    }

    pub async fn commit(
        &self,
        base_message: Option<String>,
        assume_yes: bool,
    ) -> anyhow::Result<()> {
        let confirmor = Confirmor::new(assume_yes)?;
        let prompt_translator = OpenAITranslator::new(
            config::get(config::Item::OpenaiModel)?.unwrap_or("gpt-4-turbo-preview".to_string()),
        );
        let comment_generator =
            comment_generator::CommentGenerator::new(prompt_translator, base_message);
        let committer = Committer::new(confirmor, comment_generator)?;

        committer.commit().await?;

        Ok(())
    }
}
