//! Shared error types for Gitlane core operations.

use std::path::{Path, PathBuf};

use thiserror::Error;

pub use crate::frontmatter::{FrontmatterParseError, FrontmatterSerializeError};

use crate::{
    config::{ConfigParseError, ConfigSerializeError, ConfigValidationError},
    frontmatter::{FrontmatterError, FrontmatterValidationError},
    fs::FsError,
    issues::issue::IssueValidationError,
};

/// Top-level error type for Gitlane core operations.
#[derive(Debug, Error)]
pub enum GitlaneError {
    /// The provided project name was empty or whitespace-only.
    #[error("project name must be a non-empty, non-whitespace string")]
    InvalidProjectName,
    /// Initialization was attempted where a project config already exists.
    #[error("project already exists at `{path}`")]
    ProjectAlreadyExists { path: PathBuf },
    /// A required logical config file was not found.
    #[error("missing supported {config_name} config file in `{directory}`")]
    MissingConfigFile {
        config_name: &'static str,
        directory: PathBuf,
    },
    /// More than one supported file exists for the same logical config.
    #[error("found multiple supported {config_name} config files: {paths:?}")]
    AmbiguousConfigFiles {
        config_name: &'static str,
        paths: Vec<PathBuf>,
    },
    /// A config file path did not use a supported extension.
    #[error("unsupported config format for `{path}`; expected .toml, .json, .yaml, or .yml")]
    UnsupportedConfigFormat { path: PathBuf },
    /// A config file parsed but failed semantic validation.
    #[error("invalid config in `{path}`: {message}")]
    InvalidConfig { path: PathBuf, message: String },
    /// Parsing a config file failed.
    #[error("failed to parse `{path}` as {format}", format = .source.format_name())]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: ConfigParseError,
    },
    /// Serializing a config file failed.
    #[error("failed to serialize `{path}` as {format}", format = .source.format_name())]
    SerializeConfig {
        path: PathBuf,
        #[source]
        source: ConfigSerializeError,
    },
    /// Issue front matter parsed but failed semantic validation.
    #[error("invalid front matter in `{path}`: {message}")]
    InvalidFrontmatter { path: PathBuf, message: String },
    /// Parsing issue front matter failed.
    #[error("failed to parse front matter in `{path}` as {format}", format = .source.format_name())]
    ParseFrontmatter {
        path: PathBuf,
        #[source]
        source: FrontmatterParseError,
    },
    /// Serializing issue front matter failed.
    #[error(
        "failed to serialize front matter in `{path}` as {format}",
        format = .source.format_name()
    )]
    SerializeFrontmatter {
        path: PathBuf,
        #[source]
        source: FrontmatterSerializeError,
    },
    /// Parsed issue metadata failed semantic validation.
    #[error("invalid issue in `{path}`: {message}")]
    InvalidIssue { path: PathBuf, message: String },
    /// A filesystem operation failed.
    #[error(transparent)]
    Filesystem(#[from] FsError),
}

impl GitlaneError {
    /// Builds an invalid-config error from a validation failure.
    pub(crate) fn invalid_config(path: &Path, source: ConfigValidationError) -> Self {
        Self::InvalidConfig {
            path: path.to_path_buf(),
            message: source.to_string(),
        }
    }

    /// Builds a parse error for a config file.
    pub(crate) fn parse_config(path: &Path, source: impl Into<ConfigParseError>) -> Self {
        Self::ParseConfig {
            path: path.to_path_buf(),
            source: source.into(),
        }
    }

    /// Builds a serialization error for a config file.
    pub(crate) fn serialize_config(path: &Path, source: impl Into<ConfigSerializeError>) -> Self {
        Self::SerializeConfig {
            path: path.to_path_buf(),
            source: source.into(),
        }
    }

    /// Builds an invalid-front-matter error from a validation failure.
    pub(crate) fn invalid_frontmatter(path: &Path, source: FrontmatterValidationError) -> Self {
        Self::InvalidFrontmatter {
            path: path.to_path_buf(),
            message: source.to_string(),
        }
    }

    /// Builds a parse error for a front matter document.
    pub(crate) fn parse_frontmatter(path: &Path, source: FrontmatterParseError) -> Self {
        Self::ParseFrontmatter {
            path: path.to_path_buf(),
            source,
        }
    }

    /// Builds a top-level error from a front matter parsing failure.
    pub(crate) fn from_frontmatter(path: &Path, source: FrontmatterError) -> Self {
        match source {
            FrontmatterError::Validation(source) => Self::invalid_frontmatter(path, source),
            FrontmatterError::Parse(source) => Self::parse_frontmatter(path, source),
        }
    }

    /// Builds a serialization error for a front matter document.
    pub(crate) fn serialize_frontmatter(path: &Path, source: FrontmatterSerializeError) -> Self {
        Self::SerializeFrontmatter {
            path: path.to_path_buf(),
            source,
        }
    }

    /// Builds an invalid-issue error from a validation failure.
    pub(crate) fn invalid_issue(path: &Path, source: IssueValidationError) -> Self {
        Self::InvalidIssue {
            path: path.to_path_buf(),
            message: source.to_string(),
        }
    }
}
