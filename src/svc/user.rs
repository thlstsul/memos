use std::sync::Arc;

use crate::google::api::HttpBody;
use crate::model::user::User as UserModel;
use crate::{
    api::{
        prefix::ExtractName,
        v1::gen::{
            user_service_server::{self, UserServiceServer},
            CreateUserAccessTokenRequest, CreateUserRequest, DeleteUserAccessTokenRequest,
            DeleteUserRequest, GetUserAvatarBinaryRequest, GetUserRequest, GetUserSettingRequest,
            ListUserAccessTokensRequest, ListUserAccessTokensResponse, ListUsersRequest,
            ListUsersResponse, SearchUsersRequest, SearchUsersResponse, UpdateUserRequest,
            UpdateUserSettingRequest, User, UserAccessToken, UserSetting,
        },
    },
    dao::user::UserRepository,
};
use async_trait::async_trait;
use sm3::{Digest, Sm3};
use snafu::{OptionExt, Snafu};
use tonic::{Request, Response, Status};

use super::{RequestExt, Service};

#[async_trait]
pub trait UserService: user_service_server::UserService + Clone + Send + Sync + 'static {
    fn user_server(self: Arc<Self>) -> UserServiceServer<Self> {
        UserServiceServer::from_arc(self)
    }

    async fn sign_in(&self, name: &str, password: &str) -> Result<UserModel, Error>;
    async fn petch_user(&self, id: i32) -> Result<UserModel, Error>;
    async fn find_user(&self, name: &str) -> Result<UserModel, Error>;
}

#[async_trait]
impl<R: UserRepository> UserService for Service<R> {
    async fn sign_in(&self, name: &str, password: &str) -> Result<UserModel, Error> {
        let mut hasher = Sm3::new();
        hasher.update(password);

        let password_hash = hex::encode(hasher.finalize());
        self.repo
            .find_user(name, Some(&password_hash))
            .await?
            .context(Login)
    }

    async fn petch_user(&self, id: i32) -> Result<UserModel, Error> {
        self.repo.petch_user(id).await?.context(UserNotFound {
            ident: id.to_string(),
        })
    }

    async fn find_user(&self, name: &str) -> Result<UserModel, Error> {
        self.repo
            .find_user(name, None)
            .await?
            .context(UserNotFound { ident: name })
    }
}

#[tonic::async_trait]
impl<R: UserRepository> user_service_server::UserService for Service<R> {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let name = request.into_inner().get_name();
        let user = Self::find_user(self, &name).await?;
        Ok(Response::new(user.into()))
    }

    async fn get_user_setting(
        &self,
        request: Request<GetUserSettingRequest>,
    ) -> Result<Response<UserSetting>, Status> {
        let user = request.get_current_user()?;
        let settings = self.repo.find_user_setting(user.id).await?;
        Ok(Response::new(settings.into()))
    }

    async fn update_user_setting(
        &self,
        request: Request<UpdateUserSettingRequest>,
    ) -> Result<Response<UserSetting>, Status> {
        let user = request.get_current_user()?;
        let settings = request.get_ref().as_settings(user.id);
        self.repo.upsert_user_setting(settings).await?;

        let settings = self.repo.find_user_setting(user.id).await?;

        Ok(Response::new(settings.into()))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<User>, Status> {
        Err(Status::unimplemented("unimplemented"))
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
    ) -> Result<Response<UserAccessToken>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// DeleteUserAccessToken deletes an access token for a user.
    async fn delete_user_access_token(
        &self,
        request: Request<DeleteUserAccessTokenRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn search_users(
        &self,
        request: Request<SearchUsersRequest>,
    ) -> Result<tonic::Response<SearchUsersResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn get_user_avatar_binary(
        &self,
        request: Request<GetUserAvatarBinaryRequest>,
    ) -> Result<Response<HttpBody>, Status> {
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

    #[snafu(display("User not found: {ident}"), context(suffix(false)))]
    UserNotFound { ident: String },

    #[snafu(context(false))]
    QueryUser {
        source: crate::dao::user::FindUserError,
    },

    #[snafu(context(false))]
    PetchUser {
        source: crate::dao::user::PetchUserError,
    },
}
