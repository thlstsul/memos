use snafu::Snafu;
use tonic::{Request, Status};
use tracing::error;

use crate::{api::v2::User, ctrl::auth::AuthSession};

pub mod auth;
pub mod inbox;
pub mod memo;
pub mod resource;
pub mod system;
pub mod tag;
pub mod user;
pub mod webhook;
pub mod workspace;

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

impl From<resource::Error> for Status {
    fn from(value: resource::Error) -> Self {
        error!("{value}");
        match value {
            resource::Error::ResourceNotFound { .. } => Status::not_found(value.to_string()),
            _ => Status::internal(value.to_string()),
        }
    }
}

impl From<auth::Error> for Status {
    fn from(value: auth::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}
