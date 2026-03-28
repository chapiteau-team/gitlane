use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::fs::FsError;

#[derive(Debug, Error)]
pub enum GitlaneError {
    #[error("project name must be a non-empty, non-whitespace string")]
    InvalidProjectName,
    #[error("project already exists at `{path}`")]
    ProjectAlreadyExists { path: PathBuf },
    #[error("invalid config in `{path}`: {message}")]
    InvalidConfig { path: PathBuf, message: String },
    #[error("failed to parse `{path}` as {format}", format = .source.format_name())]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: ConfigParseError,
    },
    #[error("failed to serialize `{path}` as {format}", format = .source.format_name())]
    SerializeConfig {
        path: PathBuf,
        #[source]
        source: ConfigSerializeError,
    },
    #[error("unsupported config format for `{path}`; expected .toml, .yaml, or .yml")]
    UnsupportedConfigFormat { path: PathBuf },
    #[error("found multiple supported {config_name} config files: {paths:?}")]
    AmbiguousConfigFiles {
        config_name: &'static str,
        paths: Vec<PathBuf>,
    },
    #[error(transparent)]
    Filesystem(#[from] FsError),
}

impl GitlaneError {
    pub fn invalid_config(path: &Path, source: ConfigValidationError) -> Self {
        Self::InvalidConfig {
            path: path.to_path_buf(),
            message: source.to_string(),
        }
    }

    pub fn parse_config(path: &Path, source: impl Into<ConfigParseError>) -> Self {
        Self::ParseConfig {
            path: path.to_path_buf(),
            source: source.into(),
        }
    }

    pub fn serialize_config(path: &Path, source: impl Into<ConfigSerializeError>) -> Self {
        Self::SerializeConfig {
            path: path.to_path_buf(),
            source: source.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigParseError {
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl ConfigParseError {
    fn format_name(&self) -> &'static str {
        match self {
            Self::Toml(_) => "TOML",
            Self::Yaml(_) => "YAML",
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigSerializeError {
    #[error(transparent)]
    Toml(#[from] toml::ser::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl ConfigSerializeError {
    fn format_name(&self) -> &'static str {
        match self {
            Self::Toml(_) => "TOML",
            Self::Yaml(_) => "YAML",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ConfigValidationError {
    message: String,
}

impl ConfigValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
