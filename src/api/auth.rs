use super::{
    v2::{GetAuthStatusResponse, User},
    USER_NAME_PREFIX,
};

impl From<User> for GetAuthStatusResponse {
    fn from(value: User) -> Self {
        let mut user = value;
        user.name = format!("{}/{}", USER_NAME_PREFIX, user.name);
        Self { user: Some(user) }
    }
}
