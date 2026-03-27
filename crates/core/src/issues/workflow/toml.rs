use std::path::Path;

use crate::{
    config::{load_config_from_path, parse_config, save_toml_config, to_toml_string},
    errors::GitlaneError,
};

use super::{WorkflowConfig, codec::WorkflowConfigRepr};

pub fn load_from_path(workflow_path: &Path) -> Result<WorkflowConfig, GitlaneError> {
    load_config_from_path(workflow_path, parse_str)
}

pub fn parse_str(content: &str, workflow_path: &Path) -> Result<WorkflowConfig, GitlaneError> {
    parse_config(content, workflow_path, |content| {
        ::toml::from_str::<WorkflowConfigRepr>(content)
    })
}

pub fn to_string(config: &WorkflowConfig, workflow_path: &Path) -> Result<String, GitlaneError> {
    to_toml_string(&WorkflowConfigRepr::from(config), workflow_path)
}

pub fn save_to_path(workflow_path: &Path, config: &WorkflowConfig) -> Result<(), GitlaneError> {
    save_toml_config(workflow_path, &WorkflowConfigRepr::from(config))
}
