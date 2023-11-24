use std::sync::Arc;

use actix_web::{http::StatusCode, ResponseError};
use libsql_client::Client;
use sm3::{Digest, Sm3};
use snafu::{ResultExt, Snafu};

use crate::{dao::user::UserDao, pb::memos_api_v2::User};

pub struct AuthService {
    dao: UserDao,
}

impl AuthService {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            dao: UserDao { client },
        }
    }

    pub async fn sign_in(&self, name: String, password: String) -> Result<User, Error> {
        let mut hasher = Sm3::new();
        hasher.update(password);

        let password_hash = hex::encode(hasher.finalize());
        self.dao
            .get_user(name, password_hash)
            .await
            .context(GetUser)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Data does not exsit"), context(suffix(false)))]
    GetUser { source: crate::dao::user::Error },
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}
