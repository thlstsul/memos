use std::sync::Arc;

use libsql_client::Client;
use sm3::{Digest, Sm3};
use snafu::{ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::api::v1::user_setting::UserSetting;
use crate::api::v2::{
    user_service_server, CreateUserAccessTokenRequest, CreateUserAccessTokenResponse,
    CreateUserRequest, CreateUserResponse, DeleteUserAccessTokenRequest,
    DeleteUserAccessTokenResponse, GetUserRequest, GetUserResponse, ListUserAccessTokensRequest,
    ListUserAccessTokensResponse, UpdateUserRequest, UpdateUserResponse, User,
};
use crate::dao::user::Error as DaoErr;
use crate::dao::user::UserDao;
use crate::dao::user_setting::UserSettingDao;

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

    pub async fn find_setting(&self, user_id: i32) -> Result<Vec<UserSetting>, Error> {
        self.setting_dao
            .find_setting(user_id)
            .await
            .context(QuerySettingFailed)
    }
}

#[tonic::async_trait]
impl user_service_server::UserService for UserService {
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
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(
        display("Incorrect login credentials, please try again"),
        context(suffix(false))
    )]
    Login { source: crate::dao::user::Error },
    #[snafu(display("User not found: {ident}"), context(suffix(false)))]
    UserNotFound {
        ident: String,
        source: crate::dao::user::Error,
    },
    #[snafu(display("Failed to find user"), context(suffix(false)))]
    QueryUserFailed { source: crate::dao::user::Error },
    #[snafu(display("Failed to find userSettingList"), context(suffix(false)))]
    QuerySettingFailed {
        source: crate::dao::user_setting::Error,
    },
}
