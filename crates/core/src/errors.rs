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
}

impl GitlaneError {
    pub(crate) fn invalid_config(path: &Path, source: ConfigValidationError) -> Self {
        Self::InvalidConfig {
            path: path.to_path_buf(),
            message: source.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ConfigValidationError {
    message: String,
}

impl ConfigValidationError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
