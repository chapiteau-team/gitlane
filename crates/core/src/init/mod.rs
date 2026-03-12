use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;
use toml::{Table, Value};

use crate::{
    errors::GitlaneError,
    fs::{FsError, ensure_directory, write_file_if_missing},
    paths::{
        ISSUES_CONFIG_FILE, ISSUES_DIR, ISSUES_LABELS_FILE, ISSUES_WORKFLOW_FILE,
        PROJECT_CONFIG_FILE,
    },
};

const ISSUES_STATE_DIRS: [&str; 4] = ["todo", "in_progress", "review", "done"];
const ISSUES_WORKFLOW_TOML: &str = include_str!("scaffold/issues/workflow.toml");
const ISSUES_CONFIG_TOML: &str = include_str!("scaffold/issues/issues.toml");
const ISSUES_LABELS_TOML: &str = include_str!("scaffold/issues/labels.toml");
const ISSUES_SCAFFOLD_FILES: [(&str, &str); 3] = [
    (ISSUES_WORKFLOW_FILE, ISSUES_WORKFLOW_TOML),
    (ISSUES_CONFIG_FILE, ISSUES_CONFIG_TOML),
    (ISSUES_LABELS_FILE, ISSUES_LABELS_TOML),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    pub name: Option<String>,
    pub default_name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

impl InitOptions {
    fn updates_project_config(&self) -> bool {
        self.name.is_some() || self.description.is_some() || self.homepage.is_some()
    }
}

#[derive(Debug, Error)]
pub enum GitlaneInitError {
    #[error("`--name` must be a non-empty string")]
    EmptyNameArgument,
    #[error("default project name must be a non-empty string")]
    EmptyDefaultName,
    #[error("failed to read `{path}`")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse `{path}` as TOML")]
    ParseToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to serialize TOML for `{path}`")]
    SerializeToml {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
    #[error(transparent)]
    Filesystem(#[from] FsError),
    #[error(transparent)]
    LoadService(#[from] GitlaneError),
}

pub(crate) fn initialize(
    project_path: impl Into<PathBuf>,
    options: InitOptions,
) -> Result<PathBuf, GitlaneInitError> {
    let project_path = project_path.into();
    ensure_directory(&project_path)?;
    initialize_issues(&project_path)?;
    ensure_project_config(&project_path, options)?;

    Ok(project_path)
}

fn initialize_issues(project_path: &Path) -> Result<(), GitlaneInitError> {
    let issues_dir = project_path.join(ISSUES_DIR);
    ensure_directory(&issues_dir)?;

    for state in ISSUES_STATE_DIRS {
        ensure_directory(&issues_dir.join(state))?;
    }

    for (file_name, content) in ISSUES_SCAFFOLD_FILES {
        write_file_if_missing(&issues_dir.join(file_name), content)?;
    }

    Ok(())
}

fn ensure_project_config(
    project_path: &Path,
    options: InitOptions,
) -> Result<(), GitlaneInitError> {
    let config_path = project_path.join(PROJECT_CONFIG_FILE);
    if config_path.exists() {
        if !config_path.is_file() {
            return Err(FsError::ExpectedFile {
                path: config_path.clone(),
            }
            .into());
        }

        if options.updates_project_config() {
            update_project_config(&config_path, options)?;
        }

        return Ok(());
    }

    let InitOptions {
        name,
        default_name,
        description,
        homepage,
    } = options;

    let name = match name {
        Some(name) => {
            validate_explicit_name(&name)?;
            name
        }
        None => {
            validate_default_name(&default_name)?;
            default_name
        }
    };

    let content = render_project_toml(&name, description.as_deref(), homepage.as_deref());
    fs::write(&config_path, content).map_err(|source| {
        FsError::WriteFile {
            path: config_path,
            source,
        }
        .into()
    })
}

fn update_project_config(config_path: &Path, options: InitOptions) -> Result<(), GitlaneInitError> {
    let InitOptions {
        name,
        default_name: _,
        description,
        homepage,
    } = options;

    let content = fs::read_to_string(config_path).map_err(|source| GitlaneInitError::ReadFile {
        path: config_path.to_path_buf(),
        source,
    })?;

    let mut table: Table = content
        .parse()
        .map_err(|source| GitlaneInitError::ParseToml {
            path: config_path.to_path_buf(),
            source,
        })?;

    let mut changed = false;
    if let Some(name) = name {
        validate_explicit_name(&name)?;
        table.insert("name".to_owned(), Value::String(name));
        changed = true;
    }

    if let Some(description) = description {
        table.insert("description".to_owned(), Value::String(description));
        changed = true;
    }

    if let Some(homepage) = homepage {
        table.insert("homepage".to_owned(), Value::String(homepage));
        changed = true;
    }

    if !changed {
        return Ok(());
    }

    let mut serialized =
        toml::to_string_pretty(&table).map_err(|source| GitlaneInitError::SerializeToml {
            path: config_path.to_path_buf(),
            source,
        })?;
    if !serialized.ends_with('\n') {
        serialized.push('\n');
    }

    fs::write(config_path, serialized).map_err(|source| {
        FsError::WriteFile {
            path: config_path.to_path_buf(),
            source,
        }
        .into()
    })
}

fn render_project_toml(name: &str, description: Option<&str>, homepage: Option<&str>) -> String {
    let mut lines = vec![format!("name = {}", Value::String(name.to_owned()))];

    if let Some(description) = description {
        lines.push(format!(
            "description = {}",
            Value::String(description.to_owned())
        ));
    }

    if let Some(homepage) = homepage {
        lines.push(format!("homepage = {}", Value::String(homepage.to_owned())));
    }

    format!("{}\n", lines.join("\n"))
}

fn validate_explicit_name(name: &str) -> Result<(), GitlaneInitError> {
    if name.trim().is_empty() {
        return Err(GitlaneInitError::EmptyNameArgument);
    }

    Ok(())
}

fn validate_default_name(name: &str) -> Result<(), GitlaneInitError> {
    if name.trim().is_empty() {
        return Err(GitlaneInitError::EmptyDefaultName);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        paths::{
            GITLANE_DIR, ISSUES_CONFIG_FILE, ISSUES_DIR, ISSUES_LABELS_FILE, ISSUES_WORKFLOW_FILE,
            PROJECT_CONFIG_FILE,
        },
        project::ProjectConfig,
    };
    use tempfile::TempDir;

    fn default_options(default_name: &str) -> InitOptions {
        InitOptions {
            name: None,
            default_name: default_name.to_owned(),
            description: None,
            homepage: None,
        }
    }

    fn issues_file_path(project_path: &Path, file_name: &str) -> PathBuf {
        project_path.join(ISSUES_DIR).join(file_name)
    }

    fn assert_issue_state_dirs_exist(project_path: &Path) {
        for state in ISSUES_STATE_DIRS {
            assert!(
                project_path.join(ISSUES_DIR).join(state).is_dir(),
                "state directory `{state}` should exist"
            );
        }
    }

    #[test]
    fn initialize_creates_full_project_layout() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = initialize(
            temp_dir.path().join(GITLANE_DIR),
            InitOptions {
                name: Some("My Project".to_owned()),
                default_name: "Ignored".to_owned(),
                description: Some("Git-native tracker".to_owned()),
                homepage: Some("https://example.com".to_owned()),
            },
        )
        .expect("init should succeed");

        assert_eq!(project_path, temp_dir.path().join(GITLANE_DIR));

        let config = ProjectConfig::load(&project_path).expect("project config should load");
        assert_eq!(config.name(), "My Project");
        assert_eq!(config.description(), Some("Git-native tracker"));
        assert_eq!(config.homepage(), Some("https://example.com"));

        assert!(issues_file_path(&project_path, ISSUES_WORKFLOW_FILE).is_file());
        assert!(issues_file_path(&project_path, ISSUES_CONFIG_FILE).is_file());
        assert!(issues_file_path(&project_path, ISSUES_LABELS_FILE).is_file());
        assert_issue_state_dirs_exist(&project_path);

        let labels_content =
            fs::read_to_string(issues_file_path(&project_path, ISSUES_LABELS_FILE))
                .expect("labels config should be readable");
        assert!(labels_content.contains("type_docs"));
    }

    #[test]
    fn initialize_uses_default_name_when_explicit_name_missing() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = initialize(
            temp_dir.path().join(GITLANE_DIR),
            default_options("Fallback"),
        )
        .expect("init should succeed");

        let config = ProjectConfig::load(&project_path).expect("project config should load");
        assert_eq!(config.name(), "Fallback");
    }

    #[test]
    fn initialize_creates_missing_artifacts_without_overwriting_existing_files() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = temp_dir.path().join(GITLANE_DIR);
        let issues_dir = gitlane_dir.join(ISSUES_DIR);
        fs::create_dir_all(&issues_dir).expect("issues directory should be created");

        let workflow_path = issues_file_path(&gitlane_dir, ISSUES_WORKFLOW_FILE);
        let custom_workflow =
            "initial_state = \"custom\"\n[states]\ncustom = { name = \"Custom\" }\n";
        fs::write(&workflow_path, custom_workflow).expect("workflow config should be written");

        fs::write(
            gitlane_dir.join(PROJECT_CONFIG_FILE),
            "name = \"Existing\"\ncustom = \"keep\"\n",
        )
        .expect("project config should be written");

        let project_path = initialize(gitlane_dir.clone(), default_options("Unused"))
            .expect("init should succeed");

        let config = ProjectConfig::load(&project_path).expect("project config should load");
        assert_eq!(config.name(), "Existing");
        assert_eq!(
            fs::read_to_string(&workflow_path).expect("workflow config should be readable"),
            custom_workflow
        );
        assert!(issues_file_path(&project_path, ISSUES_CONFIG_FILE).is_file());
        assert!(issues_file_path(&project_path, ISSUES_LABELS_FILE).is_file());
        assert_issue_state_dirs_exist(&project_path);

        let project_content = fs::read_to_string(project_path.join(PROJECT_CONFIG_FILE))
            .expect("project config should be readable");
        assert!(project_content.contains("custom = \"keep\""));
    }

    #[test]
    fn initialize_updates_existing_project_metadata_fields() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path().join(GITLANE_DIR);
        fs::create_dir_all(&project_path).expect("project directory should be created");
        fs::write(
            project_path.join(PROJECT_CONFIG_FILE),
            "name = \"Existing\"\ncustom = \"keep\"\n",
        )
        .expect("project config should be written");

        let project_path = initialize(
            project_path,
            InitOptions {
                name: Some("Renamed".to_owned()),
                default_name: "Ignored".to_owned(),
                description: Some("Updated description".to_owned()),
                homepage: Some("https://example.com/project".to_owned()),
            },
        )
        .expect("init should succeed");

        let config = ProjectConfig::load(&project_path).expect("project config should load");
        assert_eq!(config.name(), "Renamed");
        assert_eq!(config.description(), Some("Updated description"));
        assert_eq!(config.homepage(), Some("https://example.com/project"));

        let project_content = fs::read_to_string(project_path.join(PROJECT_CONFIG_FILE))
            .expect("project config should be readable");
        assert!(project_content.contains("custom = \"keep\""));
    }

    #[test]
    fn initialize_rejects_empty_name_argument() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let err = initialize(
            temp_dir.path().join(GITLANE_DIR),
            InitOptions {
                name: Some("   ".to_owned()),
                default_name: "fallback".to_owned(),
                description: None,
                homepage: None,
            },
        )
        .expect_err("empty name should fail");

        assert!(matches!(err, GitlaneInitError::EmptyNameArgument));
    }

    #[test]
    fn initialize_rejects_empty_default_name_when_name_is_missing() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let err = initialize(temp_dir.path().join(GITLANE_DIR), default_options("   "))
            .expect_err("empty default name should fail");

        assert!(matches!(err, GitlaneInitError::EmptyDefaultName));
    }

    #[test]
    fn initialize_supports_non_gitlane_project_path() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path().join("custom-project-data");

        let initialized_path =
            initialize(project_path.clone(), default_options("demo")).expect("init should succeed");

        assert_eq!(initialized_path, project_path);
        assert!(initialized_path.join(PROJECT_CONFIG_FILE).is_file());
        assert!(issues_file_path(&initialized_path, ISSUES_WORKFLOW_FILE).is_file());
    }

    #[test]
    fn initialize_creates_missing_parent_directories() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let missing_parent = temp_dir.path().join("missing-parent");
        let project_path = missing_parent.join("project-data");

        let initialized_path = initialize(project_path.clone(), default_options("demo"))
            .expect("init should create missing parent directories");

        assert_eq!(initialized_path, project_path);
        assert!(initialized_path.join(PROJECT_CONFIG_FILE).is_file());
    }
}
