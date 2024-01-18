use libsql_client::Statement;
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{
    api::v2::{user, User},
    state::AppState,
};

use super::Dao;

#[derive(Debug, Clone)]
pub struct UserDao {
    pub state: AppState,
}

impl Dao for UserDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl UserDao {
    pub async fn find_user(
        &self,
        name: String,
        password_hash: Option<String>,
    ) -> Result<User, Error> {
        let stmt = if let Some(password_hash) = password_hash {
            Statement::with_args(
                "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where username = ? and password_hash = ?",
                &[name, password_hash],
            )
        } else {
            Statement::with_args("select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where username = ?", &[name])
        };
        let mut users: Vec<User> = self.execute(stmt).await.context(Database)?;
        users.pop().context(Inexistent)
    }

    pub async fn petch_user(&self, id: i32) -> Result<User, Error> {
        let stmt = Statement::with_args("select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where id = ?", &[id]);
        let mut users: Vec<User> = self.execute(stmt).await.context(Database)?;
        users.pop().context(Inexistent)
    }

    pub async fn host_user(&self) -> Result<User, Error> {
        let stmt = Statement::with_args(
            "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where role = ?",
            &[user::Role::Host.as_str_name()],
        );
        let mut users: Vec<User> = self.execute(stmt).await.context(Database)?;
        users.pop().context(Inexistent)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
    #[snafu(display("Data does not exsit"), context(suffix(false)))]
    Inexistent,
}
