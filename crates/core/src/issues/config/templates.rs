use std::collections::BTreeMap;

use crate::errors::ConfigValidationError;

use super::{IssuePriority, IssuesConfig};

/// Builds the default issue config scaffold.
pub fn default() -> Result<IssuesConfig, ConfigValidationError> {
    IssuesConfig::new(
        "ISS".to_owned(),
        BTreeMap::from([
            (
                "p0".to_owned(),
                IssuePriority::new(
                    "No Priority".to_owned(),
                    Some("Default when urgency is not assigned".to_owned()),
                )?,
            ),
            (
                "p1".to_owned(),
                IssuePriority::new(
                    "Urgent".to_owned(),
                    Some("Needs immediate attention".to_owned()),
                )?,
            ),
            (
                "p2".to_owned(),
                IssuePriority::new(
                    "High".to_owned(),
                    Some("Important and should be scheduled soon".to_owned()),
                )?,
            ),
            (
                "p3".to_owned(),
                IssuePriority::new("Medium".to_owned(), Some("Normal planned work".to_owned()))?,
            ),
            (
                "p4".to_owned(),
                IssuePriority::new("Low".to_owned(), Some("Can be deferred".to_owned()))?,
            ),
        ]),
        vec![
            "p1".to_owned(),
            "p2".to_owned(),
            "p3".to_owned(),
            "p4".to_owned(),
            "p0".to_owned(),
        ],
    )
}
