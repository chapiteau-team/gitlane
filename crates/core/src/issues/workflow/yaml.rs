use std::path::Path;

use crate::{
    codec::{load_config_from_path, parse_config, save_yaml_config},
    errors::GitlaneError,
};

use super::{WorkflowConfig, codec::WorkflowConfigRepr};

pub fn load_from_path(workflow_path: &Path) -> Result<WorkflowConfig, GitlaneError> {
    load_config_from_path(workflow_path, parse_str)
}

pub fn parse_str(content: &str, workflow_path: &Path) -> Result<WorkflowConfig, GitlaneError> {
    parse_config(content, workflow_path, |content| {
        serde_yaml::from_str::<WorkflowConfigRepr>(content)
    })
}

pub fn save_to_path(workflow_path: &Path, config: &WorkflowConfig) -> Result<(), GitlaneError> {
    save_yaml_config(workflow_path, &WorkflowConfigRepr::from(config))
}
