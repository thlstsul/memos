use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    api::{prefix, to_timestamp},
    impl_extract_name,
    model::{gen::UserSettingKey, user::UserSetting},
};

use super::{
    prefix::FormatName,
    v1::gen::{
        user::Role, GetUserRequest, UpdateUserSettingRequest, User, UserSetting as UserSettingApi,
    },
};

impl_extract_name!(GetUserRequest, prefix::USER_NAME_PREFIX);

impl From<Vec<UserSetting>> for UserSettingApi {
    fn from(value: Vec<UserSetting>) -> Self {
        let mut setting = UserSettingApi::default();
        for s in value {
            match s.key as UserSettingKey {
                UserSettingKey::Locale => setting.locale = s.value,
                UserSettingKey::Appearance => setting.appearance = s.value,
                UserSettingKey::MemoVisibility => setting.memo_visibility = s.value,
                _ => (),
            }
        }
        setting
    }
}

impl From<crate::model::user::User> for User {
    fn from(value: crate::model::user::User) -> Self {
        Self {
            name: value.get_name(),
            id: value.id,
            role: value.role as i32,
            username: value.username,
            email: value.email,
            nickname: value.nickname,
            avatar_url: value.avatar_url,
            description: value.description,
            password: value.password_hash,
            row_status: value.row_status as i32,
            create_time: to_timestamp(value.created_ts),
            update_time: to_timestamp(value.updated_ts),
        }
    }
}

impl FormatName for crate::model::user::User {
    fn get_name(&self) -> String {
        format!("{}/{}", prefix::USER_NAME_PREFIX, self.id)
    }
}

impl UpdateUserSettingRequest {
    pub fn as_settings(&self, user_id: i32) -> Vec<UserSetting> {
        let mut rtn = Vec::new();
        if let UpdateUserSettingRequest {
            update_mask: Some(field_mask),
            setting: Some(settings),
        } = self
        {
            for path in &field_mask.paths {
                let setting = match path.as_str() {
                    "locale" => UserSetting {
                        user_id,
                        key: UserSettingKey::Locale,
                        value: settings.locale.clone(),
                    },
                    "appearance" => UserSetting {
                        user_id,
                        key: UserSettingKey::Appearance,
                        value: settings.appearance.clone(),
                    },
                    "memo_visibility" => UserSetting {
                        user_id,
                        key: UserSettingKey::MemoVisibility,
                        value: settings.memo_visibility.clone(),
                    },
                    _ => continue,
                };
                rtn.push(setting);
            }
        }
        rtn
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let role = self.as_str_name();
        serializer.serialize_str(role)
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Role, D::Error>
    where
        D: Deserializer<'de>,
    {
        let role = String::deserialize(deserializer)?;
        let role = Role::from_str_name(&role).unwrap_or_default();
        Ok(role)
    }
}
