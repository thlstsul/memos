use libsql_client::{Statement, Value};

use crate::{api::user::UserSetting, state::AppState};

use super::{Dao, Error};

#[derive(Debug, Clone)]
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
        let stmt = Statement::with_args("select * from user_setting where user_id = ?", &[user_id]);
        self.query(stmt).await
    }

    pub async fn upsert_setting(&self, settings: Vec<UserSetting>) -> Result<(), Error> {
        let stmts: Vec<_> = settings
            .into_iter()
            .map(|setting| {
                Statement::with_args(
                    "insert into user_setting (user_id, key, value) values (?, ?, ?) 
                    on conflict(user_id, key) do update set value = excluded.value",
                    &[
                        Value::from(setting.user_id),
                        Value::from(setting.key.to_string()),
                        Value::from(setting.value),
                    ],
                )
            })
            .collect();

        self.batch(stmts).await?;
        Ok(())
    }
}
