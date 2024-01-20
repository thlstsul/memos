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
