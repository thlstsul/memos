use libsql_client::{Statement, Value};
use snafu::{ResultExt, Snafu};

use crate::{api::user::UserSetting, state::AppState};

use super::Dao;

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
        self.execute(stmt).await.context(Database)
    }

    pub async fn upsert_setting(&self, settings: Vec<UserSetting>) -> Result<(), Error> {
        let stmts: Vec<Statement> = settings
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

        self.state.batch(stmts).await.context(Database)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
