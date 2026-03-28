use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    codec::{TomlFormat, inline_child_tables, table_mut},
    errors::ConfigValidationError,
};

use super::{Label, LabelGroup, LabelGroupId, LabelId, LabelsConfig};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct LabelsConfigRepr {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(super) label_groups: BTreeMap<LabelGroupId, LabelGroupRepr>,
    pub(super) labels: BTreeMap<LabelId, LabelRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct LabelGroupRepr {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct LabelRepr {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) group: Option<LabelGroupId>,
}

impl TryFrom<LabelGroupRepr> for LabelGroup {
    type Error = ConfigValidationError;

    fn try_from(repr: LabelGroupRepr) -> Result<Self, Self::Error> {
        LabelGroup::new(repr.name, repr.description, repr.color)
    }
}

impl TryFrom<LabelRepr> for Label {
    type Error = ConfigValidationError;

    fn try_from(repr: LabelRepr) -> Result<Self, Self::Error> {
        Label::new(repr.name, repr.description, repr.color, repr.group)
    }
}

impl From<&LabelGroup> for LabelGroupRepr {
    fn from(group: &LabelGroup) -> Self {
        Self {
            name: group.name().to_owned(),
            description: group.description().map(ToOwned::to_owned),
            color: group.color().map(ToOwned::to_owned),
        }
    }
}

impl From<&Label> for LabelRepr {
    fn from(label: &Label) -> Self {
        Self {
            name: label.name().to_owned(),
            description: label.description().map(ToOwned::to_owned),
            color: label.color().map(ToOwned::to_owned),
            group: label.group().map(ToOwned::to_owned),
        }
    }
}

impl TryFrom<LabelsConfigRepr> for LabelsConfig {
    type Error = ConfigValidationError;

    fn try_from(repr: LabelsConfigRepr) -> Result<Self, Self::Error> {
        let label_groups = repr
            .label_groups
            .into_iter()
            .map(|(group_id, group)| group.try_into().map(|group| (group_id, group)))
            .collect::<Result<_, _>>()?;
        let labels = repr
            .labels
            .into_iter()
            .map(|(label_id, label)| label.try_into().map(|label| (label_id, label)))
            .collect::<Result<_, _>>()?;

        LabelsConfig::new(label_groups, labels)
    }
}

impl From<&LabelsConfig> for LabelsConfigRepr {
    fn from(config: &LabelsConfig) -> Self {
        let label_groups = config
            .label_groups()
            .iter()
            .map(|(group_id, group)| (group_id.clone(), LabelGroupRepr::from(group)))
            .collect();
        let labels = config
            .labels()
            .iter()
            .map(|(label_id, label)| (label_id.clone(), LabelRepr::from(label)))
            .collect();

        Self {
            label_groups,
            labels,
        }
    }
}

impl TomlFormat for LabelsConfigRepr {
    fn format_toml_document(&self, document: &mut toml_edit::DocumentMut) {
        if let Some(label_groups) = table_mut(document, "label_groups") {
            inline_child_tables(label_groups);
        }

        if let Some(labels) = table_mut(document, "labels") {
            inline_child_tables(labels);
        }
    }
}
