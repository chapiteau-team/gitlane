use thiserror::Error;

use crate::project::ProjectConfigError;

#[derive(Debug, Error)]
pub enum GitlaneError {
    #[error(transparent)]
    ProjectConfig(#[from] ProjectConfigError),
}
