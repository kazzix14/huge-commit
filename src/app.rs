use git2::Repository;

use crate::{committer::Committer, confirmor::Confirmor};

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
        let confirmor = Confirmor::new()?;
        let committer = Committer::new(confirmor)?;

        committer.commit(base_message, assume_yes).await?;

        Ok(())
    }
}
