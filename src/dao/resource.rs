use std::sync::Arc;

use libsql_client::{Client, Statement, Value};
use snafu::{ResultExt, Snafu};

use crate::api::{
    resource::{CreateResource, FindResource},
    v2::Resource,
};

use super::Dao;

pub struct ResourceDao {
    pub client: Arc<Client>,
}

impl Dao for ResourceDao {
    fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

impl ResourceDao {
    pub async fn create_resource(&self, create: CreateResource) -> Result<Resource, Error> {
        let mut fields = vec![
            "filename",
            "blob",
            "external_link",
            "type",
            "size",
            "creator_id",
            "internal_path",
        ];
        let mut placeholder = vec!["?", "?", "?", "?", "?", "?", "?"];
        let mut args = vec![
            Value::from(create.filename),
            Value::from(create.blob),
            Value::from(create.external_link),
            Value::from(create.r#type),
            Value::from(create.size),
            Value::from(create.creator_id),
            Value::from(create.internal_path),
        ];

        if let Some(id) = create.id {
            fields.push("id");
            placeholder.push("?");
            args.push(Value::from(id));
        }

        if let Some(created_ts) = create.created_ts {
            fields.push("created_ts");
            placeholder.push("?");
            args.push(Value::from(created_ts));
        }

        if let Some(updated_ts) = create.updated_ts {
            fields.push("updated_ts");
            placeholder.push("?");
            args.push(Value::from(updated_ts));
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

        let mut rs = self.execute(stmt).await.context(Database)?;
        if let Some(res) = rs.pop() {
            Ok(res)
        } else {
            Err(Error::CreateResourceFailed)
        }
    }

    pub fn list_resource(&self, find: FindResource) -> Result<Vec<Resource>, Error> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
    #[snafu(display("Create resource failed"), context(suffix(false)))]
    CreateResourceFailed,
}
