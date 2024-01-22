use std::collections::HashMap;

use libsql_client::{Statement, Value};

use crate::{
    api::{
        resource::{FindResource, WholeResource},
        v2::Resource,
    },
    state::AppState,
};

use super::{Dao, Error};

pub struct ResourceDao {
    pub state: AppState,
}

impl Dao for ResourceDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl ResourceDao {
    pub async fn create_resource(
        &self,
        WholeResource {
            filename,
            r#type,
            size,
            creator_id,
            blob,
            external_link,
            internal_path,
            id,
            created_ts,
            updated_ts,
            memo_id,
        }: WholeResource,
    ) -> Result<Option<Resource>, Error> {
        let mut fields = vec!["filename", "type", "size", "creator_id"];
        let mut placeholder = vec!["?", "?", "?", "?"];
        let mut args = vec![
            Value::from(filename),
            Value::from(r#type),
            Value::from(size),
            Value::from(creator_id),
        ];

        if !blob.is_empty() {
            fields.push("blob");
            placeholder.push("?");
            args.push(Value::from(blob));
        }

        if !external_link.is_empty() {
            fields.push("external_link");
            placeholder.push("?");
            args.push(Value::from(external_link));
        }

        if !internal_path.is_empty() {
            fields.push("internal_path");
            placeholder.push("?");
            args.push(Value::from(internal_path));
        }

        if id > 0 {
            fields.push("id");
            placeholder.push("?");
            args.push(Value::from(id));
        }

        if created_ts > 0 {
            fields.push("created_ts");
            placeholder.push("?");
            args.push(Value::from(created_ts));
        }

        if updated_ts > 0 {
            fields.push("updated_ts");
            placeholder.push("?");
            args.push(Value::from(updated_ts));
        }

        if let Some(memo_id) = memo_id {
            fields.push("memo_id");
            placeholder.push("?");
            args.push(Value::from(memo_id));
        }

        let insert_sql = format!(
            "insert into resource ({}) values ({}) returning id, filename, type, size, created_ts as create_time, external_link",
            fields.join(", "),
            placeholder.join(", ")
        );

        let stmt = Statement::with_args(insert_sql, &args);

        let mut rs = self.query(stmt).await?;
        Ok(rs.pop())
    }

    pub async fn get_resource(&self, id: i32) -> Result<Option<WholeResource>, Error> {
        let stmt = Statement::with_args("select * from resource where id = ?", &[id]);
        let mut rs: Vec<WholeResource> = self.query(stmt).await?;
        Ok(rs.pop())
    }

    pub async fn list_resources(&self, find: FindResource) -> Result<Vec<Resource>, Error> {
        let stmt: Statement = find.into();
        self.query(stmt).await
    }

    pub async fn delete_resource(&self, id: i32, creator_id: i32) -> Result<(), Error> {
        let stmt = Statement::with_args(
            "delete from resource where id = ? and creator_id = ?",
            &[id, creator_id],
        );
        self.execute(stmt).await?;
        Ok(())
    }

    pub async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<Resource>>, Error> {
        if memo_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let stmts: Vec<Statement> = memo_ids.iter().map(|i: &i32| Statement::with_args("select id, filename, external_link, type, size, created_ts as create_time, memo_id from resource where memo_id = ?", &[*i])).collect();
        let mut rss = self.batch_query::<_, Resource>(stmts).await?;
        let mut rtn = HashMap::new();
        for (i, memo_id) in memo_ids.iter().enumerate() {
            rtn.insert(*memo_id, rss.pop_front().unwrap_or_default());
        }
        Ok(rtn)
    }
}

impl From<FindResource> for Statement {
    fn from(value: FindResource) -> Self {
        let FindResource {
            id,
            creator_id,
            filename,
            memo_id,
            limit,
            offset,
            has_relate_memo,
        } = value;

        let mut wheres = vec!["1 = 1"];
        let mut args = Vec::new();

        if let Some(id) = id {
            wheres.push("id = ?");
            args.push(Value::from(id));
        }

        if let Some(creator_id) = creator_id {
            wheres.push("creator_id = ?");
            args.push(Value::from(creator_id));
        }

        if let Some(filename) = filename {
            wheres.push("filename = ?");
            args.push(Value::from(filename));
        }

        if let Some(memo_id) = memo_id {
            wheres.push("memo_id = ?");
            args.push(Value::from(memo_id));
        }

        if has_relate_memo {
            wheres.push("memo_id IS NOT NULL");
        }

        let mut sql = format!("select id, filename, external_link, type, size, created_ts as create_time, memo_id from resource where {}", wheres.join(" AND "));

        if let Some(limit) = limit {
            sql = format!("{sql} LIMIT {limit}");
            if let Some(offset) = offset {
                sql = format!("{sql} OFFSET {offset}");
            }
        }

        Statement::with_args(sql, &args)
    }
}
