use std::{collections::BTreeMap, path::Path};

use crate::{
    codec,
    errors::{ConfigValidationError, GitlaneError},
    validate::{validate_id, validate_non_blank},
};

mod repr;
pub mod templates;

/// Stable identifier for a label.
pub type LabelId = String;
/// Stable identifier for a label group.
pub type LabelGroupId = String;

/// Validated label configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelsConfig {
    label_groups: BTreeMap<LabelGroupId, LabelGroup>,
    labels: BTreeMap<LabelId, Label>,
}

/// Validated label group definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelGroup {
    name: String,
    description: Option<String>,
    color: Option<String>,
}

/// Validated label definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    name: String,
    description: Option<String>,
    color: Option<String>,
    group: Option<LabelGroupId>,
}

impl LabelsConfig {
    /// Builds validated label configuration.
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

    /// Loads this config from a supported file path.
    pub fn load(config_path: &Path) -> Result<Self, GitlaneError> {
        codec::load::<Self, repr::LabelsConfigRepr>(config_path)
    }

    /// Saves this config using the format implied by `config_path`.
    pub fn save(&self, config_path: &Path) -> Result<(), GitlaneError> {
        codec::save::<Self, repr::LabelsConfigRepr>(config_path, self)
    }

    /// Returns label groups keyed by group id.
    pub fn label_groups(&self) -> &BTreeMap<LabelGroupId, LabelGroup> {
        &self.label_groups
    }

    /// Returns labels keyed by label id.
    pub fn labels(&self) -> &BTreeMap<LabelId, Label> {
        &self.labels
    }

    /// Resolves a label color, falling back to the group color when needed.
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
    /// Builds a validated label group.
    pub fn new(
        name: String,
        description: Option<String>,
        color: Option<String>,
    ) -> Result<Self, ConfigValidationError> {
        validate_non_blank(&name, "label groups must have a non-empty `name`")?;

        Ok(Self {
            name,
            description,
            color,
        })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional color.
    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }
}

impl Label {
    /// Builds a validated label.
    pub fn new(
        name: String,
        description: Option<String>,
        color: Option<String>,
        group: Option<LabelGroupId>,
    ) -> Result<Self, ConfigValidationError> {
        validate_non_blank(&name, "labels must have a non-empty `name`")?;

        Ok(Self {
            name,
            description,
            color,
            group,
        })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional explicit color.
    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    /// Returns the optional referenced label group id.
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

        validate_non_blank(
            group.name(),
            format!("label group `{group_id}` must have a non-empty `name`"),
        )?;
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

        validate_non_blank(
            label.name(),
            format!("label `{label_id}` must have a non-empty `name`"),
        )?;

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::Path};

    use crate::codec;
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
        codec::parse::<LabelsConfig, super::repr::LabelsConfigRepr>(
            content,
            Path::new("labels.toml"),
        )
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
            .save(&config_path)
            .expect("labels config should save");
        let loaded =
            LabelsConfig::load(&config_path).expect("labels config should load after saving");

        assert_eq!(loaded, config);
        assert_eq!(
            fs::read_to_string(config_path).expect("labels config should be readable"),
            concat!(
                "[label_groups]\n",
                "type = { name = \"Type\", description = \"Group description\", color = \"#334155\" }\n",
                "\n",
                "[labels]\n",
                "blocked = { name = \"Blocked\", description = \"Label description\", color = \"#b91c1c\" }\n",
                "type_bug = { name = \"Bug\", description = \"Label description\", group = \"type\" }\n",
            )
        );
    }

    #[test]
    fn loads_yaml_labels_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("labels.yaml");
        fs::write(
            &config_path,
            concat!(
                "label_groups:\n",
                "  type:\n",
                "    name: Type\n",
                "    color: '#334155'\n",
                "labels:\n",
                "  type_bug:\n",
                "    name: Bug\n",
                "    group: type\n",
                "  blocked:\n",
                "    name: Blocked\n",
                "    color: '#b91c1c'\n"
            ),
        )
        .expect("yaml labels config should be written");

        let config = LabelsConfig::load(&config_path).expect("yaml labels config should load");

        assert_eq!(config.resolved_color("type_bug"), Some("#334155"));
        assert_eq!(config.resolved_color("blocked"), Some("#b91c1c"));
    }

    #[test]
    fn saves_and_loads_yaml_labels_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("labels.yaml");
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
            .save(&config_path)
            .expect("yaml labels config should save");
        let loaded =
            LabelsConfig::load(&config_path).expect("yaml labels config should load after saving");

        assert_eq!(loaded, config);
    }

    #[test]
    fn loads_yml_labels_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("labels.yml");
        fs::write(
            &config_path,
            concat!(
                "labels:\n",
                "  blocked:\n",
                "    name: Blocked\n",
                "    color: '#b91c1c'\n"
            ),
        )
        .expect("yml labels config should be written");

        let config = LabelsConfig::load(&config_path).expect("yml labels config should load");

        assert_eq!(config.resolved_color("blocked"), Some("#b91c1c"));
    }

    #[test]
    fn saves_and_loads_yml_labels_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("labels.yml");
        let config = LabelsConfig::new(
            BTreeMap::new(),
            BTreeMap::from([(
                "blocked".to_owned(),
                label("Blocked", Some("#b91c1c"), None),
            )]),
        )
        .expect("labels config should be valid");

        config
            .save(&config_path)
            .expect("yml labels config should save");
        let loaded =
            LabelsConfig::load(&config_path).expect("yml labels config should load after saving");

        assert_eq!(loaded, config);
    }

    #[test]
    fn saves_and_loads_json_labels_config() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let config_path = temp_dir.path().join("labels.json");
        let config = LabelsConfig::new(
            BTreeMap::new(),
            BTreeMap::from([(
                "blocked".to_owned(),
                label("Blocked", Some("#b91c1c"), None),
            )]),
        )
        .expect("labels config should be valid");

        config
            .save(&config_path)
            .expect("json labels config should save");
        let loaded =
            LabelsConfig::load(&config_path).expect("json labels config should load after saving");

        assert_eq!(loaded, config);
    }
}
