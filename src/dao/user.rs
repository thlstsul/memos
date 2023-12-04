use std::sync::Arc;

use libsql_client::{Client, Statement};
use snafu::{ResultExt, Snafu};

use crate::api::memos_api_v2::User;

use super::Dao;

#[derive(Debug)]
pub struct UserDao {
    pub client: Arc<Client>,
}

impl Dao for UserDao {
    fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

impl UserDao {
    pub async fn find_user(&self, name: String, password_hash: String) -> Result<User, Error> {
        let stmt = Statement::with_args(
            "select * from user where username = ? and password_hash = ?",
            &[name, password_hash],
        );
        let users: Vec<User> = self.execute(stmt).await.context(Database)?;
        if let Some(user) = users.first() {
            Ok(user.clone())
        } else {
            Err(Error::Inexistent)
        }
    }

    pub async fn petch_user(&self, id: i32) -> Result<User, Error> {
        let stmt = Statement::with_args("select * from user where id = ?", &[id]);
        let users: Vec<User> = self.execute(stmt).await.context(Database)?;
        if let Some(user) = users.first() {
            Ok(user.clone())
        } else {
            Err(Error::Inexistent)
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute fail"), context(suffix(false)))]
    Database { source: anyhow::Error },
    #[snafu(display("Data does not exsit"), context(suffix(false)))]
    Inexistent,
}
