use std::path::Path;

use crate::{
    config::{load_config_from_path, parse_config},
    errors::GitlaneError,
};

use super::{LabelsConfig, codec::LabelsConfigRepr};

pub fn load_from_path(config_path: &Path) -> Result<LabelsConfig, GitlaneError> {
    load_config_from_path(config_path, parse_str)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<LabelsConfig, GitlaneError> {
    parse_config(content, config_path, |content| {
        serde_yaml::from_str::<LabelsConfigRepr>(content)
    })
}
