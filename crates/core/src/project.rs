use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

use crate::paths::PROJECT_CONFIG_FILE;

#[derive(Debug, Error)]
pub enum ProjectConfigError {
    #[error("failed to read `{path}`")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse project config `{path}`")]
    ParseFile {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("`name` must be a non-empty string")]
    EmptyName,
    #[error("`people[{index}]` must be a non-empty handle")]
    EmptyPersonHandle { index: usize },
    #[error("duplicate handle `{handle}` in `people`")]
    DuplicatePersonHandle { handle: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    name: String,
    description: Option<String>,
    homepage: Option<String>,
    people: Vec<String>,
}

impl ProjectConfig {
    pub fn load(project_dir: impl AsRef<Path>) -> Result<Self, ProjectConfigError> {
        let config_path = project_dir.as_ref().join(PROJECT_CONFIG_FILE);
        let content =
            fs::read_to_string(&config_path).map_err(|source| ProjectConfigError::ReadFile {
                path: config_path.clone(),
                source,
            })?;

        let raw: RawProjectConfig =
            toml::from_str(&content).map_err(|source| ProjectConfigError::ParseFile {
                path: config_path,
                source,
            })?;

        Self::from_raw(raw)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    pub fn people(&self) -> &[String] {
        &self.people
    }

    fn from_raw(raw: RawProjectConfig) -> Result<Self, ProjectConfigError> {
        if raw.name.trim().is_empty() {
            return Err(ProjectConfigError::EmptyName);
        }

        let people = raw.people.unwrap_or_default();
        let mut seen = HashSet::with_capacity(people.len());
        for (index, handle) in people.iter().enumerate() {
            if handle.trim().is_empty() {
                return Err(ProjectConfigError::EmptyPersonHandle { index });
            }

            if !seen.insert(handle) {
                return Err(ProjectConfigError::DuplicatePersonHandle {
                    handle: handle.clone(),
                });
            }
        }

        Ok(Self {
            name: raw.name,
            description: raw.description,
            homepage: raw.homepage,
            people,
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawProjectConfig {
    name: String,
    description: Option<String>,
    homepage: Option<String>,
    people: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_project_config(content: &str) -> Result<ProjectConfig, ProjectConfigError> {
        let raw: RawProjectConfig =
            toml::from_str(content).expect("test project config snippets should be valid TOML");
        ProjectConfig::from_raw(raw)
    }

    #[test]
    fn parses_minimal_project_config() {
        let config = parse_project_config(
            r#"
name = "Gitlane"
"#,
        )
        .expect("minimal config should parse");

        assert_eq!(config.name(), "Gitlane");
        assert_eq!(config.description(), None);
        assert_eq!(config.homepage(), None);
        assert!(config.people().is_empty());
    }

    #[test]
    fn parses_full_project_config_with_people() {
        let config = parse_project_config(
            r#"
name = "Gitlane"
description = "Git-native task tracker"
homepage = "https://github.com/example/gitlane"
people = ["@alice", "@bob", "@carol"]
"#,
        )
        .expect("full config should parse");

        assert_eq!(config.name(), "Gitlane");
        assert_eq!(config.description(), Some("Git-native task tracker"));
        assert_eq!(
            config.homepage(),
            Some("https://github.com/example/gitlane")
        );
        assert_eq!(
            config.people(),
            &[
                "@alice".to_string(),
                "@bob".to_string(),
                "@carol".to_string()
            ]
        );
    }

    #[test]
    fn rejects_empty_name() {
        let err = parse_project_config(
            r#"
name = ""
"#,
        )
        .expect_err("empty name should be rejected");

        assert!(matches!(err, ProjectConfigError::EmptyName));
    }

    #[test]
    fn rejects_empty_person_handle() {
        let err = parse_project_config(
            r#"
name = "Gitlane"
people = ["@alice", ""]
"#,
        )
        .expect_err("empty person handle should be rejected");

        assert!(matches!(
            err,
            ProjectConfigError::EmptyPersonHandle { index: 1 }
        ));
    }

    #[test]
    fn rejects_duplicate_people_handles() {
        let err = parse_project_config(
            r#"
name = "Gitlane"
people = ["@alice", "@alice"]
"#,
        )
        .expect_err("duplicate handles should be rejected");

        assert!(matches!(
            err,
            ProjectConfigError::DuplicatePersonHandle { ref handle } if handle == "@alice"
        ));
    }

    #[test]
    fn preserves_people_order() {
        let config = parse_project_config(
            r#"
name = "Gitlane"
people = ["@carol", "@alice", "@bob"]
"#,
        )
        .expect("ordered people list should parse");

        assert_eq!(
            config.people(),
            &[
                "@carol".to_string(),
                "@alice".to_string(),
                "@bob".to_string()
            ]
        );
    }
}
