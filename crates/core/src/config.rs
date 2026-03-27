use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    errors::{ConfigParseError, ConfigValidationError, GitlaneError},
    fs::{read_text_file, write_text_file},
    paths::ISSUES_DIR,
};

pub const PROJECT_CONFIG_STEM: &str = "project";
pub const WORKFLOW_CONFIG_STEM: &str = "workflow";
pub const ISSUES_CONFIG_STEM: &str = "issues";
pub const LABELS_CONFIG_STEM: &str = "labels";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKind {
    Project,
    Workflow,
    Issues,
    Labels,
}

impl ConfigKind {
    pub const fn stem(self) -> &'static str {
        match self {
            Self::Project => PROJECT_CONFIG_STEM,
            Self::Workflow => WORKFLOW_CONFIG_STEM,
            Self::Issues => ISSUES_CONFIG_STEM,
            Self::Labels => LABELS_CONFIG_STEM,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileExtension {
    Toml,
    Yaml,
    Yml,
}

impl ConfigFileExtension {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Yml => "yml",
        }
    }

    pub const fn all() -> [Self; 3] {
        [Self::Toml, Self::Yaml, Self::Yml]
    }

    pub fn from_path(path: &Path) -> Result<Self, GitlaneError> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("toml") => Ok(Self::Toml),
            Some("yaml") => Ok(Self::Yaml),
            Some("yml") => Ok(Self::Yml),
            _ => Err(GitlaneError::UnsupportedConfigFormat {
                path: path.to_path_buf(),
            }),
        }
    }
}

pub fn config_file_name(kind: ConfigKind, extension: ConfigFileExtension) -> String {
    format!("{}.{}", kind.stem(), extension.as_str())
}

pub fn config_file_names(kind: ConfigKind) -> [String; 3] {
    ConfigFileExtension::all().map(|extension| config_file_name(kind, extension))
}

pub fn config_path(
    project_dir: &Path,
    kind: ConfigKind,
    extension: ConfigFileExtension,
) -> PathBuf {
    config_dir(project_dir, kind).join(config_file_name(kind, extension))
}

pub fn default_config_path(project_dir: &Path, kind: ConfigKind) -> PathBuf {
    config_path(project_dir, kind, ConfigFileExtension::Toml)
}

pub fn config_candidate_paths(project_dir: &Path, kind: ConfigKind) -> [PathBuf; 3] {
    ConfigFileExtension::all().map(|extension| config_path(project_dir, kind, extension))
}

fn config_dir(project_dir: &Path, kind: ConfigKind) -> PathBuf {
    match kind {
        ConfigKind::Project => project_dir.to_path_buf(),
        ConfigKind::Workflow | ConfigKind::Issues | ConfigKind::Labels => {
            project_dir.join(ISSUES_DIR)
        }
    }
}

pub(crate) fn validate_non_blank(
    value: &str,
    message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    if value.trim().is_empty() {
        return Err(ConfigValidationError::new(message));
    }

    Ok(())
}

pub(crate) fn validate_id(
    id: &str,
    empty_message: impl Into<String>,
    whitespace_message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    if id.is_empty() {
        return Err(ConfigValidationError::new(empty_message));
    }

    if id.trim() != id {
        return Err(ConfigValidationError::new(whitespace_message));
    }

    Ok(())
}

pub(crate) fn load_config_from_path<T>(
    config_path: &Path,
    parse: impl FnOnce(&str, &Path) -> Result<T, GitlaneError>,
) -> Result<T, GitlaneError> {
    let content = read_text_file(config_path)?;
    parse(&content, config_path)
}

pub(crate) fn parse_config<T, Repr, E>(
    content: &str,
    config_path: &Path,
    parse: impl FnOnce(&str) -> Result<Repr, E>,
) -> Result<T, GitlaneError>
where
    T: TryFrom<Repr, Error = ConfigValidationError>,
    E: Into<ConfigParseError>,
{
    let repr = parse(content).map_err(|source| GitlaneError::parse_config(config_path, source))?;

    repr.try_into()
        .map_err(|source| GitlaneError::invalid_config(config_path, source))
}

pub(crate) fn to_toml_string<T: Serialize>(
    value: &T,
    config_path: &Path,
) -> Result<String, GitlaneError> {
    ::toml::to_string(value).map_err(|source| GitlaneError::SerializeToml {
        path: config_path.to_path_buf(),
        source,
    })
}

pub(crate) fn save_toml_config<T: Serialize>(
    config_path: &Path,
    value: &T,
) -> Result<(), GitlaneError> {
    let content = to_toml_string(value, config_path)?;
    write_text_file(config_path, &content)?;
    Ok(())
}

macro_rules! impl_config {
    ($type:ident) => {
        impl $type {
            pub fn load_from_path(
                config_path: &std::path::Path,
            ) -> Result<Self, $crate::errors::GitlaneError> {
                match $crate::config::ConfigFileExtension::from_path(config_path)? {
                    $crate::config::ConfigFileExtension::Toml => toml::load_from_path(config_path),
                    $crate::config::ConfigFileExtension::Yaml
                    | $crate::config::ConfigFileExtension::Yml => yaml::load_from_path(config_path),
                }
            }
        }
    };
}

pub(crate) use impl_config;
