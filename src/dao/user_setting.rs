use libsql::params;

use crate::{api::user::UserSetting, state::AppState};

use super::{Dao, Error};

#[derive(Clone)]
pub struct UserSettingDao {
    pub state: AppState,
}

impl Dao for UserSettingDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl UserSettingDao {
    pub async fn find_setting(&self, user_id: i32) -> Result<Vec<UserSetting>, Error> {
        let sql = "select * from user_setting where user_id = ?";
        self.query(sql, [user_id]).await
    }

    pub async fn upsert_setting(&self, settings: Vec<UserSetting>) -> Result<(), libsql::Error> {
        let transaction = self.get_state().transaction().await?;
        let mut stmt = transaction
                .prepare("insert into user_setting (user_id, key, value) values (?, ?, ?) on conflict(user_id, key) do update set value = excluded.value")
                .await?;
        for setting in settings {
            stmt.execute(params![
                setting.user_id,
                setting.key.to_string(),
                setting.value,
            ])
            .await?;
            stmt.reset();
        }
        transaction.commit().await?;

        Ok(())
    }
}
