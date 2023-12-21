use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ListMemoRequest {
    #[serde(rename = "creatorId")]
    pub creator_id: Option<i32>,
    #[serde(rename = "creatorUsername")]
    pub creator_username: Option<String>,
    #[serde(rename = "rowStatus")]
    pub row_status: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    pub tag: Option<String>,
    pub content: Option<String>,
    pub limit: Option<isize>,
    pub offset: Option<isize>,
}
