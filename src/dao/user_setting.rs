use std::sync::Arc;

use libsql_client::{Client, Statement};
use snafu::{ResultExt, Snafu};

use crate::api::user::UserSetting;

use super::Dao;

#[derive(Debug, Clone)]
pub struct UserSettingDao {
    pub client: Arc<Client>,
}

impl Dao for UserSettingDao {
    fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

impl UserSettingDao {
    pub async fn find_setting(&self, user_id: i32) -> Result<Vec<UserSetting>, Error> {
        let stmt = Statement::with_args("select * from user_setting where user_id = ?", &[user_id]);
        self.execute(stmt).await.context(Database)
    }

    pub async fn upsert_setting(&self, setting: UserSetting) -> Result<(), Error> {
        let stmt = Statement::with_args(
            "
        INSERT INTO user_setting (
			user_id, key, value
		)
		VALUES (?, ?, ?)
		ON CONFLICT(user_id, key) DO UPDATE 
		SET value = EXCLUDED.value
        ",
            &[
                setting.user_id.to_string(),
                setting.key.to_string(),
                setting.value,
            ],
        );
        self.client.execute(stmt).await.context(Database)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
