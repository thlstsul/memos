use std::sync::Arc;

use libsql_client::Client;
use snafu::{ResultExt, Snafu};

use crate::{api::v1::system::SystemSetting, dao::system_setting::SystemSettingDao};

pub struct SystemService {
    dao: SystemSettingDao,
}

impl SystemService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            dao: SystemSettingDao {
                client: Arc::clone(client),
            },
        }
    }

    pub async fn list_setting(&self) -> Result<Vec<SystemSetting>, Error> {
        self.dao.list_setting().await.context(ListSettingFailed)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to find system setting list"), context(suffix(false)))]
    ListSettingFailed {
        source: crate::dao::system_setting::Error,
    },
}
