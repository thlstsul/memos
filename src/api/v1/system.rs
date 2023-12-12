use serde::{Deserialize, Serialize};
use tracing::error;

use crate::api::v2::User;

const VERSION: &str = "v0.17.1";
const MODE: &str = "prod";

#[derive(Deserialize, Serialize)]
pub struct SystemSetting {
    #[serde(with = "crate::api::system_setting")]
    name: SystemSettingKey,
    value: String,
    description: String,
}

#[derive(Deserialize, Serialize)]
pub struct SystemStatus {
    #[serde(rename = "additionalScript")]
    pub additional_script: String,
    #[serde(rename = "additionalStyle")]
    pub additional_style: String,
    #[serde(rename = "allowSignUp")]
    pub allow_sign_up: bool,
    #[serde(rename = "autoBackupInterval")]
    pub auto_backup_interval: i32,
    #[serde(rename = "customizedProfile")]
    pub customized_profile: CustomizedProfile,
    #[serde(rename = "dbSize")]
    pub db_size: i32,
    #[serde(rename = "disablePasswordLogin")]
    pub disable_password_login: bool,
    #[serde(rename = "disablePublicMemos")]
    pub disable_public_memos: bool,
    pub host: Host,
    #[serde(rename = "localStoragePath")]
    pub local_storage_path: String,
    #[serde(rename = "maxUploadSizeMiB")]
    pub max_upload_size_mi_b: i32,
    #[serde(rename = "memoDisplayWithUpdatedTs")]
    pub memo_display_with_updated_ts: bool,
    pub profile: Profile,
    #[serde(rename = "storageServiceId")]
    pub storage_service_id: i32,
}

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

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            allow_sign_up: true,
            customized_profile: CustomizedProfile {
                locale: "zh-Hans".to_owned(),
                name: "memos".to_owned(),
                appearance: "system".to_owned(),
                ..Default::default()
            },
            local_storage_path: "assets/{timestamp}_{filename}".to_owned(),
            max_upload_size_mi_b: 32,
            profile: Profile {
                mode: MODE.to_owned(),
                version: VERSION.to_owned(),
                ..Default::default()
            },
            storage_service_id: -1,
            additional_script: Default::default(),
            additional_style: Default::default(),
            auto_backup_interval: Default::default(),
            db_size: Default::default(),
            disable_password_login: Default::default(),
            disable_public_memos: Default::default(),
            host: Default::default(),
            memo_display_with_updated_ts: Default::default(),
        }
    }
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

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct CustomizedProfile {
    appearance: String,
    description: String,
    #[serde(rename = "externalUrl")]
    external_url: String,
    locale: String,
    #[serde(rename = "logoUrl")]
    logo_url: String,
    name: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Profile {
    // Mode can be "prod" or "dev" or "demo"
    mode: String,
    // Version is the current version of server
    version: String,
    // Driver is the database driver
    // sqlite, mysql
    #[serde(skip)]
    _driver: String,
    // DSN points to where memos stores its own data
    #[serde(skip)]
    _dsn: String,
    // Addr is the binding address for server
    #[serde(skip)]
    _addr: String,
    // Port is the binding port for server
    #[serde(skip)]
    _port: String,
    // Data is the data directory
    #[serde(skip)]
    _data: String,
    // Metric indicate the metric collection is enabled or not
    #[serde(skip)]
    _metric: bool,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Host {
    id: i32,
    #[serde(rename = "createdTs")]
    pub create_time: i32,

    #[serde(rename = "updatedTs")]
    pub update_time: i32,
}

impl From<User> for Host {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            create_time: 0,
            update_time: 0,
        }
    }
}
