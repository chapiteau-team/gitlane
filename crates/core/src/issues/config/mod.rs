use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

use crate::{
    codec,
    errors::{ConfigValidationError, GitlaneError},
    validate::{validate_id, validate_non_blank},
};

mod repr;
pub mod templates;

/// Stable identifier for an issue priority.
pub type PriorityId = String;

/// Validated issue tracker configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuesConfig {
    issue_prefix: String,
    priorities: BTreeMap<PriorityId, IssuePriority>,
    priority_order: Vec<PriorityId>,
}

/// Validated issue priority definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuePriority {
    name: String,
    description: Option<String>,
}

impl IssuesConfig {
    /// Builds validated issue configuration.
    pub fn new(
        issue_prefix: String,
        priorities: BTreeMap<PriorityId, IssuePriority>,
        priority_order: Vec<PriorityId>,
    ) -> Result<Self, ConfigValidationError> {
        validate_issue_prefix(&issue_prefix)?;
        validate_priorities(&priorities)?;
        validate_priority_order(&priorities, &priority_order)?;

        Ok(Self {
            issue_prefix,
            priorities,
            priority_order,
        })
    }

    /// Loads this config from a supported file path.
    pub fn load(config_path: &Path) -> Result<Self, GitlaneError> {
        codec::load::<Self, repr::IssuesConfigRepr>(config_path)
    }

    /// Saves this config using the format implied by `config_path`.
    pub fn save(&self, config_path: &Path) -> Result<(), GitlaneError> {
        codec::save::<Self, repr::IssuesConfigRepr>(config_path, self)
    }

    /// Returns the issue id prefix.
    pub fn issue_prefix(&self) -> &str {
        &self.issue_prefix
    }

    /// Returns priorities keyed by priority id.
    pub fn priorities(&self) -> &BTreeMap<PriorityId, IssuePriority> {
        &self.priorities
    }

    /// Returns the priority display order.
    pub fn priority_order(&self) -> &[PriorityId] {
        &self.priority_order
    }
}

impl IssuePriority {
    /// Builds a validated issue priority.
    pub fn new(name: String, description: Option<String>) -> Result<Self, ConfigValidationError> {
        validate_non_blank(&name, "issue priorities must have a non-empty `name`")?;

        Ok(Self { name, description })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

fn validate_issue_prefix(issue_prefix: &str) -> Result<(), ConfigValidationError> {
    validate_id(
        issue_prefix,
        "`issue_prefix` must be a non-empty string",
        "`issue_prefix` must not have leading or trailing whitespace",
    )
}

fn validate_priorities(
    priorities: &BTreeMap<PriorityId, IssuePriority>,
) -> Result<(), ConfigValidationError> {
    if priorities.is_empty() {
        return Err(ConfigValidationError::new(
            "`priorities` must declare at least one priority",
        ));
    }

    for (priority_id, priority) in priorities {
        validate_id(
            priority_id,
            "priority ids must be non-empty",
            "priority ids must not have leading or trailing whitespace",
        )?;

        validate_non_blank(
            priority.name(),
            format!("priority `{priority_id}` must have a non-empty `name`"),
        )?;
    }

    Ok(())
}

fn validate_priority_order(
    priorities: &BTreeMap<PriorityId, IssuePriority>,
    priority_order: &[PriorityId],
) -> Result<(), ConfigValidationError> {
    if priority_order.is_empty() {
        return Err(ConfigValidationError::new(
            "`priority_order` must contain every priority id exactly once",
        ));
    }

    let mut seen = HashSet::with_capacity(priority_order.len());
    for priority_id in priority_order {
        validate_id(
            priority_id,
            "`priority_order` entries must be non-empty",
            "`priority_order` entries must not have leading or trailing whitespace",
        )?;

        if !priorities.contains_key(priority_id) {
            return Err(ConfigValidationError::new(format!(
                "`priority_order` references unknown priority `{priority_id}`"
            )));
        }

        if !seen.insert(priority_id.clone()) {
            return Err(ConfigValidationError::new(format!(
                "`priority_order` contains duplicate priority `{priority_id}`"
            )));
        }
    }

    if seen.len() != priorities.len() {
        let missing = priorities
            .keys()
            .filter(|priority_id| !seen.contains(*priority_id))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ConfigValidationError::new(format!(
            "`priority_order` is missing priorities: {missing}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::Path};

    use crate::codec;
    use tempfile::TempDir;

    fn priority(name: &str, description: Option<&str>) -> IssuePriority {
        IssuePriority::new(name.to_owned(), description.map(ToOwned::to_owned))
            .expect("priority should be valid")
    }

    fn parse_issues_config(content: &str) -> Result<IssuesConfig, GitlaneError> {
        codec::parse::<IssuesConfig, super::repr::IssuesConfigRepr>(
            content,
            Path::new("issues.toml"),
        )
    }

    #[test]
    fn builds_valid_issues_config() {
        let config = IssuesConfig::new(
            "ISS".to_owned(),
            BTreeMap::from([
                ("p0".to_owned(), priority("No Priority", None)),
                (
                    "p1".to_owned(),
                    priority("Urgent", Some("Needs immediate attention")),
                ),
            ]),
            vec!["p1".to_owned(), "p0".to_owned()],
        )
        .expect("issues config should build");

        assert_eq!(config.issue_prefix(), "ISS");
        assert_eq!(
            config.priority_order(),
            &["p1".to_string(), "p0".to_string()]
        );
    }

    #[test]
    fn parses_valid_toml_config() {
        let config = parse_issues_config(
            r#"
issue_prefix = "ISS"
priority_order = ["p1", "p0"]

[priorities]
p0 = { name = "No Priority" }
p1 = { name = "Urgent", description = "Needs immediate attention" }
"#,
        )
        .expect("issues config should parse");

        assert_eq!(config.issue_prefix(), "ISS");
        assert_eq!(
            config.priority_order(),
            &["p1".to_string(), "p0".to_string()]
        );
    }

    #[test]
    fn default_template_builds_valid_issues_config() {
        let config = templates::default().expect("default issues template should build");

        assert_eq!(config.issue_prefix(), "ISS");
        assert_eq!(
            config.priority_order(),
            &[
                "p1".to_string(),
                "p2".to_string(),
                "p3".to_string(),
                "p4".to_string(),
                "p0".to_string()
            ]
        );
    }

    #[test]
    fn rejects_empty_issue_prefix() {
        let err = parse_issues_config(
            r#"
issue_prefix = ""
priority_order = ["p1"]

[priorities]
p1 = { name = "Urgent" }
"#,
        )
        .expect_err("empty prefix should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_duplicate_priority_in_order() {
        let err = parse_issues_config(
            r#"
issue_prefix = "ISS"
priority_order = ["p1", "p1"]

[priorities]
p1 = { name = "Urgent" }
"#,
        )
        .expect_err("duplicate priority order should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_unknown_priority_in_order() {
        let err = parse_issues_config(
            r#"
issue_prefix = "ISS"
priority_order = ["p2"]

[priorities]
p1 = { name = "Urgent" }
"#,
        )
        .expect_err("unknown ordered priority should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_missing_priority_from_order() {
        let err = parse_issues_config(
            r#"
issue_prefix = "ISS"
priority_order = ["p1"]

[priorities]
p0 = { name = "No Priority" }
p1 = { name = "Urgent" }
"#,
        )
        .expect_err("missing priority in order should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn saves_and_loads_toml_issues_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("issues.toml");
        let config = IssuesConfig::new(
            "ISS".to_owned(),
            BTreeMap::from([
                ("p0".to_owned(), priority("No Priority", None)),
                (
                    "p1".to_owned(),
                    priority("Urgent", Some("Needs immediate attention")),
                ),
            ]),
            vec!["p1".to_owned(), "p0".to_owned()],
        )
        .expect("issues config should be valid");

        config
            .save(&config_path)
            .expect("issues config should save");
        let loaded =
            IssuesConfig::load(&config_path).expect("issues config should load after saving");

        assert_eq!(loaded, config);
    }

    #[test]
    fn loads_yaml_issues_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("issues.yaml");
        fs::write(
            &config_path,
            concat!(
                "issue_prefix: ISS\n",
                "priority_order:\n",
                "  - p1\n",
                "  - p0\n",
                "priorities:\n",
                "  p0:\n",
                "    name: No Priority\n",
                "  p1:\n",
                "    name: Urgent\n",
                "    description: Needs immediate attention\n"
            ),
        )
        .expect("yaml issues config should be written");

        let config = IssuesConfig::load(&config_path).expect("yaml issues config should load");

        assert_eq!(config.issue_prefix(), "ISS");
        assert_eq!(
            config.priority_order(),
            &["p1".to_string(), "p0".to_string()]
        );
    }

    #[test]
    fn saves_and_loads_yaml_issues_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("issues.yaml");
        let config = IssuesConfig::new(
            "ISS".to_owned(),
            BTreeMap::from([
                ("p0".to_owned(), priority("No Priority", None)),
                (
                    "p1".to_owned(),
                    priority("Urgent", Some("Needs immediate attention")),
                ),
            ]),
            vec!["p1".to_owned(), "p0".to_owned()],
        )
        .expect("issues config should be valid");

        config
            .save(&config_path)
            .expect("yaml issues config should save");
        let loaded =
            IssuesConfig::load(&config_path).expect("yaml issues config should load after saving");

        assert_eq!(loaded, config);
    }

    #[test]
    fn loads_yml_issues_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("issues.yml");
        fs::write(
            &config_path,
            concat!(
                "issue_prefix: ISS\n",
                "priority_order:\n",
                "  - p1\n",
                "priorities:\n",
                "  p1:\n",
                "    name: Urgent\n"
            ),
        )
        .expect("yml issues config should be written");

        let config = IssuesConfig::load(&config_path).expect("yml issues config should load");

        assert_eq!(config.issue_prefix(), "ISS");
    }

    #[test]
    fn saves_and_loads_yml_issues_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("issues.yml");
        let config = IssuesConfig::new(
            "ISS".to_owned(),
            BTreeMap::from([("p1".to_owned(), priority("Urgent", None))]),
            vec!["p1".to_owned()],
        )
        .expect("issues config should be valid");

        config
            .save(&config_path)
            .expect("yml issues config should save");
        let loaded =
            IssuesConfig::load(&config_path).expect("yml issues config should load after saving");

        assert_eq!(loaded, config);
    }
}
