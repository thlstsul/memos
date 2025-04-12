pub mod auth;
pub mod inbox;
pub mod memo;
pub mod prefix;
pub mod resource;
pub mod user;
pub mod v1;
pub mod workspace;

use std::{fmt::Display, str::FromStr};

use prost_types::Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::api::v1::gen::State;

pub fn to_timestamp(value: i64) -> Option<Timestamp> {
    if value == 0 {
        None
    } else {
        Some(Timestamp {
            seconds: value,
            nanos: 0,
        })
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str_name())
    }
}

impl FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        State::from_str_name(s).ok_or(())
    }
}

/// enmu RowStatus serialize
impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state = self.to_string();
        serializer.serialize_str(&state)
    }
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<State, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = String::deserialize(deserializer)?;
        let state = status.parse().unwrap_or_default();
        Ok(state)
    }
}
