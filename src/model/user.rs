use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::api::v1::gen::{user::Role, State};

use super::gen::UserSettingKey;

#[derive(Debug, Default, Deserialize)]
pub struct UserSetting {
    pub user_id: i32,
    pub key: UserSettingKey,
    pub value: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct User {
    pub id: i32,
    pub role: Role,
    pub username: String,
    pub email: String,
    pub nickname: String,
    pub avatar_url: String,
    pub description: String,
    pub password_hash: String,
    pub state: State,
    pub created_ts: i64,
    pub updated_ts: i64,
}

impl Serialize for UserSettingKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = self.as_str_name();
        serializer.serialize_str(key)
    }
}

impl<'de> Deserialize<'de> for UserSettingKey {
    fn deserialize<D>(deserializer: D) -> Result<UserSettingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        let key = UserSettingKey::from_str_name(&key).unwrap_or_default();
        Ok(key)
    }
}
