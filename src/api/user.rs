use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    api::{prefix, to_timestamp},
    impl_extract_name,
};

use super::{
    prefix::FormatName,
    v1::generate::{GetUserRequest, GetUserStatsRequest, User, user::Role},
};

impl_extract_name!(GetUserRequest, prefix::USER_NAME_PREFIX);
impl_extract_name!(GetUserStatsRequest, prefix::USER_NAME_PREFIX);

impl From<crate::model::user::User> for User {
    fn from(value: crate::model::user::User) -> Self {
        Self {
            name: value.get_name(),
            role: value.role as i32,
            username: value.username,
            email: value.email,
            avatar_url: value.avatar_url,
            description: value.description,
            password: value.password_hash,
            state: value.state as i32,
            create_time: to_timestamp(value.created_ts),
            update_time: to_timestamp(value.updated_ts),
            display_name: value.nickname,
        }
    }
}

impl FormatName for crate::model::user::User {
    fn get_name(&self) -> String {
        format!("{}/{}", prefix::USER_NAME_PREFIX, self.id)
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
