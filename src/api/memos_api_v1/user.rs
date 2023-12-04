use crate::api::memos_api_v2::User;

use super::user_setting::UserSetting;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserResponse {
    #[serde(flatten)]
    pub user: User,
    pub settings: Vec<UserSetting>,
}
