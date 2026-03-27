use std::path::PathBuf;

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
    #[error(transparent)]
    InvalidPersonHandle(#[from] PersonHandleError),
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

#[derive(Debug, Error)]
pub enum PersonHandleError {
    #[error("`people[{index}]` must be a non-empty handle")]
    Empty { index: usize },
    #[error("duplicate handle `{handle}` in `people`")]
    Duplicate { handle: String },
}
