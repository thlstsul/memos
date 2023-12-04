use serde::{Deserialize, Serialize};

use crate::api::memos_api_v2::User;

#[derive(Deserialize, Serialize)]
pub struct SystemSetting {
    name: SystemSettingKey,
    value: String,
    description: String,
}

#[derive(Deserialize, Serialize, Default)]
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

impl From<String> for SystemSettingKey {
    fn from(value: String) -> Self {
        match value.as_str() {
            "server-id" => SystemSettingKey::ServerId,
            "secret-session" => SystemSettingKey::SecretSession,
            "allow-signup" => SystemSettingKey::AllowSignup,
            "disable-password-login" => SystemSettingKey::DisablePasswordLogin,
            "disable-public-memos" => SystemSettingKey::DisablePublicMemos,
            "max-upload-size-mib" => SystemSettingKey::MaxUploadSizeMiB,
            "additional-style" => SystemSettingKey::AdditionalStyle,
            "additional-script" => SystemSettingKey::AdditionalScript,
            "customized-profile" => SystemSettingKey::CustomizedProfile,
            "storage-service-id" => SystemSettingKey::StorageServiceID,
            "local-storage-path" => SystemSettingKey::LocalStoragePath,
            "telegram-bot-token" => SystemSettingKey::TelegramBotToken,
            "memo-display-with-updated-ts" => SystemSettingKey::MemoDisplayWithUpdatedTs,
            "auto-backup-interval" => SystemSettingKey::AutoBackupInterval,
            _ => SystemSettingKey::Unknow,
        }
    }
}

impl From<Vec<SystemSetting>> for SystemStatus {
    fn from(value: Vec<SystemSetting>) -> Self {
        let mut rtn = SystemStatus::default();
        for i in value {
            let key: SystemSettingKey = i.name.into();
            match key {
                SystemSettingKey::ServerId => todo!(),
                SystemSettingKey::SecretSession => todo!(),
                SystemSettingKey::AllowSignup => todo!(),
                SystemSettingKey::DisablePasswordLogin => todo!(),
                SystemSettingKey::DisablePublicMemos => todo!(),
                SystemSettingKey::MaxUploadSizeMiB => todo!(),
                SystemSettingKey::AdditionalStyle => todo!(),
                SystemSettingKey::AdditionalScript => todo!(),
                SystemSettingKey::CustomizedProfile => todo!(),
                SystemSettingKey::StorageServiceID => todo!(),
                SystemSettingKey::LocalStoragePath => todo!(),
                SystemSettingKey::TelegramBotToken => todo!(),
                SystemSettingKey::MemoDisplayWithUpdatedTs => todo!(),
                SystemSettingKey::AutoBackupInterval => todo!(),
                SystemSettingKey::Unknow => todo!(),
            }
        }
        rtn
    }
}

#[derive(Deserialize, Serialize, Default)]
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
    driver: String,
    // DSN points to where memos stores its own data
    #[serde(skip)]
    dsn: String,
    // Addr is the binding address for server
    #[serde(skip)]
    addr: String,
    // Port is the binding port for server
    #[serde(skip)]
    port: String,
    // Data is the data directory
    #[serde(skip)]
    data: String,
    // Metric indicate the metric collection is enabled or not
    #[serde(skip)]
    metric: bool,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Host {
    id: i32,
}

impl From<User> for Host {
    fn from(value: User) -> Self {
        Self { id: value.id }
    }
}
