use crate::api::v2::User;

use super::user_setting::UserSetting;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    #[serde(flatten)]
    pub user: User,
    #[serde(rename = "userSettingList")]
    pub settings: Vec<UserSetting>,
}
