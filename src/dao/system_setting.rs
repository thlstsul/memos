use crate::{
    api::system::{SystemSetting, SystemSettingKey},
    state::AppState,
};

use super::{Dao, Error};

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
        self.query("select * from system_setting", ()).await
    }

    pub async fn find_setting(
        &self,
        key: SystemSettingKey,
    ) -> Result<Option<SystemSetting>, Error> {
        let sql = "select * from system_setting where name = ?";
        let mut settings = self.query(sql, [key.to_string()]).await?;
        Ok(settings.pop())
    }
}
