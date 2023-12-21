use std::sync::Arc;

use libsql_client::{Client, Statement};
use snafu::{ResultExt, Snafu};

use crate::api::v1::user_setting::UserSetting;

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
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
