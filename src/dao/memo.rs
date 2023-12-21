use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tracing::info;

use libsql_client::{Client, Statement};

use crate::api::memo::{FindMemo, Memo};

use super::Dao;

pub struct MemoDao {
    pub client: Arc<Client>,
}

impl Dao for MemoDao {
    fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

impl MemoDao {
    pub async fn list_memos(&self, cond: FindMemo) -> Result<Vec<Memo>, Error> {
        let stmt = cond.into_list_stmt();
        info!("{stmt}");
        self.execute(stmt).await.context(Database)
    }
}

impl FindMemo {
    fn into_list_stmt(&self) -> Statement {
        let mut wheres = Vec::new();
        let mut args = Vec::new();

        if let Some(id) = self.id {
            wheres.push("memo.id = ?".to_owned());
            args.push(id.to_string());
        }
        if let Some(creator_id) = self.creator_id {
            wheres.push("memo.creator_id = ?".to_owned());
            args.push(creator_id.to_string());
        }
        if let Some(row_status) = &self.row_status {
            wheres.push("memo.row_status = ?".to_owned());
            args.push(row_status.clone());
        }
        if let Some(created_ts_before) = self.created_ts_before {
            wheres.push("memo.created_ts < ?".to_owned());
            args.push(created_ts_before.to_string());
        }
        if let Some(created_ts_after) = self.created_ts_after {
            wheres.push("memo.created_ts > ?".to_owned());
            args.push(created_ts_after.to_string());
        }
        for content_search in self.content_search.iter() {
            wheres.push("memo.content LIKE ?".to_owned());
            args.push(format!("%{content_search}%"));
        }
        if !self.visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in self.visibility_list.iter() {
                args.push(visibility.as_str_name().to_string());
                l.push("?");
            }
            wheres.push(format!("memo.visibility in ({})", l.join(", ")));
        }
        if self.pinned {
            wheres.push("memo_organizer.pinned = 1".to_owned());
        }
        if let Some(has_parent) = self.has_parent {
            if has_parent {
                wheres.push("parent_id IS NOT NULL".to_owned());
            } else {
                wheres.push("parent_id IS NULL".to_owned());
            }
        }

        let mut orders = Vec::new();
        orders.push("pinned DESC".to_owned());
        if self.order_by_updated_ts {
            orders.push("updated_ts DESC".to_owned());
        } else {
            orders.push("created_ts DESC".to_owned());
        }
        orders.push("id DESC".to_owned());

        let mut fields = vec![
            "memo.id AS id".to_owned(),
            "memo.creator_id AS creator_id".to_owned(),
            "memo.created_ts AS created_ts".to_owned(),
            "memo.updated_ts AS updated_ts".to_owned(),
            "memo.row_status AS row_status".to_owned(),
            "memo.visibility AS visibility".to_owned(),
        ];

        if !self.exclude_content {
            fields.push("memo.content AS content".to_owned());
        }

        let mut query = format!(
            "SELECT
            {},
            CASE WHEN mo.pinned = 1 THEN 1 ELSE 0 END AS pinned,
            (
                    SELECT
                        related_memo_id
                    FROM
                        memo_relation
                    WHERE
                        memo_relation.memo_id = memo.id AND memo_relation.type = 'COMMENT'
                    LIMIT 1
            ) AS parent_id,
            GROUP_CONCAT(resource.id) AS resource_id_list,
            (
                    SELECT
                        GROUP_CONCAT(memo_relation.memo_id || ':' || memo_relation.related_memo_id || ':' || memo_relation.type)
                    FROM
                        memo_relation
                    WHERE
                        memo_relation.memo_id = memo.id OR memo_relation.related_memo_id = memo.id
            ) AS relation_list
                FROM
                    memo
                LEFT JOIN
                    memo_organizer mo ON memo.id = mo.memo_id
                LEFT JOIN
                    resource ON memo.id = resource.memo_id
                WHERE {}
                GROUP BY memo.id
                ORDER BY {}",
            fields.join(",\n"),
            wheres.join(" AND "),
            orders.join(", ")
        );

        if let Some(limit) = self.limit {
            query = format!("{query} LIMIT {limit}");
            if let Some(offset) = self.offset {
                query = format!("{query} OFFSET {offset}");
            }
        }

        Statement::with_args(query, &args)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
