use async_trait::async_trait;
use libsql::{params, Value};

use crate::dao::memo::{
    CreateMemoError, DeleteMemoError, ListMemoError, MemoRepository, UpdateMemoError,
};
use crate::model::memo::FindMemoPayload;
use crate::model::{
    memo::{CreateMemo, FindMemo, Memo, UpdateMemo},
    pager::Paginator,
};

use super::Turso;

#[async_trait]
impl MemoRepository for Turso {
    async fn create_memo(
        &self,
        CreateMemo {
            creator_id,
            uid,
            content,
            visibility,
            payload,
        }: CreateMemo,
    ) -> Result<Option<Memo>, CreateMemoError> {
        let mut memos: Vec<Memo> = self.query(
            "INSERT INTO memo (creator_id, uid, content, visibility, payload) VALUES (?, ?, ?, ?, ?) RETURNING id, uid, creator_id, created_ts, updated_ts, row_status, content, visibility, payload",
            params![creator_id, uid, content, visibility.as_str_name(), payload]
        ).await?;

        Ok(memos.pop())
    }

    async fn list_memos(
        &self,
        FindMemo {
            id,
            uid,
            row_status,
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
        }: FindMemo,
    ) -> Result<Vec<Memo>, ListMemoError> {
        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();
        let mut tables = vec!["memo"];

        if let Some(id) = id {
            wheres.push("memo.id = ?");
            args.push(Value::from(id));
        }
        if let Some(uid) = uid {
            wheres.push("memo.uid = ?");
            args.push(Value::from(uid))
        }
        if let Some(creator_id) = creator_id {
            wheres.push("memo.creator_id = ?");
            args.push(Value::from(creator_id));
        }
        if let Some(row_status) = &row_status {
            wheres.push("memo.row_status = ?");
            args.push(Value::from(row_status.to_string()));
        }
        if let Some(created_ts_before) = created_ts_before {
            wheres.push("memo.created_ts < ?");
            args.push(Value::from(created_ts_before));
        }
        if let Some(created_ts_after) = created_ts_after {
            wheres.push("memo.created_ts > ?");
            args.push(Value::from(created_ts_after));
        }
        if let Some(updated_ts_before) = updated_ts_before {
            wheres.push("memo.updated_ts < ?");
            args.push(Value::from(updated_ts_before));
        }
        if let Some(updated_ts_after) = updated_ts_after {
            wheres.push("memo.updated_ts > ?");
            args.push(Value::from(updated_ts_after));
        }
        for content_search in content_search.iter() {
            wheres.push("memo.content LIKE ?");
            args.push(Value::from(format!("%{content_search}%")));
        }

        let w;
        if !visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in visibility_list.iter() {
                args.push(Value::from(visibility.as_str_name().to_owned()));
                l.push("?");
            }
            w = format!("memo.visibility in ({})", l.join(", "));
            wheres.push(w.as_str());
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
                wheres.push("memo.payload = ?");
                args.push(Value::from(raw));
            }
            if let Some(tags) = tags {
                tables.push("JSON_EACH(memo.payload, '$.property.tags')");
                for tag in tags {
                    wheres.push("JSON_EACH.value = ?");
                    args.push(Value::from(tag));
                }
            }
            if has_link {
                wheres.push("JSON_EXTRACT(memo.payload, '$.property.has_link') IS TRUE");
            }
            if has_task_list {
                wheres.push("JSON_EXTRACT(memo.payload, '$.property.has_task_list') IS TRUE");
            }
            if has_code {
                wheres.push("JSON_EXTRACT(memo.payload, '$.property.has_code') IS TRUE");
            }
            if has_incomplete_tasks {
                wheres
                    .push("JSON_EXTRACT(memo.payload, '$.property.has_incomplete_tasks') IS TRUE");
            }
        }

        let mut orders = Vec::new();
        if order_by_pinned {
            orders.push("pinned DESC");
        }
        if order_by_updated_ts {
            orders.push("memo.updated_ts DESC");
        } else {
            orders.push("memo.created_ts DESC");
        }
        orders.push("memo.id DESC");

        let mut fields = if only_payload {
            vec![
                "memo.id AS id",
                "memo.payload AS payload",
                "memo.created_ts AS created_ts",
                "memo.updated_ts AS updated_ts",
            ]
        } else {
            vec![
                "memo.id AS id",
                "memo.uid AS uid",
                "memo.creator_id AS creator_id",
                "memo.created_ts AS created_ts",
                "memo.updated_ts AS updated_ts",
                "memo.row_status AS row_status",
                "memo.visibility AS visibility",
                "CASE WHEN memo_organizer.pinned = 1 THEN 1 ELSE 0 END AS pinned",
                "memo.payload AS payload",
            ]
        };

        if !exclude_content && !only_payload {
            fields.push("memo.content AS content");
        }

        let mut sql = format!(
            "select {f} from {t} {j} where {w} group by memo.id order by {o}",
            f = fields.join(",\n"),
            t = tables.join(", "),
            j = if !only_payload {
                "left join memo_organizer on memo.id = memo_organizer.memo_id"
            } else {
                ""
            },
            w = wheres.join(" AND "),
            o = orders.join(", ")
        );

        if let Some(page_token) = page_token {
            sql = format!("{sql} {}", page_token.as_limit_sql());
        }
        Ok(self.query(&sql, args).await?)
    }

    async fn delete_memo(&self, memo_id: i32) -> Result<(), DeleteMemoError> {
        let sql = "delete from memo where id = ?";
        self.execute(sql, [memo_id]).await?;
        Ok(())
    }

    async fn update_memo(
        &self,
        UpdateMemo {
            id,
            creator_id,
            content,
            visibility,
            row_status,
            pinned,
            payload,
        }: UpdateMemo,
    ) -> Result<(), UpdateMemoError> {
        {
            let mut set = Vec::new();
            let mut args = Vec::new();

            if let Some(visibility) = visibility {
                set.push("visibility = ?");
                args.push(Value::from(visibility.as_str_name().to_owned()));
            }

            if let Some(row_status) = row_status {
                set.push("row_status = ?");
                args.push(Value::from(row_status.as_str_name().to_owned()));
            }

            if let Some(content) = content {
                set.push("content = ?");
                args.push(Value::from(content));
            }

            if let Some(payload) = payload {
                set.push("payload = ?");
                args.push(payload.into());
            }

            if !set.is_empty() {
                let update_sql = format!("UPDATE memo SET {} WHERE id = ?", set.join(", "));
                args.push(Value::from(id));
                self.execute(&update_sql, args).await?;
            }
        }

        if let Some(pinned) = pinned {
            // 置顶是单独操作的
            let sql = "
                INSERT INTO memo_organizer (
		        	memo_id,
		        	user_id,
		        	pinned
		        )
		        VALUES (?, ?, ?)
		        ON CONFLICT(memo_id, user_id) DO UPDATE
		        SET
		        	pinned = EXCLUDED.pinned
            ";
            self.execute(sql, [id, creator_id, if pinned { 1 } else { 0 }])
                .await?;
        }

        Ok(())
    }
}
