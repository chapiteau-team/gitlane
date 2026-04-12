use std::path::{Path, PathBuf};

use thiserror::Error;

pub use crate::frontmatter::{FrontmatterParseError, FrontmatterSerializeError};

use crate::{
    frontmatter::{FrontmatterError, FrontmatterValidationError},
    fs::FsError,
    issues::issue::IssueValidationError,
    validate::ValidationError,
};

/// Top-level error type for Gitlane core operations.
#[derive(Debug, Error)]
pub enum GitlaneError {
    #[error("project name must be a non-empty, non-whitespace string")]
    InvalidProjectName,
    #[error("project already exists at `{path}`")]
    ProjectAlreadyExists { path: PathBuf },
    #[error("missing supported {config_name} config file in `{directory}`")]
    MissingConfigFile {
        config_name: &'static str,
        directory: PathBuf,
    },
    #[error("found multiple supported {config_name} config files: {paths:?}")]
    AmbiguousConfigFiles {
        config_name: &'static str,
        paths: Vec<PathBuf>,
    },
    #[error("unsupported config format for `{path}`; expected .toml, .json, .yaml, or .yml")]
    UnsupportedConfigFormat { path: PathBuf },
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
    #[error("invalid front matter in `{path}`: {message}")]
    InvalidFrontmatter { path: PathBuf, message: String },
    #[error("failed to parse front matter in `{path}` as {format}", format = .source.format_name())]
    ParseFrontmatter {
        path: PathBuf,
        #[source]
        source: FrontmatterParseError,
    },
    #[error(
        "failed to serialize front matter in `{path}` as {format}",
        format = .source.format_name()
    )]
    SerializeFrontmatter {
        path: PathBuf,
        #[source]
        source: FrontmatterSerializeError,
    },
    #[error("invalid issue in `{path}`: {message}")]
    InvalidIssue { path: PathBuf, message: String },
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
    fn format_name(&self) -> &'static str {
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
    fn format_name(&self) -> &'static str {
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
