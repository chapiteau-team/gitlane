use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{errors::GitlaneError, fs::ensure_file, paths::ISSUES_DIR, validate::ValidationError};

/// Basename for the project config file.
pub const PROJECT_CONFIG_STEM: &str = "project";

/// Basename for the issue config file.
pub const ISSUES_CONFIG_STEM: &str = "issues";

/// Basename for the issues labels config file.
pub const ISSUES_LABELS_CONFIG_STEM: &str = "labels";

/// Basename for the issues workflow config file.
pub const ISSUES_WORKFLOW_CONFIG_STEM: &str = "workflow";

/// Logical config file kinds supported by Gitlane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKind {
    Project,
    Issues,
    IssuesLabels,
    IssuesWorkflow,
}

impl ConfigKind {
    /// Returns the basename used for this config file kind.
    pub const fn stem(self) -> &'static str {
        match self {
            Self::Project => PROJECT_CONFIG_STEM,
            Self::Issues => ISSUES_CONFIG_STEM,
            Self::IssuesLabels => ISSUES_LABELS_CONFIG_STEM,
            Self::IssuesWorkflow => ISSUES_WORKFLOW_CONFIG_STEM,
        }
    }
}

/// File extensions accepted for config files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileExtension {
    Toml,
    Json,
    Yaml,
    Yml,
}

/// Parser-specific errors for supported config formats.
#[derive(Debug, Error)]
pub enum ConfigParseError {
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl ConfigParseError {
    pub(crate) fn format_name(&self) -> &'static str {
        match self {
            Self::Toml(_) => "TOML",
            Self::Json(_) => "JSON",
            Self::Yaml(_) => "YAML",
        }
    }
}

/// Serializer-specific errors for supported config formats.
#[derive(Debug, Error)]
pub enum ConfigSerializeError {
    #[error(transparent)]
    Toml(#[from] toml_edit::ser::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl ConfigSerializeError {
    pub(crate) fn format_name(&self) -> &'static str {
        match self {
            Self::Toml(_) => "TOML",
            Self::Json(_) => "JSON",
            Self::Yaml(_) => "YAML",
        }
    }
}

/// Validation error for parsed config content.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct ConfigValidationError(#[from] ValidationError);

impl ConfigValidationError {
    /// Creates a new validation error with a user-facing message.
    pub fn new(message: impl Into<String>) -> Self {
        Self(ValidationError::new(message))
    }
}

impl ConfigFileExtension {
    /// Returns the file extension without a leading dot.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Yml => "yml",
        }
    }

    /// Returns all supported config file extensions.
    pub const fn all() -> [Self; 4] {
        [Self::Toml, Self::Json, Self::Yaml, Self::Yml]
    }

    /// Infers the config file extension from a path.
    pub fn from_path(path: &Path) -> Result<Self, GitlaneError> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("toml") => Ok(Self::Toml),
            Some("json") => Ok(Self::Json),
            Some("yaml") => Ok(Self::Yaml),
            Some("yml") => Ok(Self::Yml),
            _ => Err(GitlaneError::UnsupportedConfigFormat {
                path: path.to_path_buf(),
            }),
        }
    }
}

/// Returns the parent directory that stores a config kind.
pub fn config_dir(project_dir: &Path, kind: ConfigKind) -> PathBuf {
    match kind {
        ConfigKind::Project => project_dir.to_path_buf(),
        ConfigKind::IssuesWorkflow | ConfigKind::Issues | ConfigKind::IssuesLabels => {
            project_dir.join(ISSUES_DIR)
        }
    }
}

/// Returns the filename for a config kind and extension.
pub fn config_file_name(kind: ConfigKind, extension: ConfigFileExtension) -> String {
    format!("{}.{}", kind.stem(), extension.as_str())
}

/// Returns all supported filenames for a config kind.
pub fn config_file_names(kind: ConfigKind) -> [String; 4] {
    ConfigFileExtension::all().map(|extension| config_file_name(kind, extension))
}

/// Returns the config path for a project, kind, and extension.
pub fn config_path(
    project_dir: &Path,
    kind: ConfigKind,
    extension: ConfigFileExtension,
) -> PathBuf {
    config_dir(project_dir, kind).join(config_file_name(kind, extension))
}

/// Returns candidate paths for all supported extensions.
pub fn config_candidate_paths(project_dir: &Path, kind: ConfigKind) -> [PathBuf; 4] {
    ConfigFileExtension::all().map(|extension| config_path(project_dir, kind, extension))
}

/// Finds the single supported config file for a kind.
///
/// Returns `Ok(None)` when no config file exists and errors when multiple
/// supported files exist for the same logical config.
pub fn discover_config_path(
    project_dir: &Path,
    kind: ConfigKind,
) -> Result<Option<PathBuf>, GitlaneError> {
    let mut matches = Vec::new();

    for candidate in config_candidate_paths(project_dir, kind) {
        if candidate.exists() {
            ensure_file(&candidate)?;
            matches.push(candidate);
        }
    }

    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.pop()),
        _ => Err(GitlaneError::AmbiguousConfigFiles {
            config_name: kind.stem(),
            paths: matches,
        }),
    }
}

/// Finds the single required config file for a kind.
pub fn require_config_path(project_dir: &Path, kind: ConfigKind) -> Result<PathBuf, GitlaneError> {
    discover_config_path(project_dir, kind)?.ok_or_else(|| GitlaneError::MissingConfigFile {
        config_name: kind.stem(),
        directory: config_dir(project_dir, kind),
    })
}
