use serde::Deserialize;

#[derive(Debug, Default)]
pub struct FindResource {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub creator_id: Option<i32>,
    pub filename: Option<String>,
    pub memo_id: Option<i32>,
    pub limit: Option<isize>,
    pub offset: Option<isize>,
    pub has_relate_memo: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct WholeResource {
    pub filename: String,
    pub r#type: String,
    pub size: usize,
    pub creator_id: i32,
    pub blob: Vec<u8>,
    pub external_link: String,
    pub internal_path: String,
    pub id: i32,
    pub resource_name: String,
    pub created_ts: i64,
    pub updated_ts: i64,
    #[serde(deserialize_with = "crate::api::option_serde::deserialize")]
    pub memo_id: Option<i32>,
}
