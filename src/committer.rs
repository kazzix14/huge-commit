use crate::comment_generator::CommentGenerator;
use crate::confirmor::Confirmor;

use git2::Repository;

pub struct Committer {
    repository: git2::Repository,
    confirmor: Confirmor,
    comment_generator: CommentGenerator,
}

impl Committer {
    pub fn new(confirmor: Confirmor, comment_generator: CommentGenerator) -> anyhow::Result<Self> {
        Ok(Committer {
            repository: Repository::open(".")?,
            confirmor,
            comment_generator,
        })
    }

    pub async fn commit(&self, base_message: Option<String>) -> anyhow::Result<()> {
        let diff = self.get_diff()?;

        if !self.diff_has_change(&diff)? {
            self.stage_all_files()?;
        }

        let diff = self.get_diff()?;
        if !self.diff_has_change(&diff)? {
            Err(crate::UserError::NoChangesToCommit.into())
        } else {
            let commit_message = self
                .comment_generator
                .gen_commit_message(base_message, &diff)
                .await?;

            self.commit_changes(&commit_message)?;
            Ok(())
        }
    }

    fn get_diff(&self) -> anyhow::Result<git2::Diff<'_>> {
        let index = self.repository.index()?;
        let head_commit = self.repository.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        let diff = self
            .repository
            .diff_tree_to_index(Some(&head_tree), Some(&index), None)?;

        Ok(diff)
    }

    fn diff_has_change(&self, diff: &git2::Diff) -> anyhow::Result<bool> {
        Ok(0 < diff.stats()?.files_changed())
    }

    fn stage_all_files(&self) -> anyhow::Result<()> {
        let mut index = self.repository.index()?;

        let stage = self
            .confirmor
            .confirm("No changes to commit. stage all changes?", true);

        println!();

        if stage {
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .expect("Failed to stage changes");
            index.write().expect("Failed to write index");
            Ok(())
        } else {
            Err(crate::UserError::NoChangesToCommit.into())
        }
    }

    fn commit_changes(&self, commit_message: &str) -> anyhow::Result<()> {
        let mut index = self.repository.index()?;

        let commit = self.confirmor.confirm("commit with this message?", true);

        if commit {
            let sig = self.repository.signature()?;
            let tree_id = index.write_tree()?;
            let tree = self.repository.find_tree(tree_id)?;
            let head = self.repository.head()?.peel_to_commit()?;
            self.repository
                .commit(Some("HEAD"), &sig, &sig, commit_message, &tree, &[&head])?;
        };

        Ok(())
    }
}
