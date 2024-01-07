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

pub fn get_current_user<'a, T>(request: &'a Request<T>) -> Result<&'a User, Error> {
    if let Some(AuthSession {
        user: Some(user), ..
    }) = request.extensions().get::<AuthSession>()
    {
        Ok(user)
    } else {
        Err(Error::CurrentUser)
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
        Status::internal(value.to_string())
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

impl From<crate::dao::memo::Error> for Status {
    fn from(value: crate::dao::memo::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}