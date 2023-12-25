use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tracing::info;

use libsql_client::{Client, Statement};

use crate::api::{
    memo::{CreateMemo, FindMemo},
    v2::Memo,
};

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
    pub async fn create_memo(&self, create: CreateMemo) -> Result<Memo, Error> {
        let stmt = Statement::with_args(
            "INSERT INTO memo (creator_id, content, visibility) VALUES (?, ?, ?) RETURNING id, creator_id, created_ts as create_time, updated_ts as update_time, row_status, content, visibility",
            &[
                create.creator_id.to_string(),
                create.content,
                create.visibility.as_str_name().to_owned(),
            ],
        );

        let mut memos: Vec<Memo> = self.execute(stmt).await.context(Database)?;
        if let Some(memo) = memos.pop() {
            Ok(memo)
        } else {
            Err(Error::CreateMemoFailed)
        }
    }

    pub async fn list_memos(&self, cond: FindMemo) -> Result<Vec<Memo>, Error> {
        let stmt = cond.into_list_stmt();
        info!("{stmt}");
        self.execute(stmt).await.context(Database)
    }
}

impl FindMemo {
    fn into_list_stmt(&self) -> Statement {
        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = self.id {
            wheres.push("memo.id = ?");
            args.push(id.to_string());
        }
        if let Some(creator_id) = self.creator_id {
            wheres.push("memo.creator_id = ?");
            args.push(creator_id.to_string());
        }
        if let Some(row_status) = &self.row_status {
            wheres.push("memo.row_status = ?");
            args.push(row_status.clone());
        }
        if let Some(created_ts_before) = self.created_ts_before {
            wheres.push("memo.created_ts < ?");
            args.push(created_ts_before.to_string());
        }
        if let Some(created_ts_after) = self.created_ts_after {
            wheres.push("memo.created_ts > ?");
            args.push(created_ts_after.to_string());
        }
        for content_search in self.content_search.iter() {
            wheres.push("memo.content LIKE ?");
            args.push(format!("%{content_search}%"));
        }
        if self.pinned {
            wheres.push("memo_organizer.pinned = 1");
        }

        let mut wheres: Vec<String> = wheres.into_iter().map(|s| s.to_owned()).collect();

        if !self.visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in self.visibility_list.iter() {
                args.push(visibility.as_str_name().to_string());
                l.push("?");
            }
            wheres.push(format!("memo.visibility in ({})", l.join(", ")));
        }

        let mut orders = Vec::new();
        if self.order_by_pinned {
            orders.push("pinned DESC");
        }
        if self.order_by_updated_ts {
            orders.push("updated_ts DESC");
        } else {
            orders.push("created_ts DESC");
        }
        orders.push("id DESC");

        let mut fields = vec![
            "memo.id AS id",
            "memo.creator_id AS creator_id",
            "memo.created_ts AS create_time",
            "memo.updated_ts AS update_time",
            "memo.row_status AS row_status",
            "memo.visibility AS visibility",
            "CASE WHEN memo_organizer.pinned = 1 THEN 1 ELSE 0 END AS pinned",
        ];

        if !self.exclude_content {
            fields.push("memo.content AS content");
        }

        let mut query = format!(
            "SELECT
            {}
            FROM memo
		    LEFT JOIN memo_organizer ON memo.id = memo_organizer.memo_id
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
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
    #[snafu(display("Create memo failed"), context(suffix(false)))]
    CreateMemoFailed,
}
