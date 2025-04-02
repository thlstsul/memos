use sql_query_builder::Update;

use crate::{dao::turso::ToCriteria, model::memo::UpdateMemo};

impl ToCriteria for UpdateMemo {
    fn to_criteria(&self) -> (impl AsRef<str>, impl libsql::params::IntoParams) {
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
        let mut sql = Update::new().table("memo").where_and("id = ?");
        params.push(Value::from(id));

        if let Some(visibility) = visibility {
            sql.set("visibility = ?");
            params.push(Value::from(visibility.as_str_name().to_owned()));
        }

        if let Some(state) = state {
            sql.set("row_status = ?");
            params.push(Value::from(state.as_str_name().to_owned()));
        }

        if let Some(content) = content {
            sql.set("content = ?");
            params.push(Value::from(content));
        }

        if let Some(payload) = payload {
            sql.set("payload = ?");
            params.push(payload.into());
        }

        if let Some(pinned) = pinned {
            sql.set("pinned = ?");
            params.push(Value::from(if pinned { 1 } else { 0 }));
        }

        if params.len() = 1 {
            (..Default::default(), ..Default::default())
        } else {
            (sql, params)
        }
    }
}
