use sm3::{Digest, Sm3};
use snafu::{OptionExt, ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::api::v2::{
    user_service_server, CreateUserAccessTokenRequest, CreateUserAccessTokenResponse,
    CreateUserRequest, CreateUserResponse, DeleteUserAccessTokenRequest,
    DeleteUserAccessTokenResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, GetUserSettingRequest, GetUserSettingResponse, ListUserAccessTokensRequest,
    ListUserAccessTokensResponse, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
    UpdateUserResponse, UpdateUserSettingRequest, UpdateUserSettingResponse, User,
};
use crate::dao::user::UserDao;
use crate::dao::user_setting::UserSettingDao;
use crate::state::AppState;

use super::get_current_user;

#[derive(Debug, Clone)]
pub struct UserService {
    user_dao: UserDao,
    setting_dao: UserSettingDao,
}

impl UserService {
    pub fn new(state: &AppState) -> Self {
        Self {
            user_dao: UserDao {
                state: state.clone(),
            },
            setting_dao: UserSettingDao {
                state: state.clone(),
            },
        }
    }

    pub async fn sign_in(&self, name: String, password: String) -> Result<User, Error> {
        let mut hasher = Sm3::new();
        hasher.update(password);

        let password_hash = hex::encode(hasher.finalize());
        self.user_dao
            .find_user(name, Some(password_hash))
            .await
            .context(QueryUserFailed)?
            .context(Login)
    }

    pub async fn petch_user(&self, id: i32) -> Result<User, Error> {
        self.user_dao
            .petch_user(id)
            .await
            .context(QueryUserFailed)?
            .context(UserNotFound {
                ident: Some(id.to_string()),
            })
    }

    pub async fn host_user(&self) -> Result<User, Error> {
        self.user_dao
            .host_user()
            .await
            .context(QueryUserFailed)?
            .context(UserNotFound { ident: None })
    }

    pub async fn find_user(&self, name: String) -> Result<User, Error> {
        self.user_dao
            .find_user(name.clone(), None)
            .await
            .context(QueryUserFailed)?
            .context(UserNotFound { ident: Some(name) })
    }
}

#[tonic::async_trait]
impl user_service_server::UserService for UserService {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let name = request.into_inner().get_name().context(InvalidUsername)?;
        let user = Self::find_user(self, name).await?;
        Ok(Response::new(user.into()))
    }
    async fn get_user_setting(
        &self,
        request: Request<GetUserSettingRequest>,
    ) -> Result<Response<GetUserSettingResponse>, Status> {
        let user = get_current_user(&request)?;
        let settings = self
            .setting_dao
            .find_setting(user.id)
            .await
            .context(QuerySettingFailed)?;
        Ok(Response::new(settings.into()))
    }
    async fn update_user_setting(
        &self,
        request: Request<UpdateUserSettingRequest>,
    ) -> Result<Response<UpdateUserSettingResponse>, Status> {
        let user = get_current_user(&request)?;
        let settings = request.get_ref().as_settings(user.id);
        self.setting_dao
            .upsert_setting(settings)
            .await
            .context(UpsertSettingFailed)?;

        Ok(Response::new(UpdateUserSettingResponse {
            setting: request.get_ref().setting.clone(),
        }))
    }
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        todo!()
    }
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        todo!()
    }
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        todo!()
    }
    /// ListUserAccessTokens returns a list of access tokens for a user.
    async fn list_user_access_tokens(
        &self,
        request: Request<ListUserAccessTokensRequest>,
    ) -> Result<Response<ListUserAccessTokensResponse>, Status> {
        // TODO
        Ok(Response::new(ListUserAccessTokensResponse {
            access_tokens: vec![],
        }))
    }
    /// CreateUserAccessToken creates a new access token for a user.
    async fn create_user_access_token(
        &self,
        request: Request<CreateUserAccessTokenRequest>,
    ) -> Result<Response<CreateUserAccessTokenResponse>, Status> {
        todo!()
    }
    /// DeleteUserAccessToken deletes an access token for a user.
    async fn delete_user_access_token(
        &self,
        request: Request<DeleteUserAccessTokenRequest>,
    ) -> Result<Response<DeleteUserAccessTokenResponse>, Status> {
        todo!()
    }
    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(
        display("Incorrect login credentials, please try again"),
        context(suffix(false))
    )]
    Login,

    #[snafu(display("User not found: {ident:?}"), context(suffix(false)))]
    UserNotFound { ident: Option<String> },

    #[snafu(display("Failed to find user: {source}"), context(suffix(false)))]
    QueryUserFailed { source: crate::dao::Error },

    #[snafu(
        display("Failed to find user setting: {source}"),
        context(suffix(false))
    )]
    QuerySettingFailed { source: crate::dao::Error },

    #[snafu(
        display("Failed to update/insert user setting: {source}"),
        context(suffix(false))
    )]
    UpsertSettingFailed { source: crate::dao::Error },

    #[snafu(display("Invalid username: {source}"), context(suffix(false)))]
    InvalidUsername { source: crate::api::user::Error },
}
