use std::{collections::BTreeMap, path::Path};

use serde::Deserialize;

use crate::{errors::GitlaneError, fs::read_text_file};

type StateId = String;
type TransitionId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowConfig {
    state_ids: Vec<StateId>,
}

impl WorkflowConfig {
    pub(crate) fn load_from_path(workflow_path: &Path) -> Result<Self, GitlaneError> {
        let content = read_text_file(workflow_path)?;
        let raw: RawWorkflowConfig =
            toml::from_str(&content).map_err(|source| GitlaneError::ParseToml {
                path: workflow_path.to_path_buf(),
                source,
            })?;

        Self::from_raw(raw, workflow_path)
    }

    fn from_raw(raw: RawWorkflowConfig, workflow_path: &Path) -> Result<Self, GitlaneError> {
        let RawWorkflowConfig {
            initial_state,
            states,
            transitions,
        } = raw;

        validate_states(&states, workflow_path)?;
        validate_initial_state(&initial_state, &states, workflow_path)?;

        validate_transitions(&states, &transitions, workflow_path)?;

        Ok(Self {
            state_ids: states.into_keys().collect(),
        })
    }

    pub(crate) fn state_ids(&self) -> impl Iterator<Item = &str> {
        self.state_ids.iter().map(String::as_str)
    }
}

#[derive(Debug, Deserialize)]
struct RawWorkflowConfig {
    initial_state: StateId,
    states: BTreeMap<StateId, RawWorkflowState>,
    #[serde(default)]
    transitions: BTreeMap<StateId, BTreeMap<TransitionId, RawWorkflowTransition>>,
}

#[derive(Debug, Deserialize)]
struct RawWorkflowState {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawWorkflowTransition {
    name: String,
    to: StateId,
}

fn validate_initial_state(
    initial_state: &StateId,
    states: &BTreeMap<StateId, RawWorkflowState>,
    workflow_path: &Path,
) -> Result<(), GitlaneError> {
    validate_id(
        initial_state,
        "`initial_state` must be a non-empty state id",
        "`initial_state` must not have leading or trailing whitespace",
        workflow_path,
    )?;

    if !states.contains_key(initial_state) {
        return Err(invalid_config_error(
            workflow_path,
            format!("`initial_state` references unknown state `{initial_state}`"),
        ));
    }

    Ok(())
}

fn validate_states(
    states: &BTreeMap<StateId, RawWorkflowState>,
    workflow_path: &Path,
) -> Result<(), GitlaneError> {
    if states.is_empty() {
        return Err(invalid_config_error(
            workflow_path,
            "`states` must declare at least one state",
        ));
    }

    for (state_id, state) in states {
        validate_id(
            state_id,
            "workflow state ids must be non-empty",
            "workflow state ids must not have leading or trailing whitespace",
            workflow_path,
        )?;

        if state.name.trim().is_empty() {
            return Err(invalid_config_error(
                workflow_path,
                format!("workflow state `{state_id}` must have a non-empty `name`"),
            ));
        }
    }

    Ok(())
}

fn validate_transitions(
    states: &BTreeMap<StateId, RawWorkflowState>,
    transitions: &BTreeMap<StateId, BTreeMap<TransitionId, RawWorkflowTransition>>,
    workflow_path: &Path,
) -> Result<(), GitlaneError> {
    for (from_state, transitions) in transitions {
        validate_id(
            from_state,
            "transition source ids must be non-empty",
            "transition source ids must not have leading or trailing whitespace",
            workflow_path,
        )?;

        if !states.contains_key(from_state) {
            return Err(invalid_config_error(
                workflow_path,
                format!("transition source `{from_state}` is not declared in `[states]`"),
            ));
        }

        for (transition_id, transition) in transitions {
            validate_id(
                transition_id,
                format!("workflow transitions from `{from_state}` must use non-empty ids"),
                format!(
                    "workflow transitions from `{from_state}` must not use ids with leading or trailing whitespace"
                ),
                workflow_path,
            )?;

            if transition.name.trim().is_empty() {
                return Err(invalid_config_error(
                    workflow_path,
                    format!(
                        "workflow transition `{transition_id}` from `{from_state}` must have a non-empty `name`"
                    ),
                ));
            }

            validate_id(
                &transition.to,
                format!(
                    "workflow transition `{transition_id}` from `{from_state}` must target a non-empty state id"
                ),
                format!(
                    "workflow transition `{transition_id}` from `{from_state}` must target a state id without leading or trailing whitespace"
                ),
                workflow_path,
            )?;

            if !states.contains_key(&transition.to) {
                return Err(invalid_config_error(
                    workflow_path,
                    format!(
                        "workflow transition `{transition_id}` from `{from_state}` targets unknown state `{}`",
                        transition.to
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn validate_id(
    id: &str,
    empty_message: impl Into<String>,
    whitespace_message: impl Into<String>,
    workflow_path: &Path,
) -> Result<(), GitlaneError> {
    if id.is_empty() {
        return Err(invalid_config_error(workflow_path, empty_message));
    }

    if id.trim() != id {
        return Err(invalid_config_error(workflow_path, whitespace_message));
    }

    Ok(())
}

fn invalid_config_error(workflow_path: &Path, message: impl Into<String>) -> GitlaneError {
    GitlaneError::InvalidConfig {
        path: workflow_path.to_path_buf(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile::TempDir;

    fn parse_workflow_config(content: &str) -> Result<WorkflowConfig, GitlaneError> {
        let raw: RawWorkflowConfig =
            toml::from_str(content).expect("test workflow config snippets should be valid TOML");
        WorkflowConfig::from_raw(raw, Path::new("workflow.toml"))
    }

    fn load_workflow_config(content: &str) -> Result<WorkflowConfig, GitlaneError> {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.toml");
        fs::write(&workflow_path, content).expect("workflow config should be written");
        WorkflowConfig::load_from_path(&workflow_path)
    }

    #[test]
    fn loads_valid_workflow_config() {
        let workflow = load_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
finish = { name = "Finish", to = "done" }
"#,
        )
        .expect("workflow config should load");

        assert_eq!(workflow.state_ids().collect::<Vec<_>>(), ["done", "todo"]);
    }

    #[test]
    fn accepts_workflow_without_transitions() {
        let workflow = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }
"#,
        )
        .expect("workflow without transitions should load");

        assert_eq!(workflow.state_ids().collect::<Vec<_>>(), ["done", "todo"]);
    }

    #[test]
    fn rejects_empty_initial_state() {
        let err = parse_workflow_config(
            r#"
initial_state = ""

[states]
todo = { name = "To Do" }
"#,
        )
        .expect_err("empty initial state should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_initial_state_with_surrounding_whitespace() {
        let err = parse_workflow_config(
            r#"
initial_state = " todo "

[states]
todo = { name = "To Do" }
"#,
        )
        .expect_err("initial state with surrounding whitespace should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_unknown_initial_state() {
        let err = parse_workflow_config(
            r#"
initial_state = "review"

[states]
todo = { name = "To Do" }
"#,
        )
        .expect_err("unknown initial state should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_empty_states_table() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
"#,
        )
        .expect_err("empty states table should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_state_id_with_surrounding_whitespace() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
" todo " = { name = "To Do" }
done = { name = "Done" }
"#,
        )
        .expect_err("state id with surrounding whitespace should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_empty_state_name() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "   " }
"#,
        )
        .expect_err("empty state name should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_transition_source_with_surrounding_whitespace() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions." todo "]
finish = { name = "Finish", to = "done" }
"#,
        )
        .expect_err("transition source with surrounding whitespace should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_unknown_transition_source() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.review]
finish = { name = "Finish", to = "done" }
"#,
        )
        .expect_err("unknown transition source should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_empty_transition_id() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
"" = { name = "Finish", to = "done" }
"#,
        )
        .expect_err("empty transition id should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_transition_id_with_surrounding_whitespace() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
" finish " = { name = "Finish", to = "done" }
"#,
        )
        .expect_err("transition id with surrounding whitespace should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_empty_transition_name() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
finish = { name = "   ", to = "done" }
"#,
        )
        .expect_err("empty transition name should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_empty_transition_target() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
finish = { name = "Finish", to = "" }
"#,
        )
        .expect_err("empty transition target should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_transition_target_with_surrounding_whitespace() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
finish = { name = "Finish", to = " done " }
"#,
        )
        .expect_err("transition target with surrounding whitespace should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }

    #[test]
    fn rejects_unknown_transition_target() {
        let err = parse_workflow_config(
            r#"
initial_state = "todo"

[states]
todo = { name = "To Do" }
done = { name = "Done" }

[transitions.todo]
finish = { name = "Finish", to = "review" }
"#,
        )
        .expect_err("unknown transition target should fail");

        assert!(matches!(err, GitlaneError::InvalidConfig { .. }));
    }
}
