use serde::{Deserialize, Serialize};

use crate::errors::ConfigValidationError;

use super::ProjectConfig;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct ProjectConfigRepr {
    pub(super) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) people: Vec<String>,
}

impl TryFrom<ProjectConfigRepr> for ProjectConfig {
    type Error = ConfigValidationError;

    fn try_from(repr: ProjectConfigRepr) -> Result<Self, Self::Error> {
        ProjectConfig::new(repr.name, repr.description, repr.homepage, repr.people)
    }
}

impl From<&ProjectConfig> for ProjectConfigRepr {
    fn from(config: &ProjectConfig) -> Self {
        Self {
            name: config.name().to_owned(),
            description: config.description().map(ToOwned::to_owned),
            homepage: config.homepage().map(ToOwned::to_owned),
            people: config.people().to_vec(),
        }
    }
}
