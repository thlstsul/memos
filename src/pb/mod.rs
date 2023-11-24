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

        serializer.serialize_str(row_status.as_str_name())
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
