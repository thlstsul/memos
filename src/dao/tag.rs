use std::sync::Arc;

use libsql_client::{Client, Statement};
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
        let stmt = Statement::with_args("select a.username creator, b.name from user a, tag b where a.username = ? and a.id = b.creator_id", &[creator]);
        self.execute(stmt).await.context(Database)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
