use std::path::Path;

use toml::{Table, Value};

use crate::{
    errors::GitlaneError,
    fs::{ensure_directory, ensure_file, read_text_file, write_file_if_missing, write_text_file},
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

pub(crate) fn initialize(project_path: &Path, options: InitOptions) -> Result<(), GitlaneError> {
    ensure_directory(project_path)?;
    initialize_issues(project_path)?;
    ensure_project_config(project_path, options)?;

    Ok(())
}

fn initialize_issues(project_path: &Path) -> Result<(), GitlaneError> {
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

fn ensure_project_config(project_path: &Path, options: InitOptions) -> Result<(), GitlaneError> {
    let config_path = project_path.join(PROJECT_CONFIG_FILE);
    if config_path.exists() {
        ensure_file(&config_path)?;

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

    let name = name.unwrap_or(default_name);
    validate_project_name(&name)?;

    let content = render_project_toml(&name, description.as_deref(), homepage.as_deref());
    write_text_file(&config_path, &content)?;
    Ok(())
}

fn update_project_config(config_path: &Path, options: InitOptions) -> Result<(), GitlaneError> {
    let InitOptions {
        name,
        default_name: _,
        description,
        homepage,
    } = options;

    let content = read_text_file(config_path)?;

    let mut table: Table = content.parse().map_err(|source| GitlaneError::ParseToml {
        path: config_path.to_path_buf(),
        source,
    })?;

    if let Some(name) = name.as_deref() {
        validate_project_name(name)?;
    }

    let mut changed = insert_optional_string_field(&mut table, "name", name);
    changed |= insert_optional_string_field(&mut table, "description", description);
    changed |= insert_optional_string_field(&mut table, "homepage", homepage);

    if !changed {
        return Ok(());
    }

    let mut serialized =
        toml::to_string_pretty(&table).map_err(|source| GitlaneError::SerializeToml {
            path: config_path.to_path_buf(),
            source,
        })?;
    if !serialized.ends_with('\n') {
        serialized.push('\n');
    }

    write_text_file(config_path, &serialized)?;
    Ok(())
}

fn insert_optional_string_field(table: &mut Table, key: &str, value: Option<String>) -> bool {
    if let Some(value) = value {
        table.insert(key.to_owned(), Value::String(value));
        return true;
    }

    false
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

fn validate_project_name(name: &str) -> Result<(), GitlaneError> {
    if name.trim().is_empty() {
        return Err(GitlaneError::InvalidProjectName);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::PathBuf};

    use crate::{
        paths::{
            ISSUES_CONFIG_FILE, ISSUES_DIR, ISSUES_LABELS_FILE, ISSUES_WORKFLOW_FILE,
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
        let project_path = temp_dir.path();
        initialize(
            project_path,
            InitOptions {
                name: Some("My Project".to_owned()),
                default_name: "Ignored".to_owned(),
                description: Some("Git-native tracker".to_owned()),
                homepage: Some("https://example.com".to_owned()),
            },
        )
        .expect("init should succeed");

        let config = ProjectConfig::load(project_path).expect("project config should load");
        assert_eq!(config.name(), "My Project");
        assert_eq!(config.description(), Some("Git-native tracker"));
        assert_eq!(config.homepage(), Some("https://example.com"));

        assert!(issues_file_path(project_path, ISSUES_WORKFLOW_FILE).is_file());
        assert!(issues_file_path(project_path, ISSUES_CONFIG_FILE).is_file());
        assert!(issues_file_path(project_path, ISSUES_LABELS_FILE).is_file());
        assert_issue_state_dirs_exist(project_path);

        let labels_content = fs::read_to_string(issues_file_path(project_path, ISSUES_LABELS_FILE))
            .expect("labels config should be readable");
        assert!(labels_content.contains("type_docs"));
    }

    #[test]
    fn initialize_uses_default_name_when_explicit_name_missing() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path();
        initialize(project_path, default_options("Fallback")).expect("init should succeed");

        let config = ProjectConfig::load(project_path).expect("project config should load");
        assert_eq!(config.name(), "Fallback");
    }

    #[test]
    fn initialize_creates_missing_artifacts_without_overwriting_existing_files() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let existing_project_path = temp_dir.path();
        let issues_dir = existing_project_path.join(ISSUES_DIR);
        fs::create_dir_all(&issues_dir).expect("issues directory should be created");

        let workflow_path = issues_file_path(existing_project_path, ISSUES_WORKFLOW_FILE);
        let custom_workflow =
            "initial_state = \"custom\"\n[states]\ncustom = { name = \"Custom\" }\n";
        fs::write(&workflow_path, custom_workflow).expect("workflow config should be written");

        fs::write(
            existing_project_path.join(PROJECT_CONFIG_FILE),
            "name = \"Existing\"\ncustom = \"keep\"\n",
        )
        .expect("project config should be written");

        initialize(existing_project_path, default_options("Unused")).expect("init should succeed");

        let config =
            ProjectConfig::load(existing_project_path).expect("project config should load");
        assert_eq!(config.name(), "Existing");
        assert_eq!(
            fs::read_to_string(&workflow_path).expect("workflow config should be readable"),
            custom_workflow
        );
        assert!(issues_file_path(existing_project_path, ISSUES_CONFIG_FILE).is_file());
        assert!(issues_file_path(existing_project_path, ISSUES_LABELS_FILE).is_file());
        assert_issue_state_dirs_exist(existing_project_path);

        let project_content = fs::read_to_string(existing_project_path.join(PROJECT_CONFIG_FILE))
            .expect("project config should be readable");
        assert!(project_content.contains("custom = \"keep\""));
    }

    #[test]
    fn initialize_updates_existing_project_metadata_fields() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path();
        fs::write(
            project_path.join(PROJECT_CONFIG_FILE),
            "name = \"Existing\"\ncustom = \"keep\"\n",
        )
        .expect("project config should be written");

        initialize(
            project_path,
            InitOptions {
                name: Some("Renamed".to_owned()),
                default_name: "Ignored".to_owned(),
                description: Some("Updated description".to_owned()),
                homepage: Some("https://example.com/project".to_owned()),
            },
        )
        .expect("init should succeed");

        let config = ProjectConfig::load(project_path).expect("project config should load");
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
            temp_dir.path(),
            InitOptions {
                name: Some("   ".to_owned()),
                default_name: "fallback".to_owned(),
                description: None,
                homepage: None,
            },
        )
        .expect_err("empty name should fail");

        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }

    #[test]
    fn initialize_rejects_empty_default_name_when_name_is_missing() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let err = initialize(temp_dir.path(), default_options("   "))
            .expect_err("empty default name should fail");

        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }

    #[test]
    fn initialize_supports_non_gitlane_project_path() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path().join("custom-project-data");

        initialize(&project_path, default_options("demo")).expect("init should succeed");

        assert!(project_path.join(PROJECT_CONFIG_FILE).is_file());
        assert!(issues_file_path(&project_path, ISSUES_WORKFLOW_FILE).is_file());
    }

    #[test]
    fn initialize_creates_missing_parent_directories() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let missing_parent = temp_dir.path().join("missing-parent");
        let project_path = missing_parent.join("project-data");

        initialize(&project_path, default_options("demo"))
            .expect("init should create missing parent directories");

        assert!(project_path.join(PROJECT_CONFIG_FILE).is_file());
    }
}
