use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    codec::{TomlFormat, ensure_table, inline_child_tables, table_mut},
    errors::ConfigValidationError,
};

use super::{StateId, TransitionId, WorkflowConfig, WorkflowState, WorkflowTransition};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct WorkflowConfigRepr {
    pub(super) initial_state: StateId,
    pub(super) states: BTreeMap<StateId, WorkflowStateRepr>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(super) transitions: BTreeMap<StateId, BTreeMap<TransitionId, WorkflowTransitionRepr>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct WorkflowStateRepr {
    pub(super) name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct WorkflowTransitionRepr {
    pub(super) name: String,
    pub(super) to: StateId,
}

impl TryFrom<WorkflowStateRepr> for WorkflowState {
    type Error = ConfigValidationError;

    fn try_from(repr: WorkflowStateRepr) -> Result<Self, Self::Error> {
        WorkflowState::new(repr.name)
    }
}

impl TryFrom<WorkflowTransitionRepr> for WorkflowTransition {
    type Error = ConfigValidationError;

    fn try_from(repr: WorkflowTransitionRepr) -> Result<Self, Self::Error> {
        WorkflowTransition::new(repr.name, repr.to)
    }
}

impl From<&WorkflowState> for WorkflowStateRepr {
    fn from(state: &WorkflowState) -> Self {
        Self {
            name: state.name().to_owned(),
        }
    }
}

impl From<&WorkflowTransition> for WorkflowTransitionRepr {
    fn from(transition: &WorkflowTransition) -> Self {
        Self {
            name: transition.name().to_owned(),
            to: transition.to().to_owned(),
        }
    }
}

impl TryFrom<WorkflowConfigRepr> for WorkflowConfig {
    type Error = ConfigValidationError;

    fn try_from(repr: WorkflowConfigRepr) -> Result<Self, Self::Error> {
        let states = repr
            .states
            .into_iter()
            .map(|(state_id, state)| state.try_into().map(|state| (state_id, state)))
            .collect::<Result<_, _>>()?;

        let transitions = repr
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

        WorkflowConfig::new(repr.initial_state, states, transitions)
    }
}

impl From<&WorkflowConfig> for WorkflowConfigRepr {
    fn from(config: &WorkflowConfig) -> Self {
        let states = config
            .states()
            .iter()
            .map(|(state_id, state)| (state_id.clone(), WorkflowStateRepr::from(state)))
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
                            WorkflowTransitionRepr::from(transition),
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

impl TomlFormat for WorkflowConfigRepr {
    fn format_toml_document(&self, document: &mut toml_edit::DocumentMut) {
        if let Some(states) = table_mut(document, "states") {
            inline_child_tables(states);
        }

        if let Some(transitions) = table_mut(document, "transitions") {
            for (_, transition_group) in transitions.iter_mut() {
                if let Some(transition_group) = ensure_table(transition_group) {
                    inline_child_tables(transition_group);
                }
            }

            transitions.set_implicit(true);
        }
    }
}
