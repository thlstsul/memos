use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetMemoRequest {
    #[serde(rename = "creatorId")]
    pub creator_id: Option<i32>,
    #[serde(rename = "creatorUsername")]
    pub creator_username: Option<String>,
    #[serde(rename = "rowStatus")]
    pub row_status: Option<String>,
    pub pinned: Option<bool>,
    pub tag: Option<String>,
    pub content: Option<String>,
    pub limit: i32,
    pub offset: Option<i32>,
}
