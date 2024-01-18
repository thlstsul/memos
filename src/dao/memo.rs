use snafu::{OptionExt, ResultExt, Snafu};
use tracing::info;

use libsql_client::{de, Statement, Value};

use crate::{
    api::{
        memo::{CreateMemo, FindMemo, UpdateMemo},
        v2::{node, Memo, TagNode},
        Count,
    },
    state::AppState,
    util::ast::parse_document,
};

use super::Dao;

pub struct MemoDao {
    pub state: AppState,
}

impl Dao for MemoDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl MemoDao {
    pub async fn create_memo(
        &self,
        CreateMemo {
            creator_id,
            content,
            visibility,
        }: CreateMemo,
    ) -> Result<Memo, Error> {
        let mut stmts = vec![Statement::with_args(
            "INSERT INTO memo (creator_id, content, visibility) VALUES (?, ?, ?) RETURNING id, creator_id, created_ts as create_time, created_ts as display_time, updated_ts as update_time, row_status, content, visibility",
            &[
                Value::from(creator_id),
                Value::from(content.clone()),
                Value::from(visibility.as_str_name().to_owned()),
            ],
        )];

        stmts.append(&mut parse_upsert_tag(creator_id, &content));

        let rss = self.state.batch(stmts).await.context(Database)?;
        if let Some(rs) = rss.first() {
            let mut memos = rs
                .rows
                .iter()
                .map(de::from_row)
                .collect::<Result<Vec<Memo>, _>>()
                .context(Database)?;
            memos.pop().context(CreateMemoFailed)
        } else {
            CreateMemoFailed.fail()
        }
    }

    pub async fn list_memos(&self, find: FindMemo) -> Result<Vec<Memo>, Error> {
        let stmt: Statement = find.into();
        info!("{stmt}");
        self.execute(stmt).await.context(Database)
    }

    pub async fn count_memos(&self, creator_id: i32) -> Result<Count, Error> {
        let stmt = Statement::with_args(
            "select count(1) as count from memo where creator_id = ?",
            &[creator_id],
        );
        let mut rs: Vec<Count> = self.execute(stmt).await.context(Database)?;
        Ok(rs.pop().unwrap_or(Count { count: 0 }))
    }

    pub async fn delete_memo(&self, memo_id: i32) -> Result<(), Error> {
        let stmt = Statement::with_args("delete from memo where id = ?", &[memo_id]);
        self.state.execute(stmt).await.context(Database)?;
        Ok(())
    }

    pub async fn update_memo(
        &self,
        creator_id: i32,
        UpdateMemo {
            id,
            content,
            visibility,
            row_status,
            pinned,
        }: UpdateMemo,
    ) -> Result<(), Error> {
        {
            // 更新memo
            let mut stmts = Vec::new();
            let mut set = Vec::new();
            let mut args = Vec::new();
            if let Some(content) = content {
                stmts.append(&mut parse_upsert_tag(creator_id, &content));
                set.push("content = ?");
                args.push(Value::from(content));
            }
            if let Some(visibility) = visibility {
                set.push("visibility = ?");
                args.push(Value::from(visibility.as_str_name().to_owned()));
            }
            if let Some(row_status) = row_status {
                set.push("row_status = ?");
                args.push(Value::from(row_status.as_str_name().to_owned()));
            }
            if !set.is_empty() {
                let update_sql = format!("UPDATE memo SET {} WHERE id = ?", set.join(", "));
                args.push(Value::from(id));
                stmts.push(Statement::with_args(update_sql, &args));

                self.state.batch(stmts).await.context(Database)?;
            }
        }
        if let Some(pinned) = pinned {
            // 置顶是单独操作的
            let stmt = Statement::with_args(
                "
                INSERT INTO memo_organizer (
		        	memo_id,
		        	user_id,
		        	pinned
		        )
		        VALUES (?, ?, ?)
		        ON CONFLICT(memo_id, user_id) DO UPDATE 
		        SET
		        	pinned = EXCLUDED.pinned
            ",
                &[id, creator_id, if pinned { 1 } else { 0 }],
            );
            self.state.execute(stmt).await.context(Database)?;
        }

        Ok(())
    }
}

fn parse_upsert_tag(creator_id: i32, content: &str) -> Vec<Statement> {
    let mut stmts = Vec::new();
    let tags = parse_document(content, true);
    for tag in tags {
        if let Some(node::Node::TagNode(TagNode { content })) = tag.node {
            stmts.push(Statement::with_args(
                "
                insert into tag (
                    name, creator_id
                )
                values (?, ?)
                on conflict(name, creator_id) do update 
                set
                    name = excluded.name",
                &[content, creator_id.to_string()],
            ));
        }
    }
    stmts
}

impl From<FindMemo> for Statement {
    fn from(val: FindMemo) -> Self {
        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = val.id {
            wheres.push("memo.id = ?");
            args.push(Value::from(id));
        }
        if let Some(creator_id) = val.creator_id {
            wheres.push("memo.creator_id = ?");
            args.push(Value::from(creator_id));
        }
        if let Some(row_status) = &val.row_status {
            wheres.push("memo.row_status = ?");
            args.push(Value::from(row_status));
        }
        if let Some(created_ts_before) = val.created_ts_before {
            wheres.push("memo.created_ts < ?");
            args.push(Value::from(created_ts_before));
        }
        if let Some(created_ts_after) = val.created_ts_after {
            wheres.push("memo.created_ts > ?");
            args.push(Value::from(created_ts_after));
        }
        for content_search in val.content_search.iter() {
            wheres.push("memo.content LIKE ?");
            args.push(Value::from(format!("%{content_search}%")));
        }
        if val.pinned {
            wheres.push("memo_organizer.pinned = 1");
        }

        let mut wheres: Vec<String> = wheres.into_iter().map(|s| s.to_owned()).collect();

        if !val.visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in val.visibility_list.iter() {
                args.push(Value::from(visibility.as_str_name().to_owned()));
                l.push("?");
            }
            wheres.push(format!("memo.visibility in ({})", l.join(", ")));
        }

        let mut orders = Vec::new();
        if val.order_by_pinned {
            orders.push("pinned DESC");
        }
        if val.order_by_updated_ts {
            orders.push("memo.updated_ts DESC");
        } else {
            orders.push("memo.created_ts DESC");
        }
        orders.push("memo.id DESC");

        let mut fields = vec![
            "memo.id AS id",
            "user.username AS creator",
            "memo.creator_id AS creator_id",
            "memo.created_ts AS create_time",
            "memo.updated_ts AS update_time",
            "memo.row_status AS row_status",
            "memo.visibility AS visibility",
            "CASE WHEN memo_organizer.pinned = 1 THEN 1 ELSE 0 END AS pinned",
        ];

        if !val.exclude_content {
            fields.push("memo.content AS content");
        }

        if val.order_by_updated_ts {
            fields.push("memo.updated_ts AS display_time");
        } else {
            fields.push("memo.created_ts AS display_time");
        }

        let mut query = format!(
            "select
            {}
            from memo
		    left join memo_organizer on memo.id = memo_organizer.memo_id
            left join user on memo.creator_id = user.id
            where {}
            group by memo.id
            order by {}",
            fields.join(",\n"),
            wheres.join(" AND "),
            orders.join(", ")
        );

        if let Some(limit) = val.limit {
            query = format!("{query} LIMIT {limit}");
            if let Some(offset) = val.offset {
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
