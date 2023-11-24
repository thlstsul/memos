use std::sync::Arc;

use libsql_client::{de, Client, Statement};
use snafu::{ResultExt, Snafu};

use crate::pb::memos_api_v2::User;

pub struct UserDao {
    pub client: Arc<Client>,
}

impl UserDao {
    pub async fn get_user(&self, name: String, password_hash: String) -> Result<User, Error> {
        let users = self
            .client
            .execute(Statement::with_args(
                "select * from user where name = ? and password_hash = ?",
                &[name, password_hash],
            ))
            .await
            .context(Database)?
            .rows
            .iter()
            .map(de::from_row)
            .collect::<Result<Vec<User>, _>>()
            .context(Deserialize)?;
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
    #[snafu(display("Deserialize fail"), context(suffix(false)))]
    Deserialize { source: anyhow::Error },
    #[snafu(display("Data does not exsit"), context(suffix(false)))]
    Inexistent,
}
