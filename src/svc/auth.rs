use std::sync::Arc;

use libsql_client::Client;
use sm3::{Digest, Sm3};
use snafu::{ResultExt, Snafu};

use crate::{api::v2::User, dao::user::UserDao};

pub struct AuthService {
    dao: UserDao,
}

impl AuthService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            dao: UserDao {
                client: Arc::clone(client),
            },
        }
    }

    pub async fn sign_in(&self, name: String, password: String) -> Result<User, Error> {
        let mut hasher = Sm3::new();
        hasher.update(password);

        let password_hash = hex::encode(hasher.finalize());
        self.dao
            .find_user(name, Some(password_hash))
            .await
            .context(Login)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(
        display("Incorrect login credentials, please try again"),
        context(suffix(false))
    )]
    Login { source: crate::dao::user::Error },
}
