use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{errors::GitlaneError, fs::write_text_file};

#[cfg(test)]
use crate::fs::read_text_file;

use super::{IssuePriority, IssuesConfig, PriorityId};

#[derive(Debug, Deserialize, Serialize)]
struct RawIssuesConfig {
    issue_prefix: String,
    priorities: BTreeMap<PriorityId, RawIssuePriority>,
    priority_order: Vec<PriorityId>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawIssuePriority {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl TryFrom<RawIssuePriority> for IssuePriority {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawIssuePriority) -> Result<Self, Self::Error> {
        IssuePriority::new(raw.name, raw.description)
    }
}

impl From<&IssuePriority> for RawIssuePriority {
    fn from(priority: &IssuePriority) -> Self {
        Self {
            name: priority.name().to_owned(),
            description: priority.description().map(ToOwned::to_owned),
        }
    }
}

impl TryFrom<RawIssuesConfig> for IssuesConfig {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawIssuesConfig) -> Result<Self, Self::Error> {
        let priorities = raw
            .priorities
            .into_iter()
            .map(|(priority_id, priority)| {
                priority.try_into().map(|priority| (priority_id, priority))
            })
            .collect::<Result<_, _>>()?;

        IssuesConfig::new(raw.issue_prefix, priorities, raw.priority_order)
    }
}

impl From<&IssuesConfig> for RawIssuesConfig {
    fn from(config: &IssuesConfig) -> Self {
        let priorities = config
            .priorities()
            .iter()
            .map(|(priority_id, priority)| (priority_id.clone(), RawIssuePriority::from(priority)))
            .collect();

        Self {
            issue_prefix: config.issue_prefix().to_owned(),
            priorities,
            priority_order: config.priority_order().to_vec(),
        }
    }
}

#[cfg(test)]
pub(crate) fn load_from_path(config_path: &Path) -> Result<IssuesConfig, GitlaneError> {
    let content = read_text_file(config_path)?;
    parse_str(&content, config_path)
}

#[cfg(test)]
pub(crate) fn parse_str(content: &str, config_path: &Path) -> Result<IssuesConfig, GitlaneError> {
    let raw: RawIssuesConfig =
        ::toml::from_str(content).map_err(|source| GitlaneError::ParseToml {
            path: config_path.to_path_buf(),
            source,
        })?;

    raw.try_into()
        .map_err(|source| GitlaneError::invalid_config(config_path, source))
}

pub(crate) fn to_string(config: &IssuesConfig, config_path: &Path) -> Result<String, GitlaneError> {
    ::toml::to_string(&RawIssuesConfig::from(config)).map_err(|source| {
        GitlaneError::SerializeToml {
            path: config_path.to_path_buf(),
            source,
        }
    })
}

pub(crate) fn save_to_path(config_path: &Path, config: &IssuesConfig) -> Result<(), GitlaneError> {
    let content = to_string(config, config_path)?;
    write_text_file(config_path, &content)?;
    Ok(())
}
