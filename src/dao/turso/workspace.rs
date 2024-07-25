use async_trait::async_trait;
use tracing::error;

use crate::dao::workspace::{FindWorkspaceSettingError, WorkspaceRepository};
use crate::model::gen::{
    workspace_setting::Value as WorkspaceSettingValue, WorkspaceGeneralSetting,
    WorkspaceMemoRelatedSetting, WorkspaceSettingKey, WorkspaceStorageSetting,
};
use crate::model::system::SystemSetting;

use super::Turso;

#[async_trait]
impl WorkspaceRepository for Turso {
    async fn find_workspace_setting(
        &self,
        key: WorkspaceSettingKey,
    ) -> Result<Option<WorkspaceSettingValue>, FindWorkspaceSettingError> {
        let sql = "select * from system_setting where name = ?";
        let mut settings: Vec<SystemSetting> = self.query(sql, [key.as_str_name()]).await?;

        let value = settings.pop().and_then(|s| match key {
            WorkspaceSettingKey::Unspecified => None,
            WorkspaceSettingKey::Basic => None,
            WorkspaceSettingKey::General => {
                serde_json::from_str::<WorkspaceGeneralSetting>(&s.value)
                    .inspect_err(|e| error!("{e}"))
                    .map(WorkspaceSettingValue::GeneralSetting)
                    .ok()
            }
            WorkspaceSettingKey::Storage => {
                serde_json::from_str::<WorkspaceStorageSetting>(&s.value)
                    .inspect_err(|e| error!("{e}"))
                    .map(WorkspaceSettingValue::StorageSetting)
                    .ok()
            }
            WorkspaceSettingKey::MemoRelated => {
                serde_json::from_str::<WorkspaceMemoRelatedSetting>(&s.value)
                    .inspect_err(|e| error!("{e}"))
                    .map(WorkspaceSettingValue::MemoRelatedSetting)
                    .ok()
            }
        });
        Ok(value)
    }
}
