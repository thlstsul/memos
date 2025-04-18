use libsql::params;
use libsql::params::IntoParams;
use sql_query_builder::Insert;

use crate::{dao::turso::ToCriteria, model::memo::CreateMemo};

impl ToCriteria for CreateMemo {
    fn to_criteria(self) -> (impl AsRef<str>, impl IntoParams) {
        let CreateMemo {
            creator_id,
            uid,
            content,
            visibility,
            payload,
        } = self;

        let sql = Insert::new()
            .insert_into("memo (creator_id, uid, content, visibility, payload)")
            .values("(?, ?, ?, ?, ?)")
            .returning("id, uid, creator_id, created_ts, updated_ts, row_status, content, visibility, payload");
        let params = params![creator_id, uid, content, visibility.as_str_name(), payload];
        (sql.as_string(), params)
    }
}
