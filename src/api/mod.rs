use std::{fmt::Display, str::FromStr};

use prost_types::Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::api::v1::gen::RowStatus;

pub mod auth;
pub mod inbox;
pub mod memo;
pub mod prefix;
pub mod resource;
pub mod user;
pub mod v1;
pub mod workspace;

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

impl Display for RowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row_status = if self == &RowStatus::Unspecified || self == &RowStatus::Active {
            "NORMAL"
        } else {
            self.as_str_name()
        };
        write!(f, "{row_status}")
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
        let row_status = status.parse().unwrap_or_default();
        Ok(row_status)
    }
}
