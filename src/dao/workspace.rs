use async_trait::async_trait;
use snafu::Snafu;

use crate::model::gen::{workspace_setting::Value as WorkspaceSettingValue, WorkspaceSettingKey};

#[async_trait]
pub trait WorkspaceRepository: Clone + Send + Sync + 'static {
    async fn find_workspace_setting(
        &self,
        key: WorkspaceSettingKey,
    ) -> Result<Option<WorkspaceSettingValue>, FindWorkspaceSettingError>;
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to find workspace setting: {source}"))]
pub struct FindWorkspaceSettingError {
    source: anyhow::Error,
}
