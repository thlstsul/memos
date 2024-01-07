use std::sync::Arc;

use libsql_client::{Client, Statement, Value};
use snafu::{ResultExt, Snafu};

use crate::api::v2::Tag;

use super::Dao;

#[derive(Debug)]
pub struct TagDao {
    pub client: Arc<Client>,
}

impl Dao for TagDao {
    fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

impl TagDao {
    pub async fn list_tags(&self, creator: String) -> Result<Vec<Tag>, Error> {
        let stmt = Statement::with_args("select user.username as creator, tag.name from user, tag where user.username = ? and user.id = tag.creator_id", &[creator]);
        self.execute(stmt).await.context(Database)
    }

    pub async fn delete_tag(&self, tag: Tag) -> Result<(), Error> {
        let stmt = Statement::with_args(
            "delete from tag where creator_id = ( select id from user where username = ? limit 1 ) and name = ?",
            &[tag.creator, tag.name],
        );
        self.client.execute(stmt).await.context(Database)?;
        Ok(())
    }

    pub async fn upsert_tag(&self, name: String, creator_id: i32) -> Result<(), Error> {
        let stmt = Statement::with_args(
            "
            INSERT INTO tag (
                name, creator_id
            )
            VALUES (?, ?)
            ON CONFLICT(name, creator_id) DO UPDATE 
            SET
                name = EXCLUDED.name",
            &[Value::from(name), Value::from(creator_id)],
        );
        self.client.execute(stmt).await.context(Database)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
