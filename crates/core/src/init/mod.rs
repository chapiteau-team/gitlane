//! Project initialization routines.
//!
//! Initialization ensures the target directory exists, scaffolds issue
//! workflow/config/label files, and creates `project.toml`.

use std::path::Path;
use toml::Value;

use crate::{
    errors::GitlaneError,
    fs::{ensure_directory, ensure_file, write_file_if_missing, write_text_file},
    issues::workflow::WorkflowConfig,
    paths::{
        ISSUES_CONFIG_FILE, ISSUES_DIR, ISSUES_LABELS_FILE, ISSUES_WORKFLOW_FILE,
        PROJECT_CONFIG_FILE,
    },
};

const ISSUES_WORKFLOW_TOML: &str = include_str!("scaffold/issues/workflow.toml");
const ISSUES_CONFIG_TOML: &str = include_str!("scaffold/issues/issues.toml");
const ISSUES_LABELS_TOML: &str = include_str!("scaffold/issues/labels.toml");
const ISSUES_SCAFFOLD_FILES: [(&str, &str); 3] = [
    (ISSUES_WORKFLOW_FILE, ISSUES_WORKFLOW_TOML),
    (ISSUES_CONFIG_FILE, ISSUES_CONFIG_TOML),
    (ISSUES_LABELS_FILE, ISSUES_LABELS_TOML),
];

/// Options that control new project initialization metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    /// Project name written into a new config.
    name: String,
    /// Optional project description to set when creating a new config.
    description: Option<String>,
    /// Optional homepage URL string to set when creating a new config.
    homepage: Option<String>,
}

impl InitOptions {
    /// Build validated initialization options.
    pub fn new(
        name: String,
        description: Option<String>,
        homepage: Option<String>,
    ) -> Result<Self, GitlaneError> {
        Self::validate_project_name(&name)?;

        Ok(Self {
            name,
            description,
            homepage,
        })
    }

    /// Return the name to use when creating a new project config file.
    fn project_name(&self) -> &str {
        self.name.as_str()
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
/// This creates missing directories, scaffolds issue files, and creates a new
/// `project.toml` when one does not already exist.
pub(crate) fn initialize(project_path: &Path, options: InitOptions) -> Result<(), GitlaneError> {
    ensure_project_root(project_path)?;
    ensure_project_config_missing(project_path)?;
    ensure_issues_layout(project_path)?;
    create_project_config(&project_path.join(PROJECT_CONFIG_FILE), &options)?;

    Ok(())
}

/// Ensure the project root directory exists.
fn ensure_project_root(project_path: &Path) -> Result<(), GitlaneError> {
    ensure_directory(project_path)?;
    Ok(())
}

/// Ensure `project.toml` is not already present for this project.
fn ensure_project_config_missing(project_path: &Path) -> Result<(), GitlaneError> {
    let config_path = project_path.join(PROJECT_CONFIG_FILE);
    if config_path.exists() {
        ensure_file(&config_path)?;
        return Err(GitlaneError::ProjectAlreadyExists { path: config_path });
    }

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

    fn options(name: &str) -> InitOptions {
        InitOptions::new(name.to_owned(), None, None).expect("test init options should be valid")
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
                "My Project".to_owned(),
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
    fn initialize_uses_provided_name() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path();
        initialize(project_path, options("Fallback")).expect("init should succeed");

        let config = ProjectConfig::load(project_path).expect("project config should load");
        assert_eq!(config.name(), "Fallback");
    }

    #[test]
    fn initialize_completes_partial_scaffold_without_overwriting_existing_issue_files() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let existing_project_path = temp_dir.path();
        let issues_dir = existing_project_path.join(ISSUES_DIR);
        fs::create_dir_all(&issues_dir).expect("issues directory should be created");

        let workflow_path = issues_file_path(existing_project_path, ISSUES_WORKFLOW_FILE);
        let custom_workflow =
            "initial_state = \"custom\"\n[states]\ncustom = { name = \"Custom\" }\n";
        fs::write(&workflow_path, custom_workflow).expect("workflow config should be written");

        let labels_path = issues_file_path(existing_project_path, ISSUES_LABELS_FILE);
        let custom_labels = "[labels]\ncustom = { name = \"Custom\" }\n";
        fs::write(&labels_path, custom_labels).expect("labels config should be written");

        initialize(existing_project_path, options("Unused")).expect("init should succeed");

        let config =
            ProjectConfig::load(existing_project_path).expect("project config should load");
        assert_eq!(config.name(), "Unused");
        assert_eq!(
            fs::read_to_string(&workflow_path).expect("workflow config should be readable"),
            custom_workflow
        );
        assert_eq!(
            fs::read_to_string(&labels_path).expect("labels config should be readable"),
            custom_labels
        );
        assert!(issues_file_path(existing_project_path, ISSUES_CONFIG_FILE).is_file());
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

        let err = initialize(project_path, options("demo"))
            .expect_err("init should fail for invalid workflow config");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
        assert!(!project_path.join(PROJECT_CONFIG_FILE).exists());
    }

    #[test]
    fn initialize_fails_when_project_config_already_exists() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path();
        let project_toml_path = project_path.join(PROJECT_CONFIG_FILE);
        fs::write(&project_toml_path, "name = \"Existing\"\n")
            .expect("project config should be written");

        let err = initialize(project_path, options("Ignored"))
            .expect_err("init should fail when project already exists");

        assert!(matches!(
            err,
            GitlaneError::ProjectAlreadyExists { ref path } if path == &project_toml_path
        ));
        assert!(!project_path.join(ISSUES_DIR).exists());
    }

    #[test]
    fn initialize_does_not_update_existing_project_config() {
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
                "Renamed".to_owned(),
                Some("Updated description".to_owned()),
                Some("https://example.com/updated".to_owned()),
            )
            .expect("init options should be valid"),
        )
        .expect_err("init should fail when project already exists");

        let persisted_content =
            fs::read_to_string(project_toml_path).expect("project config should be readable");
        assert_eq!(persisted_content, original_content);
    }

    #[test]
    fn init_options_rejects_empty_name() {
        let err =
            InitOptions::new("   ".to_owned(), None, None).expect_err("empty name should fail");

        assert!(matches!(err, GitlaneError::InvalidProjectName));
    }

    #[test]
    fn initialize_supports_non_gitlane_project_path() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_path = temp_dir.path().join("custom-project-data");

        initialize(&project_path, options("demo")).expect("init should succeed");

        assert!(project_path.join(PROJECT_CONFIG_FILE).is_file());
        assert!(issues_file_path(&project_path, ISSUES_WORKFLOW_FILE).is_file());
    }

    #[test]
    fn initialize_creates_missing_parent_directories() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let missing_parent = temp_dir.path().join("missing-parent");
        let project_path = missing_parent.join("project-data");

        initialize(&project_path, options("demo"))
            .expect("init should create missing parent directories");

        assert!(project_path.join(PROJECT_CONFIG_FILE).is_file());
    }
}
