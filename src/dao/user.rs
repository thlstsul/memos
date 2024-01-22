use libsql_client::Statement;

use crate::{
    api::v2::{user, User},
    state::AppState,
};

use super::{Dao, Error};

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
    ) -> Result<Option<User>, Error> {
        let stmt = if let Some(password_hash) = password_hash {
            Statement::with_args(
                "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where username = ? and password_hash = ?",
                &[name, password_hash],
            )
        } else {
            Statement::with_args("select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where username = ?", &[name])
        };
        let mut users = self.query(stmt).await?;
        Ok(users.pop())
    }

    pub async fn petch_user(&self, id: i32) -> Result<Option<User>, Error> {
        let stmt = Statement::with_args("select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where id = ?", &[id]);
        let mut users = self.query(stmt).await?;
        Ok(users.pop())
    }

    pub async fn host_user(&self) -> Result<Option<User>, Error> {
        let stmt = Statement::with_args(
            "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where role = ?",
            &[user::Role::Host.as_str_name()],
        );
        let mut users = self.query(stmt).await?;
        Ok(users.pop())
    }
}
