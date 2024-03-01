use crate::{comment_generator, committer::Committer, confirmor::Confirmor};

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
        let comment_generator = comment_generator::CommentGenerator::new(base_message)?;
        let committer = Committer::new(confirmor, comment_generator)?;

        committer.commit().await?;

        Ok(())
    }
}
