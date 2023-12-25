use std::sync::Arc;

use libsql_client::{Client, Statement};
use snafu::{ResultExt, Snafu};

use crate::api::system::{SystemSetting, SystemSettingKey};

use super::Dao;

#[derive(Debug)]
pub struct SystemSettingDao {
    pub client: Arc<Client>,
}

impl Dao for SystemSettingDao {
    fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

impl SystemSettingDao {
    pub async fn list_setting(&self) -> Result<Vec<SystemSetting>, Error> {
        self.execute("select * from system_setting")
            .await
            .context(Database)
    }

    pub async fn find_setting(
        &self,
        key: SystemSettingKey,
    ) -> Result<Option<SystemSetting>, Error> {
        let stmt = Statement::with_args(
            "select * from system_setting where name = ?",
            &[key.to_string()],
        );
        let settings: Vec<SystemSetting> = self.execute(stmt).await.context(Database)?;
        Ok(settings.first().cloned())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
    #[snafu(display("Data does not exsit"), context(suffix(false)))]
    Inexistent,
}
