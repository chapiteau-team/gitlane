use std::path::{Path, PathBuf};

use crate::{
    errors::GitlaneError,
    init::{self, InitOptions},
    project::ProjectConfig,
};

/// Core service for interacting with project metadata and lifecycle.
///
/// A `Gitlane` instance stores the project directory path and validated
/// configuration loaded from `project.toml`.
#[derive(Debug, Clone)]
pub struct Gitlane {
    project_path: PathBuf,
    project_config: ProjectConfig,
}

impl Gitlane {
    /// Load an existing project from `project_path`.
    ///
    /// This reads and validates `project.toml` in the provided directory.
    pub fn load(project_path: PathBuf) -> Result<Self, GitlaneError> {
        let project_config = ProjectConfig::load(&project_path)?;

        Ok(Self {
            project_path,
            project_config,
        })
    }

    /// Initialize a project at `project_path`, then load it.
    ///
    /// See [`InitOptions`] for initialization behavior and metadata
    /// updates.
    pub fn init(project_path: PathBuf, options: InitOptions) -> Result<Self, GitlaneError> {
        init::initialize(&project_path, options)?;
        Self::load(project_path)
    }

    /// Return the project directory used by this service instance.
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// Return validated project metadata loaded for this instance.
    pub fn project_config(&self) -> &ProjectConfig {
        &self.project_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use tempfile::TempDir;

    use crate::paths::PROJECT_CONFIG_FILE;

    fn create_project_dir(project_dir: &Path, config: &str) -> PathBuf {
        fs::create_dir_all(project_dir).expect("project directory should be created");
        fs::write(project_dir.join(PROJECT_CONFIG_FILE), config)
            .expect("project config should be created");
        project_dir.to_path_buf()
    }

    #[test]
    fn loads_project_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_dir = create_project_dir(
            temp_dir.path(),
            r#"
name = "Gitlane"
people = ["@alice", "@bob"]
"#,
        );

        let service = Gitlane::load(project_dir).expect("service should initialize");

        assert_eq!(service.project_path(), temp_dir.path());
        assert_eq!(service.project_config().name(), "Gitlane");
        assert_eq!(
            service.project_config().people(),
            &["@alice".to_string(), "@bob".to_string()]
        );
    }

    #[test]
    fn errors_when_project_config_is_invalid() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_dir = create_project_dir(
            temp_dir.path(),
            r#"
name = ""
"#,
        );

        let err = Gitlane::load(project_dir).expect_err("invalid config should fail");
        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }

    #[test]
    fn init_checks_initialized_project_can_be_loaded() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_dir = create_project_dir(
            temp_dir.path(),
            r#"
name = ""
"#,
        );

        let err = Gitlane::init(
            project_dir,
            InitOptions {
                name: None,
                default_name: "ignored".to_owned(),
                description: None,
                homepage: None,
            },
        )
        .expect_err("invalid existing project config should fail");

        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }
}
