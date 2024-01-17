use std::sync::Arc;

use libsql_client::Client;
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
    pub fn get_user(client: &Arc<Client>) -> UserServiceServer<UserService> {
        let user = UserService::new(client);
        UserServiceServer::new(user)
    }

    pub fn get_tag(client: &Arc<Client>) -> TagServiceServer<TagService> {
        let tag = TagService::new(client);
        TagServiceServer::new(tag)
    }

    pub fn get_auth() -> AuthServiceServer<AuthService> {
        AuthServiceServer::new(AuthService)
    }

    pub fn get_memo(client: &Arc<Client>) -> MemoServiceServer<MemoService> {
        let memo = MemoService::new(client);
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

impl From<crate::dao::system_setting::Error> for Status {
    fn from(value: crate::dao::system_setting::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}

impl From<crate::dao::user_setting::Error> for Status {
    fn from(value: crate::dao::user_setting::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}

impl From<crate::dao::user::Error> for Status {
    fn from(value: crate::dao::user::Error) -> Self {
        error!("{value}");
        match value {
            crate::dao::user::Error::Inexistent => Status::not_found(value.to_string()),
            _ => Status::internal(value.to_string()),
        }
    }
}

impl From<crate::dao::memo::Error> for Status {
    fn from(value: crate::dao::memo::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}

impl From<crate::util::Error> for Status {
    fn from(value: crate::util::Error) -> Self {
        error!("{value}");
        Status::invalid_argument(value.to_string())
    }
}

impl From<crate::api::memo::Error> for Status {
    fn from(value: crate::api::memo::Error) -> Self {
        error!("{value}");
        Status::invalid_argument(value.to_string())
    }
}

impl From<crate::api::user::Error> for Status {
    fn from(value: crate::api::user::Error) -> Self {
        error!("{value}");
        Status::invalid_argument(value.to_string())
    }
}
