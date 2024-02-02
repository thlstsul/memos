use serde::{Deserialize, Serialize};

use crate::api::v2::User;

const VERSION: &str = "0.19.0";
const MODE: &str = "prod";

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

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            allow_sign_up: true,
            customized_profile: CustomizedProfile {
                locale: "zh-Hans".to_owned(),
                name: "Memos".to_owned(),
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
