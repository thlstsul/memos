use snafu::{ResultExt, Snafu};

use crate::api::system::{SystemSetting, SystemSettingKey};
use crate::dao::system_setting::SystemSettingDao;
use crate::state::AppState;

pub struct SystemService {
    dao: SystemSettingDao,
}

impl SystemService {
    pub fn new(state: &AppState) -> Self {
        Self {
            dao: SystemSettingDao {
                state: state.clone(),
            },
        }
    }

    pub async fn list_setting(&self) -> Result<Vec<SystemSetting>, Error> {
        self.dao.list_setting().await.context(ListSettingFailed)
    }

    #[allow(dead_code)]
    pub async fn find_setting(
        &self,
        key: SystemSettingKey,
    ) -> Result<Option<SystemSetting>, Error> {
        self.dao
            .find_setting(key.clone())
            .await
            .context(FindSettingFailed {
                key: key.to_string(),
            })
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(
        display("Failed to get system setting list: {source}"),
        context(suffix(false))
    )]
    ListSettingFailed {
        source: crate::dao::system_setting::Error,
    },
    #[snafu(
        display("Failed to find system setting with: {key}, {source}"),
        context(suffix(false))
    )]
    FindSettingFailed {
        key: String,
        source: crate::dao::system_setting::Error,
    },
}
