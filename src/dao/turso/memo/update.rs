use libsql::{params::IntoParams, Value};
use sql_query_builder::Update;

use crate::{dao::turso::ToCriteria, model::memo::UpdateMemo};

impl ToCriteria for UpdateMemo {
    fn to_criteria(self) -> (impl AsRef<str>, impl IntoParams) {
        let UpdateMemo {
            id,
            creator_id,
            content,
            visibility,
            state,
            pinned,
            payload,
        } = self;

        let mut params = Vec::new();
        let mut sql = Update::new().update("memo").where_and("id = ?");
        params.push(Value::from(id));

        if let Some(visibility) = visibility {
            sql = sql.set("visibility = ?");
            params.push(Value::from(visibility.as_str_name().to_owned()));
        }

        if let Some(state) = state {
            sql = sql.set("row_status = ?");
            params.push(Value::from(state.as_str_name().to_owned()));
        }

        if let Some(content) = content {
            sql = sql.set("content = ?");
            params.push(Value::from(content));
        }

        if let Some(payload) = payload {
            sql = sql.set("payload = ?");
            params.push(payload.into());
        }

        if let Some(pinned) = pinned {
            sql = sql.set("pinned = ?");
            params.push(Value::from(if pinned { 1 } else { 0 }));
        }

        if params.len() == 1 {
            (String::default(), Vec::default())
        } else {
            (sql.as_string(), params)
        }
    }
}
