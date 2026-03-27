use std::{collections::BTreeMap, path::Path};

use crate::errors::{ConfigValidationError, GitlaneError};

pub mod templates;
pub mod toml;

pub type StateId = String;
pub type TransitionId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowConfig {
    initial_state: StateId,
    states: BTreeMap<StateId, WorkflowState>,
    transitions: BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransition>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowState {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowTransition {
    name: String,
    to: StateId,
}

impl WorkflowConfig {
    pub fn new(
        initial_state: StateId,
        states: BTreeMap<StateId, WorkflowState>,
        transitions: BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransition>>,
    ) -> Result<Self, ConfigValidationError> {
        validate_states(&states)?;
        validate_initial_state(&initial_state, &states)?;
        validate_transitions(&states, &transitions)?;

        Ok(Self {
            initial_state,
            states,
            transitions,
        })
    }

    pub fn load_from_path(workflow_path: &Path) -> Result<Self, GitlaneError> {
        toml::load_from_path(workflow_path)
    }

    pub fn save_to_path(&self, workflow_path: &Path) -> Result<(), GitlaneError> {
        toml::save_to_path(workflow_path, self)
    }

    pub fn initial_state(&self) -> &str {
        &self.initial_state
    }

    pub fn state_ids(&self) -> impl Iterator<Item = &str> {
        self.states.keys().map(String::as_str)
    }

    pub fn states(&self) -> &BTreeMap<StateId, WorkflowState> {
        &self.states
    }

    pub fn transitions(&self) -> &BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransition>> {
        &self.transitions
    }
}

impl WorkflowState {
    pub fn new(name: String) -> Result<Self, ConfigValidationError> {
        if name.trim().is_empty() {
            return Err(ConfigValidationError::new(
                "workflow states must have a non-empty `name`",
            ));
        }

        Ok(Self { name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl WorkflowTransition {
    pub fn new(name: String, to: StateId) -> Result<Self, ConfigValidationError> {
        if name.trim().is_empty() {
            return Err(ConfigValidationError::new(
                "workflow transitions must have a non-empty `name`",
            ));
        }

        validate_id(
            &to,
            "workflow transitions must target a non-empty state id",
            "workflow transitions must target a state id without leading or trailing whitespace",
        )?;

        Ok(Self { name, to })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn to(&self) -> &str {
        &self.to
    }
}

fn validate_initial_state(
    initial_state: &StateId,
    states: &BTreeMap<StateId, WorkflowState>,
) -> Result<(), ConfigValidationError> {
    validate_id(
        initial_state,
        "`initial_state` must be a non-empty state id",
        "`initial_state` must not have leading or trailing whitespace",
    )?;

    if !states.contains_key(initial_state) {
        return Err(ConfigValidationError::new(format!(
            "`initial_state` references unknown state `{initial_state}`"
        )));
    }

    Ok(())
}

fn validate_states(states: &BTreeMap<StateId, WorkflowState>) -> Result<(), ConfigValidationError> {
    if states.is_empty() {
        return Err(ConfigValidationError::new(
            "`states` must declare at least one state",
        ));
    }

    for (state_id, state) in states {
        validate_id(
            state_id,
            "workflow state ids must be non-empty",
            "workflow state ids must not have leading or trailing whitespace",
        )?;

        if state.name().trim().is_empty() {
            return Err(ConfigValidationError::new(format!(
                "workflow state `{state_id}` must have a non-empty `name`"
            )));
        }
    }

    Ok(())
}

fn validate_transitions(
    states: &BTreeMap<StateId, WorkflowState>,
    transitions: &BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransition>>,
) -> Result<(), ConfigValidationError> {
    for (from_state, transitions) in transitions {
        validate_id(
            from_state,
            "transition source ids must be non-empty",
            "transition source ids must not have leading or trailing whitespace",
        )?;

        if !states.contains_key(from_state) {
            return Err(ConfigValidationError::new(format!(
                "transition source `{from_state}` is not declared in `[states]`"
            )));
        }

        for (transition_id, transition) in transitions {
            validate_id(
                transition_id,
                format!("workflow transitions from `{from_state}` must use non-empty ids"),
                format!(
                    "workflow transitions from `{from_state}` must not use ids with leading or trailing whitespace"
                ),
            )?;

            if transition.name().trim().is_empty() {
                return Err(ConfigValidationError::new(format!(
                    "workflow transition `{transition_id}` from `{from_state}` must have a non-empty `name`"
                )));
            }

            if !states.contains_key(transition.to()) {
                return Err(ConfigValidationError::new(format!(
                    "workflow transition `{transition_id}` from `{from_state}` targets unknown state `{}`",
                    transition.to()
                )));
            }
        }
    }

    Ok(())
}

fn validate_id(
    id: &str,
    empty_message: impl Into<String>,
    whitespace_message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    if id.is_empty() {
        return Err(ConfigValidationError::new(empty_message));
    }

    if id.trim() != id {
        return Err(ConfigValidationError::new(whitespace_message));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile::TempDir;

    fn state(name: &str) -> WorkflowState {
        WorkflowState::new(name.to_owned()).expect("workflow state should be valid")
    }

    fn transition(name: &str, to: &str) -> WorkflowTransition {
        WorkflowTransition::new(name.to_owned(), to.to_owned())
            .expect("workflow transition should be valid")
    }

    fn parse_workflow_config(content: &str) -> Result<WorkflowConfig, GitlaneError> {
        toml::parse_str(content, Path::new("workflow.toml"))
    }

    fn load_workflow_config(content: &str) -> Result<WorkflowConfig, GitlaneError> {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.toml");
        fs::write(&workflow_path, content).expect("workflow config should be written");
        WorkflowConfig::load_from_path(&workflow_path)
    }

    #[test]
    fn builds_valid_workflow_config() {
        let workflow = WorkflowConfig::new(
            "todo".to_owned(),
            BTreeMap::from([
                ("done".to_owned(), state("Done")),
                ("todo".to_owned(), state("To Do")),
            ]),
            BTreeMap::from([(
                "todo".to_owned(),
                BTreeMap::from([("finish".to_owned(), transition("Finish", "done"))]),
            )]),
        )
        .expect("workflow config should build");

        assert_eq!(workflow.initial_state(), "todo");
        assert_eq!(workflow.state_ids().collect::<Vec<_>>(), ["done", "todo"]);
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
    fn default_template_builds_valid_workflow() {
        let workflow = templates::default().expect("default workflow template should build");

        assert_eq!(workflow.initial_state(), "todo");
        assert_eq!(
            workflow.state_ids().collect::<Vec<_>>(),
            ["done", "in_progress", "review", "todo"]
        );
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

    #[test]
    fn saves_and_loads_toml_workflow() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.toml");
        let workflow = WorkflowConfig::new(
            "todo".to_owned(),
            BTreeMap::from([
                ("done".to_owned(), state("Done")),
                ("todo".to_owned(), state("To Do")),
            ]),
            BTreeMap::from([(
                "todo".to_owned(),
                BTreeMap::from([("finish".to_owned(), transition("Finish", "done"))]),
            )]),
        )
        .expect("workflow should be valid");

        workflow
            .save_to_path(&workflow_path)
            .expect("workflow should save");
        let loaded = WorkflowConfig::load_from_path(&workflow_path)
            .expect("workflow should load after saving");

        assert_eq!(loaded, workflow);
    }
}
