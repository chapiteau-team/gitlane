use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Gitlane {
    repo_root: PathBuf,
}

impl Gitlane {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }
}
