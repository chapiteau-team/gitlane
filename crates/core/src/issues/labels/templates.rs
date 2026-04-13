use std::collections::BTreeMap;

use crate::config::ConfigValidationError;

use super::{Label, LabelGroup, LabelsConfig};

/// Builds the default labels config scaffold.
pub fn default() -> Result<LabelsConfig, ConfigValidationError> {
    LabelsConfig::new(
        BTreeMap::from([(
            "type".to_owned(),
            LabelGroup::new(
                "Type".to_owned(),
                Some("Issue classification".to_owned()),
                Some("#334155".to_owned()),
            )?,
        )]),
        BTreeMap::from([
            (
                "blocked".to_owned(),
                Label::new(
                    "Blocked".to_owned(),
                    Some("Waiting on external dependency".to_owned()),
                    Some("#b91c1c".to_owned()),
                    None,
                )?,
            ),
            (
                "good_first_issue".to_owned(),
                Label::new(
                    "Good First Issue".to_owned(),
                    Some("Suitable for new contributors".to_owned()),
                    Some("#0369a1".to_owned()),
                    None,
                )?,
            ),
            (
                "needs_decision".to_owned(),
                Label::new(
                    "Needs Decision".to_owned(),
                    Some("Requires product or technical decision".to_owned()),
                    Some("#b45309".to_owned()),
                    None,
                )?,
            ),
            (
                "type_bug".to_owned(),
                Label::new(
                    "Bug".to_owned(),
                    Some("Unexpected behavior".to_owned()),
                    None,
                    Some("type".to_owned()),
                )?,
            ),
            (
                "type_chore".to_owned(),
                Label::new(
                    "Chore".to_owned(),
                    Some("Maintenance and tooling work".to_owned()),
                    None,
                    Some("type".to_owned()),
                )?,
            ),
            (
                "type_docs".to_owned(),
                Label::new(
                    "Docs".to_owned(),
                    Some("Documentation updates".to_owned()),
                    None,
                    Some("type".to_owned()),
                )?,
            ),
            (
                "type_feature".to_owned(),
                Label::new(
                    "Feature".to_owned(),
                    Some("Net-new capability".to_owned()),
                    None,
                    Some("type".to_owned()),
                )?,
            ),
            (
                "type_refactor".to_owned(),
                Label::new(
                    "Refactor".to_owned(),
                    Some("Internal structure improvements".to_owned()),
                    None,
                    Some("type".to_owned()),
                )?,
            ),
        ]),
    )
}
