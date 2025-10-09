use std::sync::Arc;

use crate::api::v1::generate::{
    CreateUserWebhookRequest, DeleteUserWebhookRequest, GetUserStatsRequest,
    ListAllUserStatsRequest, ListAllUserStatsResponse, ListUserSettingsRequest,
    ListUserSettingsResponse, ListUserWebhooksRequest, ListUserWebhooksResponse,
    UpdateUserWebhookRequest, UserStats, UserWebhook,
};
use crate::dao::memo::MemoRepository;
use crate::dao::resource::ResourceRepository;
use crate::dao::workspace::WorkspaceRepository;
use crate::google::api::HttpBody;
use crate::model::user::User as UserModel;
use crate::{
    api::{
        prefix::ExtractName,
        v1::generate::{
            CreateUserAccessTokenRequest, CreateUserRequest, DeleteUserAccessTokenRequest,
            DeleteUserRequest, GetUserAvatarRequest, GetUserRequest, GetUserSettingRequest,
            ListUserAccessTokensRequest, ListUserAccessTokensResponse, ListUserSessionsRequest,
            ListUserSessionsResponse, ListUsersRequest, ListUsersResponse,
            RevokeUserSessionRequest, UpdateUserRequest, UpdateUserSettingRequest, User,
            UserAccessToken, UserSetting,
            user_service_server::{self, UserServiceServer},
        },
    },
    dao::user::UserRepository,
};
use async_trait::async_trait;
use sm3::{Digest, Sm3};
use snafu::{OptionExt, Snafu};
use tonic::{Request, Response, Status};

use super::memo::MemoService;
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
impl<R: UserRepository + MemoRepository + ResourceRepository + WorkspaceRepository> UserService
    for Service<R>
{
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
impl<R: UserRepository + MemoRepository + ResourceRepository + WorkspaceRepository>
    user_service_server::UserService for Service<R>
{
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

    /// GetUserStats returns the stats of a user.
    async fn get_user_stats(
        &self,
        request: Request<GetUserStatsRequest>,
    ) -> Result<Response<UserStats>, Status> {
        let user = request.get_current_user().ok();
        let result = self.get_user_memo_stats(user).await?;
        Ok(Response::new(result))
    }

    /// ListAllUserStats returns all user stats.
    async fn list_all_user_stats(
        &self,
        request: Request<ListAllUserStatsRequest>,
    ) -> Result<Response<ListAllUserStatsResponse>, Status> {
        // TODO
        let result = self.get_user_memo_stats(None).await?;
        Ok(Response::new(ListAllUserStatsResponse {
            stats: vec![result],
        }))
    }

    /// ListUserSettings returns a list of user settings.
    async fn list_user_settings(
        &self,
        request: Request<ListUserSettingsRequest>,
    ) -> Result<Response<ListUserSettingsResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
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
        Ok(Response::new(ListUserAccessTokensResponse::default()))
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
    async fn get_user_avatar(
        &self,
        request: Request<GetUserAvatarRequest>,
    ) -> Result<Response<HttpBody>, Status> {
        todo!()
    }
    async fn list_user_sessions(
        &self,
        request: Request<ListUserSessionsRequest>,
    ) -> Result<Response<ListUserSessionsResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn revoke_user_session(
        &self,
        request: Request<RevokeUserSessionRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// ListUserWebhooks returns a list of webhooks for a user.
    async fn list_user_webhooks(
        &self,
        request: Request<ListUserWebhooksRequest>,
    ) -> Result<Response<ListUserWebhooksResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// CreateUserWebhook creates a new webhook for a user.
    async fn create_user_webhook(
        &self,
        request: Request<CreateUserWebhookRequest>,
    ) -> Result<Response<UserWebhook>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// UpdateUserWebhook updates an existing webhook for a user.
    async fn update_user_webhook(
        &self,
        request: Request<UpdateUserWebhookRequest>,
    ) -> Result<Response<UserWebhook>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// DeleteUserWebhook deletes a webhook for a user.
    async fn delete_user_webhook(
        &self,
        request: Request<DeleteUserWebhookRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
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
