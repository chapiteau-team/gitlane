use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{errors::GitlaneError, fs::write_text_file};

use crate::fs::read_text_file;

use super::{Label, LabelGroup, LabelGroupId, LabelId, LabelsConfig};

#[derive(Debug, Deserialize, Serialize)]
struct RawLabelsConfig {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    label_groups: BTreeMap<LabelGroupId, RawLabelGroup>,
    labels: BTreeMap<LabelId, RawLabel>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawLabelGroup {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawLabel {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<LabelGroupId>,
}

impl TryFrom<RawLabelGroup> for LabelGroup {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawLabelGroup) -> Result<Self, Self::Error> {
        LabelGroup::new(raw.name, raw.description, raw.color)
    }
}

impl TryFrom<RawLabel> for Label {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawLabel) -> Result<Self, Self::Error> {
        Label::new(raw.name, raw.description, raw.color, raw.group)
    }
}

impl From<&LabelGroup> for RawLabelGroup {
    fn from(group: &LabelGroup) -> Self {
        Self {
            name: group.name().to_owned(),
            description: group.description().map(ToOwned::to_owned),
            color: group.color().map(ToOwned::to_owned),
        }
    }
}

impl From<&Label> for RawLabel {
    fn from(label: &Label) -> Self {
        Self {
            name: label.name().to_owned(),
            description: label.description().map(ToOwned::to_owned),
            color: label.color().map(ToOwned::to_owned),
            group: label.group().map(ToOwned::to_owned),
        }
    }
}

impl TryFrom<RawLabelsConfig> for LabelsConfig {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawLabelsConfig) -> Result<Self, Self::Error> {
        let label_groups = raw
            .label_groups
            .into_iter()
            .map(|(group_id, group)| group.try_into().map(|group| (group_id, group)))
            .collect::<Result<_, _>>()?;
        let labels = raw
            .labels
            .into_iter()
            .map(|(label_id, label)| label.try_into().map(|label| (label_id, label)))
            .collect::<Result<_, _>>()?;

        LabelsConfig::new(label_groups, labels)
    }
}

impl From<&LabelsConfig> for RawLabelsConfig {
    fn from(config: &LabelsConfig) -> Self {
        let label_groups = config
            .label_groups()
            .iter()
            .map(|(group_id, group)| (group_id.clone(), RawLabelGroup::from(group)))
            .collect();
        let labels = config
            .labels()
            .iter()
            .map(|(label_id, label)| (label_id.clone(), RawLabel::from(label)))
            .collect();

        Self {
            label_groups,
            labels,
        }
    }
}

pub fn load_from_path(config_path: &Path) -> Result<LabelsConfig, GitlaneError> {
    let content = read_text_file(config_path)?;
    parse_str(&content, config_path)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<LabelsConfig, GitlaneError> {
    let raw: RawLabelsConfig =
        ::toml::from_str(content).map_err(|source| GitlaneError::ParseToml {
            path: config_path.to_path_buf(),
            source,
        })?;

    raw.try_into()
        .map_err(|source| GitlaneError::invalid_config(config_path, source))
}

pub fn to_string(config: &LabelsConfig, config_path: &Path) -> Result<String, GitlaneError> {
    ::toml::to_string(&RawLabelsConfig::from(config)).map_err(|source| {
        GitlaneError::SerializeToml {
            path: config_path.to_path_buf(),
            source,
        }
    })
}

pub fn save_to_path(config_path: &Path, config: &LabelsConfig) -> Result<(), GitlaneError> {
    let content = to_string(config, config_path)?;
    write_text_file(config_path, &content)?;
    Ok(())
}
