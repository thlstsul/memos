use crate::api::prefix;
use crate::impl_extract_name;

use super::v1::gen::{
    workspace_setting::Value as WorkspaceSettingValue, workspace_storage_setting::S3Config,
    GetWorkspaceSettingRequest, WorkspaceCustomProfile, WorkspaceGeneralSetting,
    WorkspaceMemoRelatedSetting, WorkspaceStorageSetting,
};

impl_extract_name!(
    GetWorkspaceSettingRequest,
    prefix::WORKSPACE_SETTING_NAME_PREFIX
);

impl From<crate::model::gen::workspace_setting::Value> for WorkspaceSettingValue {
    fn from(value: crate::model::gen::workspace_setting::Value) -> Self {
        match value {
            crate::model::gen::workspace_setting::Value::BasicSetting(_) => unimplemented!(),
            crate::model::gen::workspace_setting::Value::GeneralSetting(s) => {
                WorkspaceSettingValue::GeneralSetting(s.into())
            }
            crate::model::gen::workspace_setting::Value::StorageSetting(s) => {
                WorkspaceSettingValue::StorageSetting(s.into())
            }
            crate::model::gen::workspace_setting::Value::MemoRelatedSetting(s) => {
                WorkspaceSettingValue::MemoRelatedSetting(s.into())
            }
        }
    }
}

impl From<crate::model::gen::WorkspaceCustomProfile> for WorkspaceCustomProfile {
    fn from(value: crate::model::gen::WorkspaceCustomProfile) -> Self {
        Self {
            title: value.title,
            description: value.description,
            logo_url: value.logo_url,
            locale: value.locale,
            appearance: value.appearance,
        }
    }
}

impl From<crate::model::gen::WorkspaceGeneralSetting> for WorkspaceGeneralSetting {
    fn from(value: crate::model::gen::WorkspaceGeneralSetting) -> Self {
        Self {
            additional_script: value.additional_script,
            additional_style: value.additional_style,
            custom_profile: value.custom_profile.map(|cp| cp.into()),
            disallow_user_registration: value.disallow_user_registration,
            disallow_password_auth: value.disallow_password_auth,
            week_start_day_offset: value.week_start_day_offset,
            disallow_change_username: value.disallow_change_username,
            disallow_change_nickname: value.disallow_change_nickname,
        }
    }
}

impl From<crate::model::gen::WorkspaceMemoRelatedSetting> for WorkspaceMemoRelatedSetting {
    fn from(value: crate::model::gen::WorkspaceMemoRelatedSetting) -> Self {
        Self {
            disallow_public_visibility: value.disallow_public_visibility,
            display_with_update_time: value.display_with_update_time,
            content_length_limit: value.content_length_limit,
            enable_double_click_edit: value.enable_double_click_edit,
            enable_link_preview: value.enable_link_preview,
            enable_comment: value.enable_comment,
            enable_location: value.enable_location,
            reactions: value.reactions,
            disable_markdown_shortcuts: value.disable_markdown_shortcuts,
            enable_blur_nsfw_content: value.enable_blur_nsfw_content,
            nsfw_tags: value.nsfw_tags,
        }
    }
}

impl From<crate::model::gen::StorageS3Config> for S3Config {
    fn from(value: crate::model::gen::StorageS3Config) -> Self {
        Self {
            access_key_id: value.access_key_id,
            access_key_secret: value.access_key_secret,
            endpoint: value.endpoint,
            region: value.region,
            bucket: value.bucket,
            use_path_style: value.use_path_style,
        }
    }
}

impl From<crate::model::gen::WorkspaceStorageSetting> for WorkspaceStorageSetting {
    fn from(value: crate::model::gen::WorkspaceStorageSetting) -> Self {
        Self {
            storage_type: value.storage_type,
            filepath_template: value.filepath_template,
            upload_size_limit_mb: value.upload_size_limit_mb,
            s3_config: value.s3_config.map(|s| s.into()),
        }
    }
}
