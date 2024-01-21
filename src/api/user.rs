use std::str::FromStr;

use super::{
    v2::{GetUserRequest, GetUserResponse, UpdateUserSettingRequest, User},
    USER_NAME_PREFIX,
};

use serde::{Deserialize, Deserializer};
use snafu::{ResultExt, Snafu};

use crate::{api::v2::GetUserSettingResponse, util::get_name_parent_token};

#[derive(Debug, Deserialize)]
pub struct UserSetting {
    pub user_id: i32,
    #[serde(deserialize_with = "deserialize_key")]
    pub key: UserSettingKey,
    pub value: String,
}

#[derive(Debug)]
pub enum UserSettingKey {
    Unspecified,
    AccessTokens,
    Locale,
    Appearance,
    Visibility,
    TelegramUserId,
}

const UNSPECIFIED: &str = "USER_SETTING_KEY_UNSPECIFIED";
const ACCESS_TOKENS: &str = "USER_SETTING_ACCESS_TOKENS";
const LOCALE: &str = "USER_SETTING_LOCALE";
const APPEARANCE: &str = "USER_SETTING_APPEARANCE";
const VISIBILITY: &str = "USER_SETTING_MEMO_VISIBILITY";
const TELEGRAM_USER_ID: &str = "USER_SETTING_TELEGRAM_USER_ID";

impl FromStr for UserSettingKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = match s {
            ACCESS_TOKENS => UserSettingKey::AccessTokens,
            LOCALE => UserSettingKey::Locale,
            APPEARANCE => UserSettingKey::Appearance,
            VISIBILITY => UserSettingKey::Visibility,
            TELEGRAM_USER_ID => UserSettingKey::TelegramUserId,
            _ => UserSettingKey::Unspecified,
        };
        Ok(key)
    }
}

impl ToString for UserSettingKey {
    fn to_string(&self) -> String {
        let key = match self {
            UserSettingKey::Unspecified => UNSPECIFIED,
            UserSettingKey::AccessTokens => ACCESS_TOKENS,
            UserSettingKey::Locale => LOCALE,
            UserSettingKey::Appearance => APPEARANCE,
            UserSettingKey::Visibility => VISIBILITY,
            UserSettingKey::TelegramUserId => TELEGRAM_USER_ID,
        };
        key.to_owned()
    }
}

impl From<Vec<UserSetting>> for GetUserSettingResponse {
    fn from(value: Vec<UserSetting>) -> Self {
        let mut setting = super::v2::UserSetting::default();
        for s in value {
            match s.key {
                UserSettingKey::Locale => setting.locale = s.value,
                UserSettingKey::Appearance => setting.appearance = s.value,
                UserSettingKey::Visibility => setting.memo_visibility = s.value,
                UserSettingKey::TelegramUserId => setting.telegram_user_id = s.value,
                _ => (),
            }
        }
        Self {
            setting: Some(setting),
        }
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
                        key: UserSettingKey::Visibility,
                        value: settings.memo_visibility.clone(),
                    },
                    "telegram_user_id" => UserSetting {
                        user_id,
                        key: UserSettingKey::TelegramUserId,
                        value: settings.telegram_user_id.clone(),
                    },
                    _ => continue,
                };
                rtn.push(setting);
            }
        }
        rtn
    }
}

impl GetUserRequest {
    pub fn get_name(&self) -> Result<String, Error> {
        get_name_parent_token(&self.name, USER_NAME_PREFIX).context(InvalidUsername)
    }
}

impl From<User> for GetUserResponse {
    fn from(value: User) -> Self {
        let mut user = value;
        user.name = format!("{}/{}", USER_NAME_PREFIX, user.username);
        Self { user: Some(user) }
    }
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<UserSettingKey, D::Error>
where
    D: Deserializer<'de>,
{
    let key = String::deserialize(deserializer)?;
    let key = UserSettingKey::from_str(&key).unwrap();
    Ok(key)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid username : {source}"), context(suffix(false)))]
    InvalidUsername { source: crate::util::Error },
}
