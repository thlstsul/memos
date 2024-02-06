use crate::{
    api::v2::{user, User},
    state::AppState,
};

use super::{Dao, Error};

#[derive(Clone)]
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
        name: &str,
        password_hash: Option<&str>,
    ) -> Result<Option<User>, Error> {
        let mut users = if let Some(password_hash) = password_hash {
            let sql = "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where username = ? and password_hash = ?";
            self.query(sql, [name, password_hash]).await?
        } else {
            let sql = "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where username = ?";
            self.query(sql, [name]).await?
        };

        Ok(users.pop())
    }

    pub async fn petch_user(&self, id: i32) -> Result<Option<User>, Error> {
        let sql = "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where id = ?";
        let mut users = self.query(sql, [id]).await?;
        Ok(users.pop())
    }

    pub async fn host_user(&self) -> Result<Option<User>, Error> {
        let sql = "select id, created_ts as create_time, updated_ts as update_time, row_status, username, role, email, nickname, password_hash as password, avatar_url from user where role = ?";
        let mut users = self.query(sql, [user::Role::Host.as_str_name()]).await?;
        Ok(users.pop())
    }
}
