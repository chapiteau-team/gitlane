use std::{collections::BTreeMap, path::Path};

use crate::{
    codec,
    errors::{ConfigValidationError, GitlaneError},
    validate::{validate_id, validate_non_blank},
};

mod repr;
pub mod templates;

/// Stable identifier for a workflow state.
pub type StateId = String;
/// Stable identifier for a workflow transition.
pub type TransitionId = String;

/// Validated issue workflow configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowConfig {
    initial_state: StateId,
    states: BTreeMap<StateId, WorkflowState>,
    transitions: BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransition>>,
}

/// Validated workflow state definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowState {
    name: String,
}

/// Validated workflow transition definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowTransition {
    name: String,
    to: StateId,
}

impl WorkflowConfig {
    /// Builds validated workflow configuration.
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

    /// Loads this config from a supported file path.
    pub fn load(workflow_path: &Path) -> Result<Self, GitlaneError> {
        codec::load::<Self, repr::WorkflowConfigRepr>(workflow_path)
    }

    /// Saves this config using the format implied by `workflow_path`.
    pub fn save(&self, workflow_path: &Path) -> Result<(), GitlaneError> {
        codec::save::<Self, repr::WorkflowConfigRepr>(workflow_path, self)
    }

    /// Returns the initial state id.
    pub fn initial_state(&self) -> &str {
        &self.initial_state
    }

    /// Returns state ids in key order.
    pub fn state_ids(&self) -> impl Iterator<Item = &str> {
        self.states.keys().map(String::as_str)
    }

    /// Returns states keyed by state id.
    pub fn states(&self) -> &BTreeMap<StateId, WorkflowState> {
        &self.states
    }

    /// Returns transitions keyed by source state id.
    pub fn transitions(&self) -> &BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransition>> {
        &self.transitions
    }
}

impl WorkflowState {
    /// Builds a validated workflow state.
    pub fn new(name: String) -> Result<Self, ConfigValidationError> {
        validate_non_blank(&name, "workflow states must have a non-empty `name`")?;

        Ok(Self { name })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl WorkflowTransition {
    /// Builds a validated workflow transition.
    pub fn new(name: String, to: StateId) -> Result<Self, ConfigValidationError> {
        validate_non_blank(&name, "workflow transitions must have a non-empty `name`")?;

        validate_id(
            &to,
            "workflow transitions must target a non-empty state id",
            "workflow transitions must target a state id without leading or trailing whitespace",
        )?;

        Ok(Self { name, to })
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the destination state id.
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

        validate_non_blank(
            state.name(),
            format!("workflow state `{state_id}` must have a non-empty `name`"),
        )?;
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

            validate_non_blank(
                transition.name(),
                format!(
                    "workflow transition `{transition_id}` from `{from_state}` must have a non-empty `name`"
                ),
            )?;

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::Path};

    use crate::codec;
    use tempfile::TempDir;

    fn state(name: &str) -> WorkflowState {
        WorkflowState::new(name.to_owned()).expect("workflow state should be valid")
    }

    fn transition(name: &str, to: &str) -> WorkflowTransition {
        WorkflowTransition::new(name.to_owned(), to.to_owned())
            .expect("workflow transition should be valid")
    }

    fn parse_workflow_config(content: &str) -> Result<WorkflowConfig, GitlaneError> {
        codec::parse::<WorkflowConfig, super::repr::WorkflowConfigRepr>(
            content,
            Path::new("workflow.toml"),
        )
    }

    fn load_workflow_config(content: &str) -> Result<WorkflowConfig, GitlaneError> {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.toml");
        fs::write(&workflow_path, content).expect("workflow config should be written");
        WorkflowConfig::load(&workflow_path)
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

        workflow.save(&workflow_path).expect("workflow should save");
        let loaded =
            WorkflowConfig::load(&workflow_path).expect("workflow should load after saving");

        assert_eq!(loaded, workflow);
    }

    #[test]
    fn loads_yaml_workflow() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.yaml");
        fs::write(
            &workflow_path,
            concat!(
                "initial_state: todo\n",
                "states:\n",
                "  todo:\n",
                "    name: To Do\n",
                "  done:\n",
                "    name: Done\n",
                "transitions:\n",
                "  todo:\n",
                "    finish:\n",
                "      name: Finish\n",
                "      to: done\n"
            ),
        )
        .expect("yaml workflow should be written");

        let workflow = WorkflowConfig::load(&workflow_path).expect("yaml workflow should load");

        assert_eq!(workflow.state_ids().collect::<Vec<_>>(), ["done", "todo"]);
    }

    #[test]
    fn saves_and_loads_yaml_workflow() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.yaml");
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
            .save(&workflow_path)
            .expect("yaml workflow should save");
        let loaded =
            WorkflowConfig::load(&workflow_path).expect("yaml workflow should load after saving");

        assert_eq!(loaded, workflow);
    }

    #[test]
    fn loads_yml_workflow() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.yml");
        fs::write(
            &workflow_path,
            concat!(
                "initial_state: todo\n",
                "states:\n",
                "  todo:\n",
                "    name: To Do\n"
            ),
        )
        .expect("yml workflow should be written");

        let workflow = WorkflowConfig::load(&workflow_path).expect("yml workflow should load");

        assert_eq!(workflow.state_ids().collect::<Vec<_>>(), ["todo"]);
    }

    #[test]
    fn saves_and_loads_yml_workflow() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let workflow_path = temp_dir.path().join("workflow.yml");
        let workflow = WorkflowConfig::new(
            "todo".to_owned(),
            BTreeMap::from([("todo".to_owned(), state("To Do"))]),
            BTreeMap::new(),
        )
        .expect("workflow should be valid");

        workflow
            .save(&workflow_path)
            .expect("yml workflow should save");
        let loaded =
            WorkflowConfig::load(&workflow_path).expect("yml workflow should load after saving");

        assert_eq!(loaded, workflow);
    }
}
