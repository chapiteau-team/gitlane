use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    codec::{TomlFormat, inline_child_tables, table_mut},
    errors::ConfigValidationError,
};

use super::{IssuePriority, IssuesConfig, PriorityId};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct IssuesConfigRepr {
    pub(super) issue_prefix: String,
    pub(super) priorities: BTreeMap<PriorityId, IssuePriorityRepr>,
    pub(super) priority_order: Vec<PriorityId>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct IssuePriorityRepr {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,
}

impl TryFrom<IssuePriorityRepr> for IssuePriority {
    type Error = ConfigValidationError;

    fn try_from(repr: IssuePriorityRepr) -> Result<Self, Self::Error> {
        IssuePriority::new(repr.name, repr.description)
    }
}

impl From<&IssuePriority> for IssuePriorityRepr {
    fn from(priority: &IssuePriority) -> Self {
        Self {
            name: priority.name().to_owned(),
            description: priority.description().map(ToOwned::to_owned),
        }
    }
}

impl TryFrom<IssuesConfigRepr> for IssuesConfig {
    type Error = ConfigValidationError;

    fn try_from(repr: IssuesConfigRepr) -> Result<Self, Self::Error> {
        let priorities = repr
            .priorities
            .into_iter()
            .map(|(priority_id, priority)| {
                priority.try_into().map(|priority| (priority_id, priority))
            })
            .collect::<Result<_, _>>()?;

        IssuesConfig::new(repr.issue_prefix, priorities, repr.priority_order)
    }
}

impl From<&IssuesConfig> for IssuesConfigRepr {
    fn from(config: &IssuesConfig) -> Self {
        let priorities = config
            .priorities()
            .iter()
            .map(|(priority_id, priority)| (priority_id.clone(), IssuePriorityRepr::from(priority)))
            .collect();

        Self {
            issue_prefix: config.issue_prefix().to_owned(),
            priorities,
            priority_order: config.priority_order().to_vec(),
        }
    }
}

impl TomlFormat for IssuesConfigRepr {
    fn format_toml_document(&self, document: &mut toml_edit::DocumentMut) {
        if let Some(priorities) = table_mut(document, "priorities") {
            inline_child_tables(priorities);
        }
    }
}
