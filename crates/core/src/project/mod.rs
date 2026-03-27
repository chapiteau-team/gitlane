use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    config::{ConfigKind, config_candidate_paths, impl_config, validate_non_blank},
    errors::{ConfigValidationError, GitlaneError},
    fs::ensure_file,
};

mod codec;
pub mod toml;
mod yaml;

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
        validate_non_blank(
            &name,
            "project name must be a non-empty, non-whitespace string",
        )?;

        let mut seen = HashSet::with_capacity(people.len());
        for (index, handle) in people.iter().enumerate() {
            validate_non_blank(
                handle,
                format!("`people[{index}]` must be a non-empty handle"),
            )?;

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

    /// Load and validate project configuration from the supported config file.
    pub fn load(project_dir: &Path) -> Result<Self, GitlaneError> {
        match discover_config_path(project_dir)? {
            Some(config_path) => Self::load_from_path(&config_path),
            None => toml::load(project_dir),
        }
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

impl_config!(ProjectConfig);

fn discover_config_path(project_dir: &Path) -> Result<Option<PathBuf>, GitlaneError> {
    let mut matches = Vec::new();

    for candidate in config_candidate_paths(project_dir, ConfigKind::Project) {
        if candidate.exists() {
            ensure_file(&candidate)?;
            matches.push(candidate);
        }
    }

    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.pop()),
        _ => Err(GitlaneError::AmbiguousConfigFiles {
            config_name: "project",
            paths: matches,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::Path};

    use crate::{
        config::{ConfigFileExtension, ConfigKind, config_path},
        errors::ConfigParseError,
    };
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
        let config_path = config_path(
            temp_dir.path(),
            ConfigKind::Project,
            ConfigFileExtension::Toml,
        );
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

    #[test]
    fn loads_yaml_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        fs::write(
            config_path(
                temp_dir.path(),
                ConfigKind::Project,
                ConfigFileExtension::Yaml,
            ),
            concat!(
                "name: Gitlane\n",
                "description: Git-native task tracker\n",
                "homepage: https://github.com/example/gitlane\n",
                "people:\n",
                "  - '@alice'\n",
                "  - '@bob'\n"
            ),
        )
        .expect("yaml project config should be written");

        let config = ProjectConfig::load(temp_dir.path()).expect("yaml project config should load");

        assert_eq!(config.name(), "Gitlane");
        assert_eq!(config.description(), Some("Git-native task tracker"));
        assert_eq!(
            config.homepage(),
            Some("https://github.com/example/gitlane")
        );
        assert_eq!(
            config.people(),
            &["@alice".to_string(), "@bob".to_string(),]
        );
    }

    #[test]
    fn loads_yml_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        fs::write(
            config_path(
                temp_dir.path(),
                ConfigKind::Project,
                ConfigFileExtension::Yml,
            ),
            "name: Gitlane\n",
        )
        .expect("yml project config should be written");

        let config = ProjectConfig::load(temp_dir.path()).expect("yml project config should load");

        assert_eq!(config.name(), "Gitlane");
    }

    #[test]
    fn errors_when_multiple_project_config_formats_exist() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_dir = temp_dir.path();
        fs::write(
            config_path(project_dir, ConfigKind::Project, ConfigFileExtension::Toml),
            "name = \"Gitlane\"\n",
        )
        .expect("toml project config should be written");
        fs::write(
            config_path(project_dir, ConfigKind::Project, ConfigFileExtension::Yaml),
            "name: Gitlane\n",
        )
        .expect("yaml project config should be written");

        let err = ProjectConfig::load(project_dir)
            .expect_err("multiple project config files should fail");

        assert!(matches!(err, GitlaneError::AmbiguousConfigFiles { .. }));
    }

    #[test]
    fn reports_toml_parse_errors_with_unified_variant() {
        let err = toml::parse_str("name = [", Path::new("project.toml"))
            .expect_err("invalid TOML should fail");

        assert!(matches!(
            err,
            GitlaneError::ParseConfig {
                source: ConfigParseError::Toml(_),
                ..
            }
        ));
    }

    #[test]
    fn reports_yaml_parse_errors_with_unified_variant() {
        let err = yaml::parse_str("name: [", Path::new("project.yaml"))
            .expect_err("invalid YAML should fail");

        assert!(matches!(
            err,
            GitlaneError::ParseConfig {
                source: ConfigParseError::Yaml(_),
                ..
            }
        ));
    }
}
