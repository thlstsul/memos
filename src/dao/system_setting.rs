use std::sync::Arc;

use libsql_client::Client;
use snafu::{ResultExt, Snafu};

use crate::api::v1::system::SystemSetting;

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
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute fail"), context(suffix(false)))]
    Database { source: anyhow::Error },
}
