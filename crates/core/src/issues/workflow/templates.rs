use std::collections::BTreeMap;

use crate::errors::ConfigValidationError;

use super::{WorkflowConfig, WorkflowState, WorkflowTransition};

pub(crate) fn default() -> Result<WorkflowConfig, ConfigValidationError> {
    WorkflowConfig::new(
        "todo".to_owned(),
        BTreeMap::from([
            ("done".to_owned(), WorkflowState::new("Done".to_owned())?),
            (
                "in_progress".to_owned(),
                WorkflowState::new("In Progress".to_owned())?,
            ),
            (
                "review".to_owned(),
                WorkflowState::new("In Review".to_owned())?,
            ),
            ("todo".to_owned(), WorkflowState::new("To Do".to_owned())?),
        ]),
        BTreeMap::from([
            (
                "in_progress".to_owned(),
                BTreeMap::from([(
                    "request_review".to_owned(),
                    WorkflowTransition::new("Request review".to_owned(), "review".to_owned())?,
                )]),
            ),
            (
                "review".to_owned(),
                BTreeMap::from([
                    (
                        "approve".to_owned(),
                        WorkflowTransition::new("Approve".to_owned(), "done".to_owned())?,
                    ),
                    (
                        "request_changes".to_owned(),
                        WorkflowTransition::new(
                            "Request changes".to_owned(),
                            "in_progress".to_owned(),
                        )?,
                    ),
                ]),
            ),
            (
                "todo".to_owned(),
                BTreeMap::from([(
                    "start_work".to_owned(),
                    WorkflowTransition::new("Start work".to_owned(), "in_progress".to_owned())?,
                )]),
            ),
        ]),
    )
}
