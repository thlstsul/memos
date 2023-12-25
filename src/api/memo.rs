use super::v2::{CreateMemoResponse, ListMemosRequest, Memo, RowStatus, Visibility};
use serde::{de::Error, Deserialize, Deserializer, Serialize};

pub struct FindMemo {
    pub id: Option<i32>,

    // Standard fields
    pub creator_id: Option<i32>,
    pub row_status: Option<RowStatus>,
    pub created_ts_after: Option<i64>,
    pub created_ts_before: Option<i64>,

    // Domain specific fields
    pub pinned: bool,
    pub content_search: Vec<String>,
    pub visibility_list: Vec<Visibility>,
    pub exclude_content: bool,

    // Pagination
    pub limit: Option<isize>,
    pub offset: Option<isize>,
    pub order_by_updated_ts: bool,
    pub order_by_pinned: bool,
}

pub struct CreateMemo {
    pub creator_id: i32,
    pub content: String,
    pub visibility: Visibility,
}

#[derive(Deserialize)]
pub struct Filter {
    pub content_search: Option<Vec<String>>,
    pub visibilities: Option<Vec<Visibility>>,
    pub order_by_pinned: Option<bool>,
    pub created_ts_before: Option<i64>,
    pub created_ts_after: Option<i64>,
    pub creator: Option<String>,
    pub row_status: Option<RowStatus>,
}

impl Into<FindMemo> for ListMemosRequest {
    fn into(self) -> FindMemo {
        todo!("serde_urlencoded")
    }
}

impl Into<CreateMemoResponse> for Memo {
    fn into(self) -> CreateMemoResponse {
        CreateMemoResponse { memo: Some(self) }
    }
}
