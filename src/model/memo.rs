use crate::util::ast;

use crate::api::v1::gen::{PageToken, RowStatus, Visibility};

use syn::{
    parse::{Parse, ParseStream},
    parse_quote, BinOp, Expr,
};

use super::gen::memo_payload::Property;
use super::gen::MemoPayload;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct Memo {
    pub id: i32,
    pub uid: String,
    pub row_status: RowStatus,
    pub creator_id: i32,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub content: String,
    pub visibility: Visibility,
    #[serde(deserialize_with = "crate::model::bool_serde::deserialize")]
    pub pinned: bool,
    #[serde(deserialize_with = "crate::model::memo::payload_serde::deserialize")]
    pub payload: MemoPayload,
}

#[derive(Debug, Default)]
pub struct FindMemo {
    pub id: Option<i32>,
    pub uid: Option<String>,

    // Standard fields
    pub row_status: Option<RowStatus>,
    pub creator_id: Option<i32>,
    pub created_ts_after: Option<i64>,
    pub created_ts_before: Option<i64>,
    pub updated_ts_after: Option<i64>,
    pub updated_ts_before: Option<i64>,

    // Domain specific fields
    pub content_search: Vec<String>,
    pub visibility_list: Vec<Visibility>,
    pub payload_find: Option<FindMemoPayload>,
    pub exclude_content: bool,
    pub exclude_comments: bool,
    pub random: bool,

    // Pagination
    pub page_token: Option<PageToken>,
    pub order_by_updated_ts: bool,
    pub order_by_pinned: bool,

    // Custom
    pub only_payload: bool,
}

#[derive(Debug, Default)]
pub struct FindMemoPayload {
    pub raw: Option<String>,
    pub tags: Option<Vec<String>>,
    pub has_link: bool,
    pub has_task_list: bool,
    pub has_code: bool,
    pub has_incomplete_tasks: bool,
}

pub struct CreateMemo {
    pub creator_id: i32,
    pub uid: String,
    pub content: String,
    pub visibility: Visibility,
    pub payload: MemoPayload,
}

#[derive(Debug, Default)]
pub struct UpdateMemo {
    pub id: i32,
    pub creator_id: i32,
    pub content: Option<String>,
    pub visibility: Option<Visibility>,
    pub row_status: Option<RowStatus>,
    pub pinned: Option<bool>,
    pub payload: Option<MemoPayload>,
}

#[derive(Debug, Default)]
pub struct SearchMemosFilter {
    pub content_search: Option<Vec<String>>,
    pub visibilities: Option<Vec<Visibility>>,
    pub tag_search: Option<Vec<String>>,
    pub order_by_pinned: Option<bool>,
    pub display_time_before: Option<i64>,
    pub display_time_after: Option<i64>,
    pub creator: Option<String>,
    pub uid: Option<String>,
    pub row_status: Option<RowStatus>,
    pub random: Option<bool>,
    pub limit: Option<i32>,
    pub include_comments: Option<bool>,
    pub has_link: Option<bool>,
    pub has_task_list: Option<bool>,
    pub has_code: Option<bool>,
    pub has_incomplete_tasks: Option<bool>,
}

impl Parse for SearchMemosFilter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut binary = Expr::parse(input)?;

        let mut filter = SearchMemosFilter::default();
        let mut get_value = |ident: Expr, lit: Expr| {
            if ident == parse_quote!(creator) {
                filter.creator = ast::get_string(lit);
            } else if ident == parse_quote!(row_status) {
                filter.row_status = ast::get_row_status(lit);
            } else if ident == parse_quote!(visibilities) {
                filter.visibilities = ast::get_visibilities(lit);
            } else if ident == parse_quote!(order_by_pinned) {
                filter.order_by_pinned = ast::get_bool(lit);
            } else if ident == parse_quote!(content_search) {
                filter.content_search = ast::get_string_list(lit);
            } else if ident == parse_quote!(display_time_before) {
                filter.display_time_before = ast::get_int(lit);
            } else if ident == parse_quote!(display_time_after) {
                filter.display_time_after = ast::get_int(lit);
            } else if ident == parse_quote!(tag_search) {
                filter.tag_search = ast::get_string_list(lit);
            } else if ident == parse_quote!(uid) {
                filter.uid = ast::get_string(lit);
            } else if ident == parse_quote!(random) {
                filter.random = ast::get_bool(lit);
            } else if ident == parse_quote!(limit) {
                filter.limit = ast::get_int(lit);
            } else if ident == parse_quote!(include_comments) {
                filter.include_comments = ast::get_bool(lit);
            } else if ident == parse_quote!(has_link) {
                filter.has_link = ast::get_bool(lit);
            } else if ident == parse_quote!(has_task_list) {
                filter.has_task_list = ast::get_bool(lit);
            } else if ident == parse_quote!(has_code) {
                filter.has_code = ast::get_bool(lit);
            } else if ident == parse_quote!(has_incomplete_tasks) {
                filter.has_incomplete_tasks = ast::get_bool(lit);
            }
        };

        while let Expr::Binary(ref bin) = binary {
            let left = *bin.left.clone();
            let right = *bin.right.clone();

            if matches!(bin.op, BinOp::And(_)) {
                if let Expr::Binary(bin) = right {
                    let ident = *bin.left;
                    let lit = *bin.right;
                    get_value(ident, lit);
                }
                binary = left;
            } else {
                get_value(left, right);
                break;
            }
        }

        Ok(filter)
    }
}

impl FindMemo {
    pub fn completed(&mut self, user_id: Option<i32>, is_display_with_update_time: bool) {
        if let Some(user_id) = user_id {
            if self.creator_id.is_some() {
                if Some(user_id) != self.creator_id {
                    self.visibility_list = vec![Visibility::Public, Visibility::Protected];
                }
            } else {
                self.creator_id = Some(user_id);
            }
        } else {
            self.visibility_list = vec![Visibility::Public];
        }
        if self.id.is_none() {
            self.order_by_updated_ts = is_display_with_update_time;
            if self.order_by_updated_ts {
                self.updated_ts_after = self.updated_ts_after.or(self.created_ts_after);
                self.updated_ts_before = self.updated_ts_before.or(self.created_ts_before);
                self.created_ts_after = None;
                self.created_ts_before = None;
            } else {
                self.created_ts_after = self.created_ts_after.or(self.updated_ts_after);
                self.created_ts_before = self.created_ts_before.or(self.updated_ts_before);
                self.updated_ts_after = None;
                self.updated_ts_before = None;
            }
        }
    }
}

impl MemoPayload {
    pub fn merge(&mut self, mut payload: MemoPayload) {
        if self.property.is_none() && payload.property.is_some() {
            self.property = payload.property;
        } else if let MemoPayload {
            property: Some(ref mut property2),
        } = payload
        {
            if let MemoPayload {
                property: Some(ref mut property1),
            } = self
            {
                property1.tags.append(&mut property2.tags);
                property1.has_code = property1.has_code || property2.has_code;
                property1.has_link = property1.has_link || property2.has_link;
                property1.has_task_list = property1.has_task_list || property2.has_task_list;
                property1.has_incomplete_tasks =
                    property1.has_incomplete_tasks || property2.has_incomplete_tasks;
            }
        }
    }

    #[allow(dead_code)]
    pub fn tag(tag: String) -> Self {
        Self {
            property: Some(Property {
                tags: vec![tag],
                ..Default::default()
            }),
        }
    }

    pub fn tags(tags: Vec<String>) -> Self {
        if tags.is_empty() {
            Self { property: None }
        } else {
            Self {
                property: Some(Property {
                    tags,
                    ..Default::default()
                }),
            }
        }
    }

    pub fn link() -> Self {
        Self {
            property: Some(Property {
                has_link: true,
                ..Default::default()
            }),
        }
    }

    pub fn code() -> Self {
        Self {
            property: Some(Property {
                has_code: true,
                ..Default::default()
            }),
        }
    }

    pub fn task() -> Self {
        Self {
            property: Some(Property {
                has_task_list: true,
                ..Default::default()
            }),
        }
    }

    pub fn incomplete_task() -> Self {
        Self {
            property: Some(Property {
                has_task_list: true,
                has_incomplete_tasks: true,
                ..Default::default()
            }),
        }
    }
}

impl From<MemoPayload> for libsql::Value {
    fn from(val: MemoPayload) -> Self {
        libsql::Value::Text(serde_json::to_string(&val).unwrap_or("{}".to_string()))
    }
}

pub mod payload_serde {
    use crate::model::gen::MemoPayload;
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S>(payload: &MemoPayload, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = serde_json::to_string(payload).unwrap_or("{}".to_string());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<MemoPayload, D::Error>
    where
        D: Deserializer<'de>,
    {
        let payload = String::deserialize(deserializer)?;
        Ok(serde_json::from_str(&payload).unwrap_or_default())
    }
}

mod test {
    #[test]
    fn parse_filter() {
        use crate::api::v1::gen::{RowStatus, Visibility};
        use crate::model::memo::SearchMemosFilter;

        let filter =
        r#"visibilities == ['PUBLIC'] && row_status == "NORMAL" && creator == "users/THELOSTSOUL" && order_by_pinned == true && display_time_before == 123 && tag_search == ["TODO"]"#
            .replace('\'', "\"");
        let filter = syn::parse_str::<SearchMemosFilter>(&filter).unwrap();
        assert_eq!(filter.row_status, Some(RowStatus::Active));
        assert_eq!(filter.visibilities, Some(vec![Visibility::Public]));
        assert_eq!(filter.creator, Some("users/THELOSTSOUL".to_string()));
        assert_eq!(filter.order_by_pinned, Some(true));
        assert_eq!(filter.display_time_before, Some(123_i64));
        assert_eq!(filter.tag_search, Some(vec!["TODO".to_string()]));
    }
}
