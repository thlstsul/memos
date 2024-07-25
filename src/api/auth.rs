use super::v1::gen::{GetAuthStatusResponse, User};

impl From<User> for GetAuthStatusResponse {
    fn from(value: User) -> Self {
        Self { user: Some(value) }
    }
}
