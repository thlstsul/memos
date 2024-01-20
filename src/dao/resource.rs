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
    pub async fn create_resource(&self, create: WholeResource) -> Result<Option<Resource>, Error> {
        let mut fields = vec!["filename", "type", "size", "creator_id"];
        let mut placeholder = vec!["?", "?", "?", "?"];
        let mut args = vec![
            Value::from(create.filename),
            Value::from(create.r#type),
            Value::from(create.size),
            Value::from(create.creator_id),
        ];

        if !create.blob.is_empty() {
            fields.push("blob");
            placeholder.push("?");
            args.push(Value::from(create.blob));
        }

        if !create.external_link.is_empty() {
            fields.push("external_link");
            placeholder.push("?");
            args.push(Value::from(create.external_link));
        }

        if !create.internal_path.is_empty() {
            fields.push("internal_path");
            placeholder.push("?");
            args.push(Value::from(create.internal_path));
        }

        if create.id > 0 {
            fields.push("id");
            placeholder.push("?");
            args.push(Value::from(create.id));
        }

        if create.created_ts > 0 {
            fields.push("created_ts");
            placeholder.push("?");
            args.push(Value::from(create.created_ts));
        }

        if create.updated_ts > 0 {
            fields.push("updated_ts");
            placeholder.push("?");
            args.push(Value::from(create.updated_ts));
        }

        if let Some(memo_id) = create.memo_id {
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

    pub async fn list_resource(&self, find: FindResource) -> Result<Vec<Resource>, Error> {
        todo!()
    }
}
