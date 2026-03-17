//! Project initialization routines.
//!
//! Initialization ensures the target directory exists, scaffolds issue
//! workflow/config/label files, and creates or updates `project.toml`.

use std::path::Path;
use toml::{Table, Value};

use crate::{
    errors::GitlaneError,
    fs::{ensure_directory, ensure_file, read_text_file, write_file_if_missing, write_text_file},
    paths::{
        ISSUES_CONFIG_FILE, ISSUES_DIR, ISSUES_LABELS_FILE, ISSUES_WORKFLOW_FILE,
        PROJECT_CONFIG_FILE,
    },
    workflow::WorkflowConfig,
};

const ISSUES_WORKFLOW_TOML: &str = include_str!("scaffold/issues/workflow.toml");
const ISSUES_CONFIG_TOML: &str = include_str!("scaffold/issues/issues.toml");
const ISSUES_LABELS_TOML: &str = include_str!("scaffold/issues/labels.toml");
const ISSUES_SCAFFOLD_FILES: [(&str, &str); 3] = [
    (ISSUES_WORKFLOW_FILE, ISSUES_WORKFLOW_TOML),
    (ISSUES_CONFIG_FILE, ISSUES_CONFIG_TOML),
    (ISSUES_LABELS_FILE, ISSUES_LABELS_TOML),
];

/// Options that control project initialization and metadata updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    /// Explicit project name.
    ///
    /// When provided, this is used for new configs and updates existing
    /// configs.
    name: Option<String>,
    /// Fallback name used only when creating a new config and `name` is not
    /// provided.
    default_name: String,
    /// Optional project description to set on create or update.
    description: Option<String>,
    /// Optional homepage URL string to set on create or update.
    homepage: Option<String>,
}

impl InitOptions {
    /// Build validated initialization options.
    ///
    /// Both `default_name` and `name` (when provided) must be non-empty and
    /// not whitespace-only.
    pub fn new(
        name: Option<String>,
        default_name: String,
        description: Option<String>,
        homepage: Option<String>,
    ) -> Result<Self, GitlaneError> {
        Self::validate_project_name(&default_name)?;
        if let Some(name) = name.as_deref() {
            Self::validate_project_name(name)?;
        }

        Ok(Self {
            name,
            default_name,
            description,
            homepage,
        })
    }

    /// Return whether these options request metadata updates.
    fn has_project_metadata_updates(&self) -> bool {
        self.name.is_some() || self.description.is_some() || self.homepage.is_some()
    }

    /// Return the name to use when creating a new project config file.
    fn project_name(&self) -> &str {
        self.name.as_deref().unwrap_or(self.default_name.as_str())
    }

    fn validate_project_name(name: &str) -> Result<(), GitlaneError> {
        if name.trim().is_empty() {
            return Err(GitlaneError::InvalidProjectName);
        }

        Ok(())
    }
}

/// Initialize project artifacts at `project_path`.
///
/// This creates missing directories, scaffolds issue files, and ensures a
/// valid `project.toml` exists.
pub(crate) fn initialize(project_path: &Path, options: InitOptions) -> Result<(), GitlaneError> {
    ensure_project_root(project_path)?;
    ensure_issues_layout(project_path)?;
    sync_project_config(project_path, &options)?;

    Ok(())
}

/// Ensure the project root directory exists.
fn ensure_project_root(project_path: &Path) -> Result<(), GitlaneError> {
    ensure_directory(project_path)?;
    Ok(())
}

/// Ensure issue directories and default scaffold files exist.
fn ensure_issues_layout(project_path: &Path) -> Result<(), GitlaneError> {
    let issues_dir = project_path.join(ISSUES_DIR);
    ensure_directory(&issues_dir)?;

    ensure_issue_scaffold_files(&issues_dir)?;
    ensure_issue_state_dirs(&issues_dir)?;

    Ok(())
}

/// Ensure all workflow state directories exist under `issues_dir`.
fn ensure_issue_state_dirs(issues_dir: &Path) -> Result<(), GitlaneError> {
    let workflow_path = issues_dir.join(ISSUES_WORKFLOW_FILE);
    let workflow = WorkflowConfig::load_from_path(&workflow_path)?;

    for state in workflow.state_ids() {
        ensure_directory(&issues_dir.join(state))?;
    }

    Ok(())
}

/// Ensure default issue scaffold files exist under `issues_dir`.
fn ensure_issue_scaffold_files(issues_dir: &Path) -> Result<(), GitlaneError> {
    for (file_name, content) in ISSUES_SCAFFOLD_FILES {
        write_file_if_missing(&issues_dir.join(file_name), content)?;
    }

    Ok(())
}

/// Ensure `project.toml` exists, creating or updating it as needed.
fn sync_project_config(project_path: &Path, options: &InitOptions) -> Result<(), GitlaneError> {
    let config_path = project_path.join(PROJECT_CONFIG_FILE);
    if config_path.exists() {
        ensure_file(&config_path)?;

        if options.has_project_metadata_updates() {
            update_project_config(&config_path, options)?;
        }

        return Ok(());
    }

    create_project_config(&config_path, options)
}

/// Create `project.toml` from initialization options.
fn create_project_config(config_path: &Path, options: &InitOptions) -> Result<(), GitlaneError> {
    let content = render_project_toml(
        options.project_name(),
        options.description.as_deref(),
        options.homepage.as_deref(),
    );
    write_text_file(config_path, &content)?;
    Ok(())
}

/// Apply metadata updates to an existing `project.toml`.
fn update_project_config(config_path: &Path, options: &InitOptions) -> Result<(), GitlaneError> {
    let content = read_text_file(config_path)?;

    let mut table: Table = content.parse().map_err(|source| GitlaneError::ParseToml {
        path: config_path.to_path_buf(),
        source,
    })?;

    let changed = apply_project_metadata_updates(&mut table, options);

    if !changed {
        return Ok(());
    }

    write_project_config(config_path, &table)?;
    Ok(())
}

/// Apply project metadata updates and return whether any field changed.
fn apply_project_metadata_updates(table: &mut Table, options: &InitOptions) -> bool {
    let mut changed = set_optional_string_field_if_changed(table, "name", options.name.as_deref());
    changed |=
        set_optional_string_field_if_changed(table, "description", options.description.as_deref());
    changed |= set_optional_string_field_if_changed(table, "homepage", options.homepage.as_deref());

    changed
}

/// Serialize and write `project.toml` with a trailing newline.
fn write_project_config(config_path: &Path, table: &Table) -> Result<(), GitlaneError> {
    let mut serialized =
        toml::to_string_pretty(table).map_err(|source| GitlaneError::SerializeToml {
            path: config_path.to_path_buf(),
            source,
        })?;
    if !serialized.ends_with('\n') {
        serialized.push('\n');
    }

    write_text_file(config_path, &serialized)?;
    Ok(())
}

/// Insert a string field only when `value` is present and different.
///
/// Returns `true` when the table was modified.
fn set_optional_string_field_if_changed(table: &mut Table, key: &str, value: Option<&str>) -> bool {
    if let Some(value) = value {
        if table
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(|current| current == value)
        {
            return false;
        }

        table.insert(key.to_owned(), Value::String(value.to_owned()));
        return true;
    }

    false
}

/// Render a minimal `project.toml` document from validated metadata.
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
        InitOptions::new(None, default_name.to_owned(), None, None)
            .expect("default test options should be valid")
    }

    fn issues_file_path(project_path: &Path, file_name: &str) -> PathBuf {
        project_path.join(ISSUES_DIR).join(file_name)
    }

    fn assert_issue_state_dirs_exist(project_path: &Path, expected_states: &[&str]) {
        for state in expected_states {
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
            InitOptions::new(
                Some("My Project".to_owned()),
                "Ignored".to_owned(),
                Some("Git-native tracker".to_owned()),
                Some("https://example.com".to_owned()),
            )
            .expect("init options should be valid"),
        )
        .expect("init should succeed");

        let config = ProjectConfig::load(project_path).expect("project config should load");
        assert_eq!(config.name(), "My Project");
        assert_eq!(config.description(), Some("Git-native tracker"));
        assert_eq!(config.homepage(), Some("https://example.com"));

        assert!(issues_file_path(project_path, ISSUES_WORKFLOW_FILE).is_file());
        assert!(issues_file_path(project_path, ISSUES_CONFIG_FILE).is_file());
        assert!(issues_file_path(project_path, ISSUES_LABELS_FILE).is_file());
        assert_issue_state_dirs_exist(project_path, &["todo", "in_progress", "review", "done"]);

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
        assert_issue_state_dirs_exist(existing_project_path, &["custom"]);
        assert!(!existing_project_path.join(ISSUES_DIR).join("todo").exists());
        assert!(
            !existing_project_path
                .join(ISSUES_DIR)
                .join("in_progress")
                .exists()
        );
        assert!(
            !existing_project_path
                .join(ISSUES_DIR)
                .join("review")
                .exists()
        );
        assert!(!existing_project_path.join(ISSUES_DIR).join("done").exists());

        let project_content = fs::read_to_string(existing_project_path.join(PROJECT_CONFIG_FILE))
            .expect("project config should be readable");
        assert!(project_content.contains("custom = \"keep\""));
    }

    #[test]
    fn initialize_fails_when_existing_workflow_is_invalid() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path();
        let issues_dir = project_path.join(ISSUES_DIR);
        fs::create_dir_all(&issues_dir).expect("issues directory should be created");

        fs::write(
            issues_file_path(project_path, ISSUES_WORKFLOW_FILE),
            "initial_state = \"todo\"\n[states]\nreview = { name = \"Review\" }\n",
        )
        .expect("workflow config should be written");

        let err = initialize(project_path, default_options("demo"))
            .expect_err("init should fail for invalid workflow config");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
        assert!(!project_path.join(PROJECT_CONFIG_FILE).exists());
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
            InitOptions::new(
                Some("Renamed".to_owned()),
                "Ignored".to_owned(),
                Some("Updated description".to_owned()),
                Some("https://example.com/project".to_owned()),
            )
            .expect("init options should be valid"),
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
    fn initialize_does_not_rewrite_project_config_when_updates_match_existing_values() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path();
        let project_toml_path = project_path.join(PROJECT_CONFIG_FILE);
        let original_content = [
            "# keep this comment",
            "name = \"Existing\"",
            "description = \"Same description\"",
            "homepage = \"https://example.com/project\"",
            "custom = \"keep\"",
            "",
        ]
        .join("\n");

        fs::write(&project_toml_path, &original_content).expect("project config should be written");

        initialize(
            project_path,
            InitOptions::new(
                Some("Existing".to_owned()),
                "Ignored".to_owned(),
                Some("Same description".to_owned()),
                Some("https://example.com/project".to_owned()),
            )
            .expect("init options should be valid"),
        )
        .expect("init should succeed");

        let persisted_content =
            fs::read_to_string(project_toml_path).expect("project config should be readable");
        assert_eq!(persisted_content, original_content);
    }

    #[test]
    fn init_options_rejects_empty_name_argument() {
        let err = InitOptions::new(Some("   ".to_owned()), "fallback".to_owned(), None, None)
            .expect_err("empty name should fail");

        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }

    #[test]
    fn init_options_rejects_empty_default_name() {
        let err = InitOptions::new(None, "   ".to_owned(), None, None)
            .expect_err("empty default name should fail");

        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }

    #[test]
    fn init_options_rejects_empty_default_name_even_when_name_is_set() {
        let err = InitOptions::new(Some("Valid".to_owned()), "   ".to_owned(), None, None)
            .expect_err("empty default name should fail even with explicit name");

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
