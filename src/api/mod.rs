pub mod memos_api_v1;
pub mod memos_api_v2;

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

    use super::memos_api_v2::RowStatus;

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

    use super::memos_api_v2::user::Role;

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

/// enmu SystemSettingKey
mod system_setting {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::memos_api_v1::system::SystemSettingKey;

    pub fn serialize<S>(key: &SystemSettingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = match key {
            SystemSettingKey::ServerId => "server-id",
            SystemSettingKey::SecretSession => "secret-session",
            SystemSettingKey::AllowSignup => "allow-signup",
            SystemSettingKey::DisablePasswordLogin => "disable-password-login",
            SystemSettingKey::DisablePublicMemos => "disable-public-memos",
            SystemSettingKey::MaxUploadSizeMiB => "max-upload-size-mib",
            SystemSettingKey::AdditionalStyle => "additional-style",
            SystemSettingKey::AdditionalScript => "additional-script",
            SystemSettingKey::CustomizedProfile => "customized-profile",
            SystemSettingKey::StorageServiceID => "storage-service-id",
            SystemSettingKey::LocalStoragePath => "local-storage-path",
            SystemSettingKey::TelegramBotToken => "telegram-bot-token",
            SystemSettingKey::MemoDisplayWithUpdatedTs => "memo-display-with-updated-ts",
            SystemSettingKey::AutoBackupInterval => "auto-backup-interval",
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
        };
        Ok(key)
    }
}
