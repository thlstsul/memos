use serde::{Deserialize, Deserializer, Serialize, Serializer};
use snafu::{ensure, Snafu};
use tonic::Status;
use tracing::error;

use self::v2::{RowStatus, Visibility};

pub mod auth;
pub mod inbox;
pub mod memo;
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

fn get_name_parent_token(name: String, token: &str) -> Result<String, Error> {
    let parts: Vec<&str> = name.split("/").collect();
    ensure!(parts.len() == 2, InvalidRequest { name });
    ensure!(token == parts[0], InvalidPrefix { name });
    Ok(parts[1].to_owned())
}

mod bool_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

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

/// enmu RowStatus serialize
impl Serialize for RowStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let row_status = if self == &RowStatus::Unspecified {
            "NORMAL"
        } else {
            self.as_str_name()
        };
        serializer.serialize_str(row_status)
    }
}

impl<'de> Deserialize<'de> for RowStatus {
    fn deserialize<D>(deserializer: D) -> Result<RowStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = String::deserialize(deserializer)?;
        let row_status = RowStatus::from_str_name(&status);
        let row_status = row_status.unwrap_or(RowStatus::Unspecified);
        Ok(row_status.into())
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
        Ok(visibility.into())
    }
}

mod visibility_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::v2::Visibility;

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

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid request : {name}"), context(suffix(false)))]
    InvalidRequest { name: String },
    #[snafu(display("Invalid prefix in request : {name}"), context(suffix(false)))]
    InvalidPrefix { name: String },
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        error!("{value}");
        Status::invalid_argument(value.to_string())
    }
}

impl From<memo::Error> for Status {
    fn from(value: memo::Error) -> Self {
        error!("{value}");
        Status::invalid_argument(value.to_string())
    }
}