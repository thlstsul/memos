use libsql::{params, Value};
use snafu::{ResultExt, Snafu};

use crate::{
    api::{
        memo::{CreateMemo, FindMemo, UpdateMemo},
        pager::Paginator,
        v2::Memo,
        Count,
    },
    state::AppState,
    util::parse_tag,
};

use super::{de, Dao};

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
            resource_name,
            content,
            visibility,
        }: CreateMemo,
    ) -> Result<Option<Memo>, Error> {
        let tags = parse_tag(&content);
        if !tags.is_empty() {
            let mut stmt = self
                .get_state()
                .prepare("insert into tag (name, creator_id) values (?, ?) on conflict(name, creator_id) do update set name = excluded.name")
                .await
                .context(PrepareStatement)?;
            for tag in tags {
                stmt.execute(params![tag, creator_id])
                    .await
                    .context(Execute)?;
            }
        }

        let rows = self.get_state().query(
            "INSERT INTO memo (creator_id, resource_name, content, visibility) VALUES (?, ?, ?, ?) RETURNING id, resource_name as name, creator_id, created_ts as create_time, created_ts as display_time, updated_ts as update_time, row_status, content, visibility", 
            params![creator_id, resource_name, content, visibility.as_str_name()]
        ).await.context(Execute)?;

        let mut memos = de(rows).context(Deserialize)?;
        Ok(memos.pop())
    }

    pub async fn list_memos(
        &self,
        FindMemo {
            id,
            name,
            creator,
            creator_id,
            row_status,
            display_time_after,
            display_time_before,
            pinned,
            content_search,
            visibility_list,
            exclude_content,
            page_token,
            order_by_updated_ts,
            order_by_pinned,
        }: FindMemo,
    ) -> Result<Vec<Memo>, super::Error> {
        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = id {
            wheres.push("memo.id = ?");
            args.push(Value::from(id));
        }
        if let Some(name) = name {
            wheres.push("memo.resource_name = ?");
            args.push(Value::from(name))
        }
        if let Some(creator_id) = creator_id {
            wheres.push("memo.creator_id = ?");
            args.push(Value::from(creator_id));
        }
        if let Some(row_status) = &row_status {
            wheres.push("memo.row_status = ?");
            args.push(Value::from(row_status.to_string()));
        }
        if let Some(display_time_before) = display_time_before {
            wheres.push("memo.created_ts < ?");
            args.push(Value::from(display_time_before));
        }
        if let Some(display_time_after) = display_time_after {
            wheres.push("memo.created_ts > ?");
            args.push(Value::from(display_time_after));
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
            "memo.resource_name as name",
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

        let mut sql = format!(
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

        if let Some(page_token) = page_token {
            sql = format!("{sql} {}", page_token.as_limit_sql());
        }
        self.query(&sql, args).await
    }

    pub async fn count_memos(&self, creator_id: i32) -> Result<Vec<Count>, super::Error> {
        let sql = "select created_date, count(1) as count from (
            select date(created_ts, 'unixepoch') as created_date from memo where creator_id = ?
        ) group by created_date";
        self.query(sql, [creator_id]).await
    }

    pub async fn delete_memo(&self, memo_id: i32) -> Result<(), super::Error> {
        let sql = "delete from memo where id = ?";
        self.execute(sql, [memo_id]).await?;
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
                let tags = parse_tag(&content);
                if !tags.is_empty() {
                    let mut stmt = self
                        .get_state()
                        .prepare("insert into tag (name, creator_id) values (?, ?) on conflict(name, creator_id) do update set name = excluded.name")
                        .await
                        .context(PrepareStatement)?;
                    for tag in tags {
                        stmt.execute(params![tag, creator_id])
                            .await
                            .context(Execute)?;
                    }
                }

                set.push("content = ?");
                args.push(Value::from(content));
            }
            if !set.is_empty() {
                let update_sql = format!("UPDATE memo SET {} WHERE id = ?", set.join(", "));
                args.push(Value::from(id));
                self.get_state()
                    .execute(&update_sql, args)
                    .await
                    .context(Execute)?;
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
            self.get_state()
                .execute(sql, [id, creator_id, if pinned { 1 } else { 0 }])
                .await
                .context(Execute)?;
        }

        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Execute { source: libsql::Error },
    #[snafu(
        display("Failed to prepare statement: {source}"),
        context(suffix(false))
    )]
    PrepareStatement { source: libsql::Error },
    #[snafu(display("Deserialize failed: {source}"), context(suffix(false)))]
    Deserialize { source: super::Error },
}
