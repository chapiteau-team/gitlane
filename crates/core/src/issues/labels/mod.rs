use std::{collections::BTreeMap, path::Path};

use crate::errors::{ConfigValidationError, GitlaneError};

pub mod templates;
pub mod toml;

pub type LabelId = String;
pub type LabelGroupId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelsConfig {
    label_groups: BTreeMap<LabelGroupId, LabelGroup>,
    labels: BTreeMap<LabelId, Label>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelGroup {
    name: String,
    description: Option<String>,
    color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    name: String,
    description: Option<String>,
    color: Option<String>,
    group: Option<LabelGroupId>,
}

impl LabelsConfig {
    pub fn new(
        label_groups: BTreeMap<LabelGroupId, LabelGroup>,
        labels: BTreeMap<LabelId, Label>,
    ) -> Result<Self, ConfigValidationError> {
        validate_label_groups(&label_groups)?;
        validate_labels(&label_groups, &labels)?;

        Ok(Self {
            label_groups,
            labels,
        })
    }

    pub fn load_from_path(config_path: &Path) -> Result<Self, GitlaneError> {
        toml::load_from_path(config_path)
    }

    pub fn save_to_path(&self, config_path: &Path) -> Result<(), GitlaneError> {
        toml::save_to_path(config_path, self)
    }

    pub fn label_groups(&self) -> &BTreeMap<LabelGroupId, LabelGroup> {
        &self.label_groups
    }

    pub fn labels(&self) -> &BTreeMap<LabelId, Label> {
        &self.labels
    }

    pub fn resolved_color(&self, label_id: &str) -> Option<&str> {
        let label = self.labels.get(label_id)?;
        label.color().or_else(|| {
            label
                .group()
                .and_then(|group_id| self.label_groups.get(group_id))
                .and_then(LabelGroup::color)
        })
    }
}

impl LabelGroup {
    pub fn new(
        name: String,
        description: Option<String>,
        color: Option<String>,
    ) -> Result<Self, ConfigValidationError> {
        if name.trim().is_empty() {
            return Err(ConfigValidationError::new(
                "label groups must have a non-empty `name`",
            ));
        }

        Ok(Self {
            name,
            description,
            color,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }
}

impl Label {
    pub fn new(
        name: String,
        description: Option<String>,
        color: Option<String>,
        group: Option<LabelGroupId>,
    ) -> Result<Self, ConfigValidationError> {
        if name.trim().is_empty() {
            return Err(ConfigValidationError::new(
                "labels must have a non-empty `name`",
            ));
        }

        Ok(Self {
            name,
            description,
            color,
            group,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }
}

fn validate_label_groups(
    label_groups: &BTreeMap<LabelGroupId, LabelGroup>,
) -> Result<(), ConfigValidationError> {
    for (group_id, group) in label_groups {
        validate_id(
            group_id,
            "label group ids must be non-empty",
            "label group ids must not have leading or trailing whitespace",
        )?;

        if group.name().trim().is_empty() {
            return Err(ConfigValidationError::new(format!(
                "label group `{group_id}` must have a non-empty `name`"
            )));
        }
    }

    Ok(())
}

fn validate_labels(
    label_groups: &BTreeMap<LabelGroupId, LabelGroup>,
    labels: &BTreeMap<LabelId, Label>,
) -> Result<(), ConfigValidationError> {
    for (label_id, label) in labels {
        validate_id(
            label_id,
            "label ids must be non-empty",
            "label ids must not have leading or trailing whitespace",
        )?;

        if label.name().trim().is_empty() {
            return Err(ConfigValidationError::new(format!(
                "label `{label_id}` must have a non-empty `name`"
            )));
        }

        if let Some(group_id) = label.group() {
            validate_id(
                group_id,
                format!("label `{label_id}` must reference a non-empty `group` id"),
                format!(
                    "label `{label_id}` must reference a `group` id without leading or trailing whitespace"
                ),
            )?;

            if !label_groups.contains_key(group_id) {
                return Err(ConfigValidationError::new(format!(
                    "label `{label_id}` references unknown group `{group_id}`"
                )));
            }
        }
    }

    Ok(())
}

fn validate_id(
    id: &str,
    empty_message: impl Into<String>,
    whitespace_message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    if id.is_empty() {
        return Err(ConfigValidationError::new(empty_message));
    }

    if id.trim() != id {
        return Err(ConfigValidationError::new(whitespace_message));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    fn label_group(name: &str, color: Option<&str>) -> LabelGroup {
        LabelGroup::new(
            name.to_owned(),
            Some("Group description".to_owned()),
            color.map(ToOwned::to_owned),
        )
        .expect("label group should be valid")
    }

    fn label(name: &str, color: Option<&str>, group: Option<&str>) -> Label {
        Label::new(
            name.to_owned(),
            Some("Label description".to_owned()),
            color.map(ToOwned::to_owned),
            group.map(ToOwned::to_owned),
        )
        .expect("label should be valid")
    }

    fn parse_labels_config(content: &str) -> Result<LabelsConfig, GitlaneError> {
        toml::parse_str(content, Path::new("labels.toml"))
    }

    #[test]
    fn builds_valid_labels_config() {
        let config = LabelsConfig::new(
            BTreeMap::from([("type".to_owned(), label_group("Type", Some("#334155")))]),
            BTreeMap::from([
                (
                    "blocked".to_owned(),
                    label("Blocked", Some("#b91c1c"), None),
                ),
                ("type_bug".to_owned(), label("Bug", None, Some("type"))),
            ]),
        )
        .expect("labels config should build");

        assert_eq!(config.resolved_color("type_bug"), Some("#334155"));
        assert_eq!(config.resolved_color("blocked"), Some("#b91c1c"));
    }

    #[test]
    fn parses_valid_toml_labels_config() {
        let config = parse_labels_config(
            r##"
[label_groups]
type = { name = "Type", color = "#334155" }

[labels]
type_bug = { name = "Bug", group = "type" }
blocked = { name = "Blocked", color = "#b91c1c" }
"##,
        )
        .expect("labels config should parse");

        assert_eq!(config.resolved_color("type_bug"), Some("#334155"));
        assert_eq!(config.resolved_color("blocked"), Some("#b91c1c"));
    }

    #[test]
    fn default_template_builds_valid_labels_config() {
        let config = templates::default().expect("default labels template should build");

        assert_eq!(config.resolved_color("type_bug"), Some("#334155"));
        assert_eq!(config.resolved_color("blocked"), Some("#b91c1c"));
    }

    #[test]
    fn rejects_unknown_label_group_reference() {
        let err = parse_labels_config(
            r#"
[labels]
type_bug = { name = "Bug", group = "type" }
"#,
        )
        .expect_err("unknown group should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_empty_label_name() {
        let err = parse_labels_config(
            r#"
[labels]
type_bug = { name = "   " }
"#,
        )
        .expect_err("empty label name should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn saves_and_loads_toml_labels_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("labels.toml");
        let config = LabelsConfig::new(
            BTreeMap::from([("type".to_owned(), label_group("Type", Some("#334155")))]),
            BTreeMap::from([
                (
                    "blocked".to_owned(),
                    label("Blocked", Some("#b91c1c"), None),
                ),
                ("type_bug".to_owned(), label("Bug", None, Some("type"))),
            ]),
        )
        .expect("labels config should be valid");

        config
            .save_to_path(&config_path)
            .expect("labels config should save");
        let loaded = LabelsConfig::load_from_path(&config_path)
            .expect("labels config should load after saving");

        assert_eq!(loaded, config);
    }
}
