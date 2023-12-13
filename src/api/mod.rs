use snafu::{ensure, ResultExt, Snafu};
use tonic::Status;

use self::v2::{
    GetUserRequest, GetUserResponse, Inbox, ListTagsRequest, ListTagsResponse, Tag, User,
};

pub mod v1;
pub mod v2;

pub const USER_NAME_PREFIX: &str = "users";
pub const INBOX_NAME_PREFIX: &str = "inboxes";

impl GetUserRequest {
    pub fn get_name(&self) -> Result<String, Error> {
        get_name_parent_token(self.name.clone(), USER_NAME_PREFIX)
    }
}

impl From<User> for GetUserResponse {
    fn from(value: User) -> Self {
        let mut user = value;
        user.name = format!("{}/{}", USER_NAME_PREFIX, user.name);
        Self { user: Some(user) }
    }
}

impl ListTagsRequest {
    pub fn get_creator(&self) -> Result<String, Error> {
        get_name_parent_token(self.creator.clone(), USER_NAME_PREFIX)
    }
}

impl From<Vec<Tag>> for ListTagsResponse {
    fn from(tags: Vec<Tag>) -> Self {
        let mut tags = tags;
        tags.iter_mut()
            .for_each(|i| i.creator = format!("{}/{}", USER_NAME_PREFIX, i.creator));

        Self { tags }
    }
}

impl Inbox {
    pub fn get_id(&self) -> Result<i32, Error> {
        get_name_parent_token(self.name.clone(), INBOX_NAME_PREFIX)?
            .parse()
            .context(InvalidInboxId {
                name: self.name.clone(),
            })
    }
}

fn get_name_parent_token(name: String, token: &str) -> Result<String, Error> {
    let parts: Vec<&str> = name.split("/").collect();
    ensure!(parts.len() == 2, InvalidRequest { name });
    ensure!(token == parts[0], InvalidPrefix { name });
    Ok(parts[1].to_owned())
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid request : {name}"), context(suffix(false)))]
    InvalidRequest { name: String },
    #[snafu(display("Invalid prefix in request : {name}"), context(suffix(false)))]
    InvalidPrefix { name: String },
    #[snafu(display("Invalid inbox id : {name}"), context(suffix(false)))]
    InvalidInboxId {
        name: String,
        source: std::num::ParseIntError,
    },
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        Status::invalid_argument(value.to_string())
    }
}

/// prost_types::Timestamp serialize
mod time_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        date: &core::option::Option<prost_types::Timestamp>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let second = if let Some(date) = date {
            date.seconds
        } else {
            0
        };
        serializer.serialize_i64(second)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<core::option::Option<prost_types::Timestamp>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = i64::deserialize(deserializer)?;
        Ok(Some(prost_types::Timestamp { seconds, nanos: 0 }))
    }
}

/// enmu RowStatus serialize
mod status_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::v2::RowStatus;

    pub fn serialize<S>(status: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let row_status = RowStatus::try_from(*status);
        let row_status = row_status.unwrap_or(RowStatus::Unspecified);
        let row_status = if row_status == RowStatus::Unspecified {
            "NORMAL"
        } else {
            row_status.as_str_name()
        };
        serializer.serialize_str(row_status)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = String::deserialize(deserializer)?;
        let row_status = RowStatus::from_str_name(&status);
        let row_status = row_status.unwrap_or(RowStatus::Unspecified);
        Ok(row_status.into())
    }
}

/// enmu Role serialize
mod role_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::v2::user::Role;

    pub fn serialize<S>(role: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let role = Role::try_from(*role);
        let role = role.unwrap_or(Role::Unspecified);

        serializer.serialize_str(role.as_str_name())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let role = String::deserialize(deserializer)?;
        let role = Role::from_str_name(&role);
        let role = role.unwrap_or(Role::Unspecified);
        Ok(role.into())
    }
}

/// enmu SystemSettingKey serialize
mod system_setting {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::v1::system::SystemSettingKey;

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

    pub fn serialize<S>(key: &SystemSettingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = match key {
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

        serializer.serialize_str(key)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemSettingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        let key = match key.as_str() {
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
