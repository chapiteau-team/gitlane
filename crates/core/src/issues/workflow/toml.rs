use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    errors::GitlaneError,
    fs::{read_text_file, write_text_file},
};

use super::{StateId, TransitionId, WorkflowConfig, WorkflowState, WorkflowTransition};

#[derive(Debug, Deserialize, Serialize)]
struct RawWorkflowConfig {
    initial_state: StateId,
    states: BTreeMap<StateId, RawWorkflowState>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    transitions: BTreeMap<StateId, BTreeMap<TransitionId, RawWorkflowTransition>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawWorkflowState {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawWorkflowTransition {
    name: String,
    to: StateId,
}

impl TryFrom<RawWorkflowState> for WorkflowState {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawWorkflowState) -> Result<Self, Self::Error> {
        WorkflowState::new(raw.name)
    }
}

impl TryFrom<RawWorkflowTransition> for WorkflowTransition {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawWorkflowTransition) -> Result<Self, Self::Error> {
        WorkflowTransition::new(raw.name, raw.to)
    }
}

impl From<&WorkflowState> for RawWorkflowState {
    fn from(state: &WorkflowState) -> Self {
        Self {
            name: state.name().to_owned(),
        }
    }
}

impl From<&WorkflowTransition> for RawWorkflowTransition {
    fn from(transition: &WorkflowTransition) -> Self {
        Self {
            name: transition.name().to_owned(),
            to: transition.to().to_owned(),
        }
    }
}

impl TryFrom<RawWorkflowConfig> for WorkflowConfig {
    type Error = crate::errors::ConfigValidationError;

    fn try_from(raw: RawWorkflowConfig) -> Result<Self, Self::Error> {
        let states = raw
            .states
            .into_iter()
            .map(|(state_id, state)| state.try_into().map(|state| (state_id, state)))
            .collect::<Result<_, _>>()?;

        let transitions = raw
            .transitions
            .into_iter()
            .map(|(from_state, transitions)| {
                transitions
                    .into_iter()
                    .map(|(transition_id, transition)| {
                        transition
                            .try_into()
                            .map(|transition| (transition_id, transition))
                    })
                    .collect::<Result<_, _>>()
                    .map(|transitions| (from_state, transitions))
            })
            .collect::<Result<_, _>>()?;

        WorkflowConfig::new(raw.initial_state, states, transitions)
    }
}

impl From<&WorkflowConfig> for RawWorkflowConfig {
    fn from(config: &WorkflowConfig) -> Self {
        let states = config
            .states()
            .iter()
            .map(|(state_id, state)| (state_id.clone(), RawWorkflowState::from(state)))
            .collect();
        let transitions = config
            .transitions()
            .iter()
            .map(|(from_state, transitions)| {
                let transitions = transitions
                    .iter()
                    .map(|(transition_id, transition)| {
                        (
                            transition_id.clone(),
                            RawWorkflowTransition::from(transition),
                        )
                    })
                    .collect();
                (from_state.clone(), transitions)
            })
            .collect();

        Self {
            initial_state: config.initial_state().to_owned(),
            states,
            transitions,
        }
    }
}

pub fn load_from_path(workflow_path: &Path) -> Result<WorkflowConfig, GitlaneError> {
    let content = read_text_file(workflow_path)?;
    parse_str(&content, workflow_path)
}

pub fn parse_str(content: &str, workflow_path: &Path) -> Result<WorkflowConfig, GitlaneError> {
    let raw: RawWorkflowConfig =
        ::toml::from_str(content).map_err(|source| GitlaneError::ParseToml {
            path: workflow_path.to_path_buf(),
            source,
        })?;

    raw.try_into()
        .map_err(|source| GitlaneError::invalid_config(workflow_path, source))
}

pub fn to_string(config: &WorkflowConfig, workflow_path: &Path) -> Result<String, GitlaneError> {
    ::toml::to_string(&RawWorkflowConfig::from(config)).map_err(|source| {
        GitlaneError::SerializeToml {
            path: workflow_path.to_path_buf(),
            source,
        }
    })
}

pub fn save_to_path(workflow_path: &Path, config: &WorkflowConfig) -> Result<(), GitlaneError> {
    let content = to_string(config, workflow_path)?;
    write_text_file(workflow_path, &content)?;
    Ok(())
}
