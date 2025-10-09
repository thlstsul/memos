use crate::api::prefix;
use crate::impl_extract_name;

use super::v1::generate::{
    GetWorkspaceSettingRequest, workspace_setting::Value as WorkspaceSettingValue,
};

impl_extract_name!(
    GetWorkspaceSettingRequest,
    prefix::WORKSPACE_SETTING_NAME_PREFIX
);

impl From<crate::model::generate::workspace_setting::Value> for WorkspaceSettingValue {
    fn from(value: crate::model::generate::workspace_setting::Value) -> Self {
        match value {
            crate::model::generate::workspace_setting::Value::BasicSetting(_) => unimplemented!(),
            crate::model::generate::workspace_setting::Value::GeneralSetting(s) => {
                WorkspaceSettingValue::GeneralSetting(s.into())
            }
            crate::model::generate::workspace_setting::Value::StorageSetting(s) => {
                WorkspaceSettingValue::StorageSetting(s.into())
            }
            crate::model::generate::workspace_setting::Value::MemoRelatedSetting(s) => {
                WorkspaceSettingValue::MemoRelatedSetting(s.into())
            }
        }
    }
}
