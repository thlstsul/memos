use snafu::{ensure, Snafu};
use tonic::Status;
use tracing::error;

pub mod auth;
pub mod inbox;
pub mod memo;
pub mod system;
pub mod tag;
pub mod user;
pub mod v1;
pub mod v2;

pub const INBOX_NAME_PREFIX: &str = "inboxes";
pub const USER_NAME_PREFIX: &str = "users";

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
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        error!("{value}");
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
