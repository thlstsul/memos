pub mod gen;
pub mod memo;
pub mod pager;
pub mod resource;
pub mod session;
pub mod system;
pub mod user;

pub mod bool_serde {
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

pub mod option_serde {
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

/// prost_types::Timestamp serialize
pub mod time_serde {
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
