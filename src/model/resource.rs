use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::util;

use super::generate::{AttachmentPayload, AttachmentStorageType};

#[derive(Debug, Default)]
pub struct FindResource {
    pub id: Option<i32>,
    pub uid: Option<String>,
    pub creator_id: Option<i32>,
    pub filename: Option<String>,
    pub get_blob: bool,
    pub memo_id: Option<i32>,
    pub limit: Option<isize>,
    pub offset: Option<isize>,
    pub has_relate_memo: bool,
    pub storage_type: Option<AttachmentStorageType>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Resource {
    pub id: i32,
    pub uid: String,

    pub creator_id: i32,
    pub created_ts: i64,
    pub updated_ts: i64,

    pub filename: String,
    pub blob: Vec<u8>,
    pub r#type: String,
    pub size: usize,

    pub storage_type: AttachmentStorageType,
    pub reference: String,

    #[serde(deserialize_with = "crate::model::resource::payload_serde::deserialize")]
    pub payload: AttachmentPayload,
    #[serde(deserialize_with = "crate::model::option_serde::deserialize")]
    pub memo_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct ResourceQry {
    pub thumbnail: Option<String>,
}

impl From<crate::api::v1::generate::Attachment> for Resource {
    fn from(value: crate::api::v1::generate::Attachment) -> Self {
        let memo_id = value.get_memo();
        let blob = value.content;
        let size = blob.len();
        Self {
            uid: util::uuid(),
            filename: value.filename,
            blob,
            r#type: value.r#type,
            size,
            memo_id,
            ..Default::default()
        }
    }
}

impl From<AttachmentPayload> for libsql::Value {
    fn from(val: AttachmentPayload) -> Self {
        libsql::Value::Text(serde_json::to_string(&val).unwrap_or("{}".to_string()))
    }
}

impl Serialize for AttachmentStorageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let storage_type = self.as_str_name();
        serializer.serialize_str(storage_type)
    }
}

impl<'de> Deserialize<'de> for AttachmentStorageType {
    fn deserialize<D>(deserializer: D) -> Result<AttachmentStorageType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let storage_type = String::deserialize(deserializer)?;
        let storage_type = AttachmentStorageType::from_str_name(&storage_type).unwrap_or_default();
        Ok(storage_type)
    }
}

pub mod payload_serde {
    use crate::model::generate::AttachmentPayload;
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S>(payload: &AttachmentPayload, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = serde_json::to_string(payload).unwrap_or("{}".to_string());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AttachmentPayload, D::Error>
    where
        D: Deserializer<'de>,
    {
        let payload = String::deserialize(deserializer)?;
        Ok(serde_json::from_str(&payload).unwrap_or_default())
    }
}
