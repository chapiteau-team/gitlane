use std::path::{Path, PathBuf};

use crate::{
    errors::GitlaneError,
    init::{self, GitlaneInitError, InitOptions},
    project::ProjectConfig,
};

#[derive(Debug, Clone)]
pub struct Gitlane {
    project_path: PathBuf,
    project_config: ProjectConfig,
}

impl Gitlane {
    pub fn new(project_path: impl Into<PathBuf>) -> Result<Self, GitlaneError> {
        let project_path = project_path.into();
        let project_config = ProjectConfig::load(&project_path)?;

        Ok(Self {
            project_path,
            project_config,
        })
    }

    pub fn init(
        project_path: impl Into<PathBuf>,
        options: InitOptions,
    ) -> Result<Self, GitlaneInitError> {
        let project_path = init::initialize(project_path, options)?;
        Ok(Self::new(project_path)?)
    }

    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    pub fn project_config(&self) -> &ProjectConfig {
        &self.project_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use tempfile::TempDir;

    use crate::{
        paths::{GITLANE_DIR, PROJECT_CONFIG_FILE},
        project::ProjectConfigError,
    };

    fn create_project_dir(base: &Path, config: &str) -> PathBuf {
        let gitlane_dir = base.join(GITLANE_DIR);
        fs::create_dir_all(&gitlane_dir).expect(".gitlane directory should be created");
        fs::write(gitlane_dir.join(PROJECT_CONFIG_FILE), config)
            .expect("project config should be created");
        gitlane_dir
    }

    #[test]
    fn loads_project_config_on_new() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_dir = create_project_dir(
            temp_dir.path(),
            r#"
name = "Gitlane"
people = ["@alice", "@bob"]
"#,
        );

        let service = Gitlane::new(project_dir.clone()).expect("service should initialize");

        assert_eq!(service.project_path(), project_dir);
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

        let err = Gitlane::new(project_dir).expect_err("invalid config should fail");
        assert!(matches!(
            err,
            GitlaneError::ProjectConfig(ProjectConfigError::EmptyName)
        ));
    }

    #[test]
    fn init_checks_initialized_project_can_be_loaded() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let _gitlane_dir = create_project_dir(
            temp_dir.path(),
            r#"
name = ""
"#,
        );

        let err = Gitlane::init(
            temp_dir.path().join(GITLANE_DIR),
            InitOptions {
                name: None,
                default_name: "ignored".to_owned(),
                description: None,
                homepage: None,
            },
        )
        .expect_err("invalid existing project config should fail");

        assert!(matches!(
            err,
            GitlaneInitError::LoadService(GitlaneError::ProjectConfig(
                ProjectConfigError::EmptyName
            ))
        ));
    }
}
