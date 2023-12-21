use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tracing::error;

use super::v1::system::{CustomizedProfile, SystemStatus};

#[derive(Deserialize, Serialize, Clone)]
pub struct SystemSetting {
    #[serde(with = "system_setting")]
    pub name: SystemSettingKey,
    pub value: String,
    pub description: String,
}

#[derive(Clone)]
pub enum SystemSettingKey {
    // the name of server id.
    ServerId,
    // the name of secret session.
    SecretSession,
    // the name of allow signup setting.
    AllowSignup,
    // the name of disable password login setting.
    DisablePasswordLogin,
    // the name of disable public memos setting.
    DisablePublicMemos,
    // the name of max upload size setting.
    MaxUploadSizeMiB,
    // the name of additional style.
    AdditionalStyle,
    // the name of additional script.
    AdditionalScript,
    // the name of customized server profile.
    CustomizedProfile,
    // the name of storage service ID.
    StorageServiceID,
    // the name of local storage path.
    LocalStoragePath,
    // the name of Telegram Bot Token.
    TelegramBotToken,
    // the name of memo display with updated ts.
    MemoDisplayWithUpdatedTs,
    // the name of auto backup interval as seconds.
    AutoBackupInterval,
    Unknow,
}

impl From<Vec<SystemSetting>> for SystemStatus {
    fn from(value: Vec<SystemSetting>) -> Self {
        let mut rtn = SystemStatus::default();
        for i in value {
            match i.name {
                SystemSettingKey::AllowSignup => {
                    rtn.allow_sign_up = i.value.parse().unwrap_or_default()
                }
                SystemSettingKey::DisablePasswordLogin => {
                    rtn.disable_password_login = i.value.parse().unwrap_or_default()
                }
                SystemSettingKey::DisablePublicMemos => {
                    rtn.disable_public_memos = i.value.parse().unwrap_or_default()
                }
                SystemSettingKey::MaxUploadSizeMiB => {
                    rtn.max_upload_size_mi_b = i.value.parse().unwrap_or_default()
                }
                SystemSettingKey::AdditionalStyle => rtn.additional_style = i.value,
                SystemSettingKey::AdditionalScript => rtn.additional_script = i.value,
                SystemSettingKey::CustomizedProfile => {
                    let c_f = serde_json::from_str::<CustomizedProfile>(&i.value);
                    if let Ok(c_f) = c_f {
                        rtn.customized_profile = c_f;
                    } else {
                        error!("{c_f:?}");
                    }
                }
                SystemSettingKey::StorageServiceID => {
                    rtn.storage_service_id = i.value.parse().unwrap_or_default()
                }
                SystemSettingKey::LocalStoragePath => rtn.local_storage_path = i.value,
                SystemSettingKey::MemoDisplayWithUpdatedTs => {
                    rtn.memo_display_with_updated_ts = i.value.parse().unwrap_or_default()
                }
                SystemSettingKey::AutoBackupInterval => {
                    rtn.auto_backup_interval = i.value.parse().unwrap_or_default()
                }
                _ => (),
            }
        }
        rtn
    }
}

const SERVER_ID: &str = "server-id";
const SECRET_SESSION: &str = "secret-session";
const ALLOW_SIGNUP: &str = "allow-signup";
const DISABLE_PASSWORD_LOGIN: &str = "disable-password-login";
const DISABLE_PUBLIC_MEMOS: &str = "disable-public-memos";
const MAX_UPLOAD_SIZE_MIB: &str = "max-upload-size-mib";
const ADDITIONAL_STYLE: &str = "additional-style";
const ADDITIONAL_SCRIPT: &str = "additional-script";
const CUSTOMIZED_PROFILE: &str = "customized-profile";
const STORAGE_SERVICE_ID: &str = "storage-service-id";
const LOCAL_STORAGE_PATH: &str = "local-storage-path";
const TELEGRAM_BOT_TOKEN: &str = "telegram-bot-token";
const MEMO_DISPLAY_WITH_UPDATED_TS: &str = "memo-display-with-updated-ts";
const AUTO_BACKUP_INTERVAL: &str = "auto-backup-interval";

impl FromStr for SystemSettingKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = match s {
            SERVER_ID => SystemSettingKey::ServerId,
            SECRET_SESSION => SystemSettingKey::SecretSession,
            ALLOW_SIGNUP => SystemSettingKey::AllowSignup,
            DISABLE_PASSWORD_LOGIN => SystemSettingKey::DisablePasswordLogin,
            DISABLE_PUBLIC_MEMOS => SystemSettingKey::DisablePublicMemos,
            MAX_UPLOAD_SIZE_MIB => SystemSettingKey::MaxUploadSizeMiB,
            ADDITIONAL_STYLE => SystemSettingKey::AdditionalStyle,
            ADDITIONAL_SCRIPT => SystemSettingKey::AdditionalScript,
            CUSTOMIZED_PROFILE => SystemSettingKey::CustomizedProfile,
            STORAGE_SERVICE_ID => SystemSettingKey::StorageServiceID,
            LOCAL_STORAGE_PATH => SystemSettingKey::LocalStoragePath,
            TELEGRAM_BOT_TOKEN => SystemSettingKey::TelegramBotToken,
            MEMO_DISPLAY_WITH_UPDATED_TS => SystemSettingKey::MemoDisplayWithUpdatedTs,
            AUTO_BACKUP_INTERVAL => SystemSettingKey::AutoBackupInterval,
            _ => SystemSettingKey::Unknow,
        };
        Ok(key)
    }
}

impl ToString for SystemSettingKey {
    fn to_string(&self) -> String {
        let key = match self {
            SystemSettingKey::ServerId => SERVER_ID,
            SystemSettingKey::SecretSession => SECRET_SESSION,
            SystemSettingKey::AllowSignup => ALLOW_SIGNUP,
            SystemSettingKey::DisablePasswordLogin => DISABLE_PASSWORD_LOGIN,
            SystemSettingKey::DisablePublicMemos => DISABLE_PUBLIC_MEMOS,
            SystemSettingKey::MaxUploadSizeMiB => MAX_UPLOAD_SIZE_MIB,
            SystemSettingKey::AdditionalStyle => ADDITIONAL_STYLE,
            SystemSettingKey::AdditionalScript => ADDITIONAL_SCRIPT,
            SystemSettingKey::CustomizedProfile => CUSTOMIZED_PROFILE,
            SystemSettingKey::StorageServiceID => STORAGE_SERVICE_ID,
            SystemSettingKey::LocalStoragePath => LOCAL_STORAGE_PATH,
            SystemSettingKey::TelegramBotToken => TELEGRAM_BOT_TOKEN,
            SystemSettingKey::MemoDisplayWithUpdatedTs => MEMO_DISPLAY_WITH_UPDATED_TS,
            SystemSettingKey::AutoBackupInterval => AUTO_BACKUP_INTERVAL,
            SystemSettingKey::Unknow => "",
        };
        key.to_string()
    }
}

/// enmu SystemSettingKey serialize
mod system_setting {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::*;
    use crate::api::system::SystemSettingKey;

    pub fn serialize<S>(key: &SystemSettingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = key.to_string();
        serializer.serialize_str(&key)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemSettingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        let key = SystemSettingKey::from_str(&key).unwrap();
        Ok(key)
    }
}
