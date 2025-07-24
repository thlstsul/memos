use super::v1::gen::{GetCurrentSessionRequest, User};

impl From<User> for GetCurrentSessionRequest {
    fn from(value: User) -> Self {
        Self { user: Some(value) }
    }
}
