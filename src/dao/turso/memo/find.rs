use sql_query_builder::Select;

use crate::{
    dao::turso::ToCriteria,
    model::{
        memo::{FindMemo, FindMemoPayload},
        pager::Paginator as _,
    },
};

impl ToCriteria for FindMemo {
    fn to_criteria(&self) -> (impl AsRef<str>, impl libsql::params::IntoParams) {
        let FindMemo {
            id,
            uid,
            state,
            creator_id,
            created_ts_after,
            created_ts_before,
            updated_ts_after,
            updated_ts_before,
            content_search,
            visibility_list,
            payload_find,
            exclude_content,
            page_token,
            order_by_updated_ts,
            order_by_pinned,
            only_payload,
            ..
        } = self;

        let mut sql = Select::new().from("memo");
        let mut params = Vec::new();

        if only_payload {
            sql = sql
                .select("memo.id AS id")
                .select("memo.payload AS payload")
                .select("memo.pinned AS pinned")
                .select("memo.created_ts AS created_ts")
                .select("memo.updated_ts AS updated_ts");
        } else {
            sql = sql
                .select("memo.id AS id")
                .select("memo.uid AS uid")
                .select("memo.creator_id AS creator_id")
                .select("memo.created_ts AS created_ts")
                .select("memo.updated_ts AS updated_ts")
                .select("memo.row_status AS row_status")
                .select("memo.visibility AS visibility")
                .select("memo.pinned AS pinned")
                .select("memo.payload AS payload");
        };

        if !exclude_content && !only_payload {
            sql = sql.select("memo.content AS content");
        }

        if let Some(id) = id {
            sql = sql.where_and("memo.id = ?");
            params.push(Value::from(id));
        }
        if let Some(uid) = uid {
            sql = sql.where_and("memo.uid = ?");
            params.push(Value::from(uid))
        }
        if let Some(creator_id) = creator_id {
            sql = sql.where_and("memo.creator_id = ?");
            params.push(Value::from(creator_id));
        }
        if let Some(state) = &state {
            sql = sql.where_and("memo.row_status = ?");
            params.push(Value::from(state.to_string()));
        }
        if let Some(created_ts_before) = created_ts_before {
            sql = sql.where_and("memo.created_ts < ?");
            params.push(Value::from(created_ts_before));
        }
        if let Some(created_ts_after) = created_ts_after {
            sql = sql.where_and("memo.created_ts > ?");
            params.push(Value::from(created_ts_after));
        }
        if let Some(updated_ts_before) = updated_ts_before {
            sql = sql.where_and("memo.updated_ts < ?");
            params.push(Value::from(updated_ts_before));
        }
        if let Some(updated_ts_after) = updated_ts_after {
            sql = sql.where_and("memo.updated_ts > ?");
            params.push(Value::from(updated_ts_after));
        }
        for content_search in content_search.iter() {
            sql = sql.where_and("memo.content LIKE ?");
            params.push(Value::from(format!("%{content_search}%")));
        }

        let w;
        if !visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in visibility_list.iter() {
                params.push(Value::from(visibility.as_str_name().to_owned()));
                l.push("?");
            }
            w = format!("memo.visibility in ({})", l.join(", "));
            sql = sql.where_and(w.as_str());
        }

        if let Some(FindMemoPayload {
            raw,
            tags,
            has_link,
            has_task_list,
            has_code,
            has_incomplete_tasks,
        }) = payload_find
        {
            if let Some(raw) = raw {
                sql = sql.where_and("memo.payload = ?");
                params.push(Value::from(raw));
            }
            if let Some(tags) = tags {
                sql = sql.from("JSON_EACH(memo.payload, '$.property.tags')");
                for tag in tags {
                    sql = sql.where_and("JSON_EACH.value = ?");
                    params.push(Value::from(tag));
                }
            }
            if has_link {
                sql = sql.where_and("JSON_EXTRACT(memo.payload, '$.property.has_link') IS TRUE");
            }
            if has_task_list {
                sql =
                    sql.where_and("JSON_EXTRACT(memo.payload, '$.property.has_task_list') IS TRUE");
            }
            if has_code {
                sql = sql.where_and("JSON_EXTRACT(memo.payload, '$.property.has_code') IS TRUE");
            }
            if has_incomplete_tasks {
                sql = sql.where_and(
                    "JSON_EXTRACT(memo.payload, '$.property.has_incomplete_tasks') IS TRUE",
                );
            }
        }

        if order_by_pinned {
            sql = sql.order_by("pinned DESC");
        }
        if order_by_updated_ts {
            sql = sql.order_by("memo.updated_ts DESC");
        } else {
            sql = sql.order_by("memo.created_ts DESC");
        }
        sql = sql.order_by("memo.id DESC");

        if let Some(page_token) = page_token {
            sql = sql.limit(page_token.limit()).offset(page_token.offset());
        }

        (sql, params)
    }
}
