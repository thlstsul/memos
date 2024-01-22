use libsql_client::Statement;

use crate::{
    api::system::{SystemSetting, SystemSettingKey},
    state::AppState,
};

use super::{Dao, Error};

#[derive(Debug)]
pub struct SystemSettingDao {
    pub state: AppState,
}

impl Dao for SystemSettingDao {
    fn get_state(&self) -> &AppState {
        &self.state
    }
}

impl SystemSettingDao {
    pub async fn list_setting(&self) -> Result<Vec<SystemSetting>, Error> {
        self.query("select * from system_setting").await
    }

    pub async fn find_setting(
        &self,
        key: SystemSettingKey,
    ) -> Result<Option<SystemSetting>, Error> {
        let stmt = Statement::with_args(
            "select * from system_setting where name = ?",
            &[key.to_string()],
        );
        let mut settings = self.query(stmt).await?;
        Ok(settings.pop())
    }
}
