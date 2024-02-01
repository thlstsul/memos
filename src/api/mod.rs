use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use self::v2::{RowStatus, Visibility};

pub mod auth;
pub mod inbox;
pub mod memo;
pub mod pager;
pub mod resource;
pub mod system;
pub mod tag;
pub mod user;
pub mod v1;
pub mod v2;

#[derive(Deserialize)]
pub struct Count {
    pub count: i32,
}

pub const INBOX_NAME_PREFIX: &str = "inboxes";
pub const USER_NAME_PREFIX: &str = "users";

mod option_serde {
    use serde::{self, de::DeserializeOwned, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        let data = T::deserialize(deserializer).ok();
        Ok(data)
    }
}

mod bool_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S>(data: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = if *data { 1 } else { 0 };
        serializer.serialize_i8(data)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = i8::deserialize(deserializer)?;
        Ok(data == 1)
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

impl ToString for RowStatus {
    fn to_string(&self) -> String {
        let row_status = if self == &RowStatus::Unspecified || self == &RowStatus::Active {
            "NORMAL"
        } else {
            self.as_str_name()
        };
        row_status.to_owned()
    }
}

impl FromStr for RowStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "NORMAL" {
            Ok(RowStatus::Active)
        } else {
            RowStatus::from_str_name(s).ok_or(())
        }
    }
}

/// enmu RowStatus serialize
impl Serialize for RowStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let row_status = self.to_string();
        serializer.serialize_str(&row_status)
    }
}

impl<'de> Deserialize<'de> for RowStatus {
    fn deserialize<D>(deserializer: D) -> Result<RowStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = String::deserialize(deserializer)?;
        let row_status = status.parse();
        let row_status = row_status.unwrap_or(RowStatus::Unspecified);
        Ok(row_status)
    }
}

mod status_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::v2::RowStatus;

    pub fn serialize<S>(status: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let row_status = RowStatus::try_from(*status);
        let row_status = row_status.unwrap_or(RowStatus::Unspecified);
        let row_status = row_status.to_string();
        serializer.serialize_str(&row_status)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = String::deserialize(deserializer)?;
        let row_status = status.parse();
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

impl Serialize for Visibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let visibility = self.as_str_name();
        serializer.serialize_str(visibility)
    }
}

impl<'de> Deserialize<'de> for Visibility {
    fn deserialize<D>(deserializer: D) -> Result<Visibility, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visibility = String::deserialize(deserializer)?;
        let visibility = Visibility::from_str_name(&visibility);
        let visibility = visibility.unwrap_or(Visibility::Unspecified);
        Ok(visibility)
    }
}

mod visibility_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::v2::Visibility;

    #[allow(dead_code)]
    pub fn serialize<S>(visibility: &i32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let visibility = Visibility::try_from(*visibility);
        let visibility = visibility.unwrap_or(Visibility::Unspecified);
        let visibility = visibility.as_str_name();
        serializer.serialize_str(visibility)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visibility = String::deserialize(deserializer)?;
        let visibility = Visibility::from_str_name(&visibility);
        let visibility = visibility.unwrap_or(Visibility::Unspecified);
        Ok(visibility.into())
    }
}
