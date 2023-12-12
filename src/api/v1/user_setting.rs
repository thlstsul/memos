use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct UserSetting {
    #[serde(rename(serialize = "userId"))]
    user_id: i32,
    key: String,
    value: String,
}
