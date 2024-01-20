use snafu::Snafu;
use tonic::{Request, Status};
use tracing::error;

use crate::{
    api::v2::{
        auth_service_server::AuthServiceServer, inbox_service_server::InboxServiceServer,
        memo_service_server::MemoServiceServer, tag_service_server::TagServiceServer,
        user_service_server::UserServiceServer, User,
    },
    ctrl::auth::AuthSession,
    state::AppState,
};

use self::{
    auth::AuthService, inbox::InboxService, memo::MemoService, tag::TagService, user::UserService,
};

pub mod auth;
pub mod inbox;
pub mod memo;
pub mod resource;
pub mod system;
pub mod tag;
pub mod user;

pub struct ServiceFactory;

impl ServiceFactory {
    pub fn get_user(state: &AppState) -> UserServiceServer<UserService> {
        let user = UserService::new(state);
        UserServiceServer::new(user)
    }

    pub fn get_tag(state: &AppState) -> TagServiceServer<TagService> {
        let tag = TagService::new(state);
        TagServiceServer::new(tag)
    }

    pub fn get_auth() -> AuthServiceServer<AuthService> {
        AuthServiceServer::new(AuthService)
    }

    pub fn get_memo(state: &AppState) -> MemoServiceServer<MemoService> {
        let memo = MemoService::new(state);
        MemoServiceServer::new(memo)
    }

    pub fn get_inbox() -> InboxServiceServer<InboxService> {
        InboxServiceServer::new(InboxService {})
    }
}

pub fn get_current_user<T>(request: &Request<T>) -> Result<&User, Error> {
    if let Some(AuthSession {
        user: Some(user), ..
    }) = request.extensions().get::<AuthSession>()
    {
        Ok(user)
    } else {
        CurrentUser.fail()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to get current user"), context(suffix(false)))]
    CurrentUser,
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        error!("{value}");
        Status::unauthenticated(value.to_string())
    }
}

impl From<user::Error> for Status {
    fn from(value: user::Error) -> Self {
        error!("{value}");
        match value {
            user::Error::UserNotFound { .. } => Status::not_found(value.to_string()),
            user::Error::InvalidUsername { .. } => Status::invalid_argument(value.to_string()),
            _ => Status::internal(value.to_string()),
        }
    }
}

impl From<system::Error> for Status {
    fn from(value: system::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}

impl From<tag::Error> for Status {
    fn from(value: tag::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}

impl From<memo::Error> for Status {
    fn from(value: memo::Error) -> Self {
        error!("{value}");
        match value {
            memo::Error::InvalidMemoFilter { .. } => Status::invalid_argument(value.to_string()),
            _ => Status::internal(value.to_string()),
        }
    }
}
