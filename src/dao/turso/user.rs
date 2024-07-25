use async_trait::async_trait;
use libsql::params;

use crate::{
    api::v1::gen::user::Role,
    dao::user::{
        FindUserError, FindUserSettingError, GetHostUserError, PetchUserError,
        UpsertUserSettingError, UserRepository,
    },
    model::user::{User, UserSetting},
};

use super::Turso;

#[async_trait]
impl UserRepository for Turso {
    async fn find_user(
        &self,
        name: &str,
        password_hash: Option<&str>,
    ) -> Result<Option<User>, FindUserError> {
        let sql = "select id, created_ts, updated_ts, row_status, username, role, email, nickname, password_hash, avatar_url, description from user where username = ?";
        let mut users = if let Some(password_hash) = password_hash {
            let sql = format!("{sql} and password_hash = ?");
            self.query(sql, [name, password_hash]).await?
        } else {
            self.query(sql, [name]).await?
        };

        Ok(users.pop())
    }

    async fn petch_user(&self, id: i32) -> Result<Option<User>, PetchUserError> {
        let sql = "select id, created_ts, updated_ts, row_status, username, role, email, nickname, password_hash, avatar_url, description from user where id = ?";
        let mut users = self.query(sql, [id]).await?;
        Ok(users.pop())
    }

    async fn host_user(&self) -> Result<Option<User>, GetHostUserError> {
        let sql = "select id, created_ts, updated_ts, row_status, username, role, email, nickname, password_hash, avatar_url, description from user where role = ?";
        let mut users = self.query(sql, [Role::Host.as_str_name()]).await?;
        Ok(users.pop())
    }

    async fn find_user_setting(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserSetting>, FindUserSettingError> {
        let sql = "select * from user_setting where user_id = ?";
        Ok(self.query(sql, [user_id]).await?)
    }

    async fn upsert_user_setting(
        &self,
        settings: Vec<UserSetting>,
    ) -> Result<(), UpsertUserSettingError> {
        let transaction = self.transaction().await?;
        let mut stmt = Self::tx_prepare( &transaction, "insert into user_setting (user_id, key, value) values (?, ?, ?) on conflict(user_id, key) do update set value = excluded.value").await?;
        for setting in settings {
            Self::statement_execute(
                &mut stmt,
                params![setting.user_id, setting.key.as_str_name(), setting.value,],
            )
            .await?;
            stmt.reset();
        }
        Self::commit(transaction).await?;

        Ok(())
    }
}
