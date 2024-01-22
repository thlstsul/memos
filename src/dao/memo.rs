use tracing::{error, info};

use libsql_client::{de, Statement, Value};

use crate::{
    api::{
        memo::{CreateMemo, FindMemo, UpdateMemo},
        v2::Memo,
        Count,
    },
    dao::tag::parse_upsert_tag,
    state::AppState,
};

use super::{Dao, Error};

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
    ) -> Result<Option<Memo>, Error> {
        let mut stmts = vec![Statement::with_args(
            "INSERT INTO memo (creator_id, content, visibility) VALUES (?, ?, ?) RETURNING id, creator_id, created_ts as create_time, created_ts as display_time, updated_ts as update_time, row_status, content, visibility",
            &[
                Value::from(creator_id),
                Value::from(content.clone()),
                Value::from(visibility.as_str_name().to_owned()),
            ],
        )];

        stmts.append(&mut parse_upsert_tag(creator_id, &content));

        let rss = self.batch(stmts).await?;
        if let Some(rs) = rss.first() {
            if let Ok(mut memos) = rs
                .rows
                .iter()
                .map(de::from_row)
                .collect::<Result<Vec<Memo>, _>>()
            {
                return Ok(memos.pop());
            } else {
                error!("Deserialize memo failed: {rs:?}");
            }
        }
        Ok(None)
    }

    pub async fn list_memos(&self, find: FindMemo) -> Result<Vec<Memo>, Error> {
        let stmt: Statement = find.into();
        info!("{stmt}");
        self.query(stmt).await
    }

    pub async fn count_memos(&self, creator_id: i32) -> Result<Count, Error> {
        let stmt = Statement::with_args(
            "select count(1) as count from memo where creator_id = ?",
            &[creator_id],
        );
        let mut rs: Vec<Count> = self.query(stmt).await?;
        Ok(rs.pop().unwrap_or(Count { count: 0 }))
    }

    pub async fn delete_memo(&self, memo_id: i32) -> Result<(), Error> {
        let stmt = Statement::with_args("delete from memo where id = ?", &[memo_id]);
        self.execute(stmt).await?;
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

                self.batch(stmts).await?;
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
            self.execute(stmt).await?;
        }

        Ok(())
    }
}

impl From<FindMemo> for Statement {
    fn from(value: FindMemo) -> Self {
        let FindMemo {
            id,
            creator,
            creator_id,
            row_status,
            created_ts_after,
            created_ts_before,
            pinned,
            content_search,
            visibility_list,
            exclude_content,
            limit,
            offset,
            order_by_updated_ts,
            order_by_pinned,
        } = value;

        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = id {
            wheres.push("memo.id = ?");
            args.push(Value::from(id));
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
        for content_search in content_search.iter() {
            wheres.push("memo.content LIKE ?");
            args.push(Value::from(format!("%{content_search}%")));
        }
        if pinned {
            wheres.push("memo_organizer.pinned = 1");
        }

        let mut wheres: Vec<_> = wheres.into_iter().map(|s| s.to_owned()).collect();

        if !visibility_list.is_empty() {
            let mut l = Vec::new();
            for visibility in visibility_list.iter() {
                args.push(Value::from(visibility.as_str_name().to_owned()));
                l.push("?");
            }
            wheres.push(format!("memo.visibility in ({})", l.join(", ")));
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

        if !exclude_content {
            fields.push("memo.content AS content");
        }

        if order_by_updated_ts {
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

        if let Some(limit) = limit {
            query = format!("{query} LIMIT {limit}");
            if let Some(offset) = offset {
                query = format!("{query} OFFSET {offset}");
            }
        }

        Statement::with_args(query, &args)
    }
}
