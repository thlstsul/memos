use serde::Deserialize;

pub struct FindResource {
    pub id: i32,
    pub creator_id: i32,
    pub filename: String,
    pub memo_id: i32,
    pub has_relate_memo: bool,
    pub limit: isize,
    pub offset: isize,
}

#[derive(Debug, Default, Deserialize)]
pub struct WholeResource {
    pub filename: String,
    pub r#type: String,
    pub size: usize,
    pub creator_id: i32,
    pub blob: Option<Vec<u8>>,
    pub external_link: Option<String>,
    pub internal_path: Option<String>,
    pub id: Option<i32>,
    pub created_ts: Option<i64>,
    pub updated_ts: Option<i64>,
    pub memo_id: Option<i32>,
}
