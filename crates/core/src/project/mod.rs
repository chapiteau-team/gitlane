use std::{collections::HashSet, path::Path};

use crate::errors::{ConfigValidationError, GitlaneError};

pub mod toml;

/// Validated project metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    name: String,
    description: Option<String>,
    homepage: Option<String>,
    people: Vec<String>,
}

impl ProjectConfig {
    /// Build validated project metadata.
    pub fn new(
        name: String,
        description: Option<String>,
        homepage: Option<String>,
        people: Vec<String>,
    ) -> Result<Self, ConfigValidationError> {
        if name.trim().is_empty() {
            return Err(ConfigValidationError::new(
                "project name must be a non-empty, non-whitespace string",
            ));
        }

        let mut seen = HashSet::with_capacity(people.len());
        for (index, handle) in people.iter().enumerate() {
            if handle.trim().is_empty() {
                return Err(ConfigValidationError::new(format!(
                    "`people[{index}]` must be a non-empty handle"
                )));
            }

            if !seen.insert(handle) {
                return Err(ConfigValidationError::new(format!(
                    "duplicate handle `{handle}` in `people`"
                )));
            }
        }

        Ok(Self {
            name,
            description,
            homepage,
            people,
        })
    }

    /// Load and validate project configuration from the default TOML file.
    pub fn load(project_dir: &Path) -> Result<Self, GitlaneError> {
        toml::load(project_dir)
    }

    /// Return the project display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the optional project description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Return the optional homepage URL string.
    pub fn homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    /// Return the ordered list of unique person handles.
    pub fn people(&self) -> &[String] {
        &self.people
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::Path};

    use crate::paths::PROJECT_CONFIG_FILE;
    use tempfile::TempDir;

    fn parse_project_config(content: &str) -> Result<ProjectConfig, GitlaneError> {
        toml::parse_str(content, Path::new("project.toml"))
    }

    #[test]
    fn creates_minimal_project_config() {
        let config = ProjectConfig::new("Gitlane".to_owned(), None, None, Vec::new())
            .expect("minimal config should build");

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
        let err = ProjectConfig::new("".to_owned(), None, None, Vec::new())
            .expect_err("empty name should be rejected");

        assert_eq!(
            err.to_string(),
            "project name must be a non-empty, non-whitespace string"
        );
    }

    #[test]
    fn rejects_empty_person_handle() {
        let err = ProjectConfig::new(
            "Gitlane".to_owned(),
            None,
            None,
            vec!["@alice".to_owned(), "".to_owned()],
        )
        .expect_err("empty person handle should be rejected");

        assert_eq!(err.to_string(), "`people[1]` must be a non-empty handle");
    }

    #[test]
    fn rejects_duplicate_people_handles() {
        let err = ProjectConfig::new(
            "Gitlane".to_owned(),
            None,
            None,
            vec!["@alice".to_owned(), "@alice".to_owned()],
        )
        .expect_err("duplicate handles should be rejected");

        assert_eq!(err.to_string(), "duplicate handle `@alice` in `people`");
    }

    #[test]
    fn preserves_people_order() {
        let config = ProjectConfig::new(
            "Gitlane".to_owned(),
            None,
            None,
            vec!["@carol".to_owned(), "@alice".to_owned(), "@bob".to_owned()],
        )
        .expect("ordered people list should build");

        assert_eq!(
            config.people(),
            &[
                "@carol".to_string(),
                "@alice".to_string(),
                "@bob".to_string()
            ]
        );
    }

    #[test]
    fn saves_and_loads_toml_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join(PROJECT_CONFIG_FILE);
        let config = ProjectConfig::new(
            "Gitlane".to_owned(),
            Some("Git-native task tracker".to_owned()),
            Some("https://example.com".to_owned()),
            vec!["@alice".to_owned()],
        )
        .expect("project config should be valid");

        toml::save_to_path(&config_path, &config).expect("project config should save");
        let loaded = toml::load_from_path(&config_path).expect("project config should load");

        assert_eq!(loaded, config);
        assert_eq!(
            fs::read_to_string(config_path).expect("project config should be readable"),
            concat!(
                "name = \"Gitlane\"\n",
                "description = \"Git-native task tracker\"\n",
                "homepage = \"https://example.com\"\n",
                "people = [\"@alice\"]\n"
            )
        );
    }
}
