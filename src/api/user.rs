use super::{
    get_name_parent_token,
    v2::{GetUserRequest, GetUserResponse, User},
    Error, USER_NAME_PREFIX,
};

impl GetUserRequest {
    pub fn get_name(&self) -> Result<String, Error> {
        get_name_parent_token(self.name.clone(), USER_NAME_PREFIX)
    }
}

impl From<User> for GetUserResponse {
    fn from(value: User) -> Self {
        let mut user = value;
        user.name = format!("{}/{}", USER_NAME_PREFIX, user.name);
        Self { user: Some(user) }
    }
}
