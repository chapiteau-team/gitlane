use serde::{Deserialize, Serialize};

use crate::{
    frontmatter::FrontmatterSerializeError,
    issues::{config::PriorityId, labels::LabelId},
};

use super::{IssueMetadata, IssueValidationError, format_utc_timestamp, parse_utc_timestamp};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct IssueMetadataRepr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reporter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) assignees: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) priority: Option<PriorityId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) labels: Option<Vec<LabelId>>,
}

impl IssueMetadataRepr {
    pub(crate) fn from_metadata(
        metadata: &IssueMetadata,
    ) -> Result<Self, FrontmatterSerializeError> {
        Ok(Self {
            title: Some(metadata.title().to_owned()),
            created_at: Some(format_utc_timestamp(metadata.created_at())?),
            updated_at: Some(format_utc_timestamp(metadata.updated_at())?),
            reporter: Some(metadata.reporter().to_owned()),
            assignees: Some(metadata.assignees().to_vec()),
            priority: Some(metadata.priority().to_owned()),
            labels: Some(metadata.labels().to_vec()),
        })
    }
}

impl TryFrom<IssueMetadataRepr> for IssueMetadata {
    type Error = IssueValidationError;

    fn try_from(repr: IssueMetadataRepr) -> Result<Self, Self::Error> {
        IssueMetadata::new(
            require_field(repr.title, "title")?,
            parse_utc_timestamp(require_field(repr.created_at, "created_at")?, "created_at")?,
            parse_utc_timestamp(require_field(repr.updated_at, "updated_at")?, "updated_at")?,
            require_field(repr.reporter, "reporter")?,
            require_field(repr.assignees, "assignees")?,
            require_field(repr.priority, "priority")?,
            require_field(repr.labels, "labels")?,
        )
    }
}

fn require_field<T>(value: Option<T>, field_name: &str) -> Result<T, IssueValidationError> {
    value.ok_or_else(|| {
        IssueValidationError::new(format!("issue metadata must include `{field_name}`"))
    })
}
