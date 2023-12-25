use std::sync::Arc;

use libsql_client::Client;
use sm3::{Digest, Sm3};
use snafu::{ResultExt, Snafu};
use tonic::{Request, Response, Status};
use tracing::error;

use crate::api::v2::{
    user_service_server, CreateUserAccessTokenRequest, CreateUserAccessTokenResponse,
    CreateUserRequest, CreateUserResponse, DeleteUserAccessTokenRequest,
    DeleteUserAccessTokenResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, GetUserSettingRequest, GetUserSettingResponse, ListUserAccessTokensRequest,
    ListUserAccessTokensResponse, ListUsersRequest, ListUsersResponse, UpdateUserRequest,
    UpdateUserResponse, UpdateUserSettingRequest, UpdateUserSettingResponse, User,
};
use crate::dao::user::Error as DaoErr;
use crate::dao::user::UserDao;
use crate::dao::user_setting::UserSettingDao;

use super::get_current_user;

#[derive(Debug, Clone)]
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

    pub async fn sign_in(&self, name: String, password: String) -> Result<User, Error> {
        let mut hasher = Sm3::new();
        hasher.update(password);

        let password_hash = hex::encode(hasher.finalize());
        self.user_dao
            .find_user(name, Some(password_hash))
            .await
            .context(Login)
    }

    pub async fn petch_user(&self, id: i32) -> Result<User, Error> {
        let rs = self.user_dao.petch_user(id).await;
        if let Err(DaoErr::Inexistent) = rs {
            rs.context(UserNotFound {
                ident: id.to_string(),
            })
        } else {
            rs.context(QueryUserFailed)
        }
    }

    pub async fn host_user(&self) -> Result<User, Error> {
        let rs = self.user_dao.host_user().await;
        if let Err(DaoErr::Inexistent) = rs {
            rs.context(UserNotFound {
                ident: String::new(),
            })
        } else {
            rs.context(QueryUserFailed)
        }
    }

    pub async fn find_user(&self, name: String) -> Result<User, Error> {
        let rs = self.user_dao.find_user(name.clone(), None).await;
        if let Err(DaoErr::Inexistent) = rs {
            rs.context(UserNotFound { ident: name })
        } else {
            rs.context(QueryUserFailed)
        }
    }
}

#[tonic::async_trait]
impl user_service_server::UserService for UserService {
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        todo!()
    }
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let name = request.into_inner().get_name()?;
        let user = self.user_dao.find_user(name.clone(), None).await?;
        Ok(Response::new(user.into()))
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
        todo!()
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
    async fn get_user_setting(
        &self,
        request: Request<GetUserSettingRequest>,
    ) -> Result<Response<GetUserSettingResponse>, Status> {
        let user = get_current_user(&request)?;
        let settings = self.setting_dao.find_setting(user.id).await?;
        Ok(Response::new(settings.into()))
    }
    async fn update_user_setting(
        &self,
        request: Request<UpdateUserSettingRequest>,
    ) -> Result<Response<UpdateUserSettingResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(
        display("Incorrect login credentials, please try again: {source}"),
        context(suffix(false))
    )]
    Login { source: crate::dao::user::Error },
    #[snafu(display("User not found: {ident}, {source}"), context(suffix(false)))]
    UserNotFound {
        ident: String,
        source: crate::dao::user::Error,
    },
    #[snafu(display("Failed to find user: {source}"), context(suffix(false)))]
    QueryUserFailed { source: crate::dao::user::Error },
    #[snafu(
        display("Failed to find userSettingList: {source}"),
        context(suffix(false))
    )]
    QuerySettingFailed {
        source: crate::dao::user_setting::Error,
    },
}

impl From<crate::dao::user_setting::Error> for Status {
    fn from(value: crate::dao::user_setting::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}
