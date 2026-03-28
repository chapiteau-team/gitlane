use std::path::{Path, PathBuf};

use crate::{errors::GitlaneError, fs::ensure_file, paths::ISSUES_DIR};

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

fn config_dir(project_dir: &Path, kind: ConfigKind) -> PathBuf {
    match kind {
        ConfigKind::Project => project_dir.to_path_buf(),
        ConfigKind::Workflow | ConfigKind::Issues | ConfigKind::Labels => {
            project_dir.join(ISSUES_DIR)
        }
    }
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

            pub fn save_to_path(
                &self,
                config_path: &std::path::Path,
            ) -> Result<(), $crate::errors::GitlaneError> {
                match $crate::config::ConfigFileExtension::from_path(config_path)? {
                    $crate::config::ConfigFileExtension::Toml => {
                        toml::save_to_path(config_path, self)
                    }
                    $crate::config::ConfigFileExtension::Yaml
                    | $crate::config::ConfigFileExtension::Yml => {
                        yaml::save_to_path(config_path, self)
                    }
                }
            }
        }
    };
}

pub(crate) use impl_config;
