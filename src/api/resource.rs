use serde::Deserialize;

#[derive(Debug, Default)]
pub struct FindResource {
    pub id: Option<i32>,
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
    pub created_ts: i64,
    pub updated_ts: i64,
    pub memo_id: Option<i32>,
}
