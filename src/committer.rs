use crate::comment_generator::CommentGenerator;
use crate::confirmor::Confirmor;
use crate::prompt_translator::PromptTranslator;

use std::io::Read;
use std::path::Path;
use std::{fmt::Write, io::BufRead};

use git2::{DiffFormat, DiffOptions, Repository};

pub struct Committer<T: PromptTranslator> {
    repository: git2::Repository,
    confirmor: Confirmor,
    comment_generator: CommentGenerator<T>,
}

impl<T: PromptTranslator> Committer<T> {
    pub fn new(
        confirmor: Confirmor,
        comment_generator: CommentGenerator<T>,
    ) -> anyhow::Result<Self> {
        while let Err(err) = Repository::open(".") {
            if let Some(parent) = Path::new("..").canonicalize().ok() {
                std::env::set_current_dir(parent)?;
            } else {
                return Err(err.into());
            }
        }
        let repository = Repository::open(".")?;

        Ok(Committer {
            repository,
            confirmor,
            comment_generator,
        })
    }

    pub async fn commit(&self) -> anyhow::Result<()> {
        let diff = self.get_diff()?;

        if !self.diff_has_change(&diff)? {
            self.stage_all_files()?;
        }

        let diff = self.get_diff()?;
        if !self.diff_has_change(&diff)? {
            Err(crate::UserError::NoChangesToCommit.into())
        } else {
            let diff_str = Self::stringify_diff(&diff)?;
            let commit_message = self.comment_generator.gen_commit_message(diff_str).await?;

            self.commit_changes(&commit_message)?;
            Ok(())
        }
    }

    fn get_diff(&self) -> anyhow::Result<git2::Diff<'_>> {
        let index = self.repository.index()?;
        let head_commit = self.repository.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let mut opts = DiffOptions::new();
        let ignore_patterns = Self::read_custom_ignore_patterns(".hcignore")?;
        for pattern in ignore_patterns {
            opts.pathspec(pattern);
        }

        let diff =
            self.repository
                .diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut opts))?;

        Ok(diff)
    }

    fn read_custom_ignore_patterns(path: &str) -> anyhow::Result<Vec<String>> {
        let path = std::path::Path::new(path);
        let file = std::fs::File::open(path);

        Ok(file
            .and_then(|file| {
                let lines = std::io::BufReader::new(file).lines();
                let mut patterns = Vec::new();

                for line in lines {
                    let line = line?;
                    if line.trim().is_empty() || line.starts_with('#') {
                        continue;
                    }
                    patterns.push(line);
                }

                Ok(patterns)
            })
            .unwrap_or(Vec::new()))
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

    fn stringify_diff(diff: &git2::Diff) -> anyhow::Result<String> {
        let mut diff_buf = String::new();

        let _ = &diff
            .print(DiffFormat::Patch, |_delta, _hunk, line| {
                let mut buf = String::new();

                line.content()
                    .read_to_string(&mut buf)
                    .expect("Failed to read line");

                diff_buf
                    .write_fmt(format_args!("{} {}", line.origin(), buf))
                    .expect("Failed to write diff");

                true
            })
            .expect("Failed to print diff");

        Ok(diff_buf)
    }
}
