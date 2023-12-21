use super::{v1::memo::ListMemoRequest, v2::Visibility};
use serde::{de::Error, Deserialize, Deserializer, Serialize};

pub struct FindMemo {
    pub id: Option<i32>,

    // Standard fields
    pub creator_id: Option<i32>,
    pub row_status: Option<String>,
    pub created_ts_after: Option<i64>,
    pub created_ts_before: Option<i64>,

    // Domain specific fields
    pub pinned: bool,
    pub content_search: Vec<String>,
    pub has_parent: Option<bool>,
    pub visibility_list: Vec<Visibility>,
    pub exclude_content: bool,

    // Pagination
    pub limit: Option<isize>,
    pub offset: Option<isize>,
    pub order_by_updated_ts: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Memo {
    content: Option<String>,
    #[serde(rename(serialize = "createdTs"))]
    created_ts: i64,
    #[serde(rename(serialize = "creatorID"))]
    creator_id: i32,
    id: i32,
    #[serde(rename(serialize = "parentID"))]
    parent_id: i32,
    pinned: bool,
    #[serde(
        rename(serialize = "relationList"),
        deserialize_with = "deserialize_relation"
    )]
    relation_list: Vec<Relation>,
    #[serde(
        rename(serialize = "resourceIDList"),
        deserialize_with = "deserialize_resource_id"
    )]
    resource_id_list: Vec<i32>,
    #[serde(rename(serialize = "rowStatus"))]
    row_status: String,
    #[serde(rename(serialize = "updatedTs"))]
    updated_ts: i64,
    visibility: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Relation {
    #[serde(rename(serialize = "memoID"))]
    memo_id: i32,
    #[serde(rename(serialize = "relatedMemoID"))]
    related_memo_id: i32,
    r#type: String,
}

fn deserialize_relation<'de, D>(deserializer: D) -> Result<Vec<Relation>, D::Error>
where
    D: Deserializer<'de>,
{
    let relation_str = String::deserialize(deserializer)?;
    let mut relation_list = Vec::new();
    if !relation_str.is_empty() {
        let rows: Vec<&str> = relation_str.split(",").collect();
        for row_str in rows {
            let row: Vec<&str> = row_str.split(":").collect();
            if row.len() != 3 {
                return Err(Error::custom("invalid relation format"));
            }
            let memo_id: i32 = row[0].parse().map_err(|e| Error::custom(e))?;
            let related_memo_id: i32 = row[1].parse().map_err(|e| Error::custom(e))?;
            let relation = Relation {
                memo_id,
                related_memo_id,
                r#type: row[2].to_owned(),
            };
            relation_list.push(relation);
        }
    }

    Ok(relation_list)
}

pub fn deserialize_resource_id<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let resource_id_str = String::deserialize(deserializer)?;
    let resource_id_list: Result<Vec<i32>, _> =
        resource_id_str.split(",").map(|id| id.parse()).collect();
    Ok(resource_id_list.map_err(|e| Error::custom(e))?)
}

impl Into<FindMemo> for ListMemoRequest {
    fn into(self) -> FindMemo {
        let mut content_search = Vec::new();
        if let Some(tag) = self.tag {
            content_search.push(format!("#{tag}"));
        }
        if let Some(content) = self.content {
            content_search.push(content);
        }
        let limit = self.limit.or(Some(20));
        FindMemo {
            id: None,
            creator_id: self.creator_id,
            row_status: self.row_status,
            created_ts_after: None,
            created_ts_before: None,
            pinned: self.pinned,
            content_search,
            has_parent: None,
            visibility_list: Vec::new(),
            exclude_content: false,
            limit,
            offset: self.offset,
            order_by_updated_ts: false,
        }
    }
}
