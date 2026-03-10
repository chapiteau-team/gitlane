use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Gitlane {
    project_path: PathBuf,
}

impl Gitlane {
    pub fn new(project_path: impl Into<PathBuf>) -> Self {
        Self {
            project_path: project_path.into(),
        }
    }

    pub fn project_path(&self) -> &Path {
        &self.project_path
    }
}
