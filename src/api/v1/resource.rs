use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResourceResponse {
    pub id: i32,
    pub creator_id: i32,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub filename: String,
    pub external_link: String,
    pub r#type: String,
    pub size: i64,
}
