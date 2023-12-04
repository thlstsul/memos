use std::sync::Arc;

use libsql_client::Client;
use snafu::{ResultExt, Snafu};

use crate::api::memos_api_v1::user_setting::UserSetting;
use crate::dao::user::Error as DaoErr;
use crate::dao::user_setting::UserSettingDao;
use crate::{api::memos_api_v2::User, dao::user::UserDao};

pub struct UserService {
    user_dao: UserDao,
    setting_dao: UserSettingDao,
}

impl UserService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            user_dao: UserDao {
                client: Arc::clone(client),
            },
            setting_dao: UserSettingDao {
                client: Arc::clone(client),
            },
        }
    }

    pub async fn petch_user(&self, id: i32) -> Result<User, Error> {
        let rs = self.user_dao.petch_user(id).await;
        if let Err(DaoErr::Inexistent) = rs {
            rs.context(UserNotFound { id })
        } else {
            rs.context(QueryUserFailed)
        }
    }

    pub async fn host_user(&self) -> Result<User, Error> {
        let rs = self.user_dao.host_user().await;
        if let Err(DaoErr::Inexistent) = rs {
            rs.context(UserNotFound { id: -1 })
        } else {
            rs.context(QueryUserFailed)
        }
    }

    pub async fn find_setting(&self, user_id: i32) -> Result<Vec<UserSetting>, Error> {
        self.setting_dao
            .find_setting(user_id)
            .await
            .context(QuerySettingFailed)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("User not found: {id}"), context(suffix(false)))]
    UserNotFound {
        id: i32,
        source: crate::dao::user::Error,
    },
    #[snafu(display("Failed to find user"), context(suffix(false)))]
    QueryUserFailed { source: crate::dao::user::Error },
    #[snafu(display("Failed to find userSettingList"), context(suffix(false)))]
    QuerySettingFailed {
        source: crate::dao::user_setting::Error,
    },
}
