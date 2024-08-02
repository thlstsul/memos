use snafu::Snafu;
use tonic::{Code, Request, Status};
use tracing::error;

use crate::ctrl::AuthSession;
use crate::dao::memo::{CreateMemoError, DeleteMemoError, ListMemoError, UpdateMemoError};
use crate::dao::resource::{
    CreateResourceError, DeleteResourceError, GetResourceError, ListResourceError,
    RelateResourceError, SetResourceError,
};
use crate::dao::user::{
    FindUserError, FindUserSettingError, GetHostUserError, PetchUserError, UpsertUserSettingError,
};
use crate::dao::workspace::FindWorkspaceSettingError;
use crate::model::user::User;

pub mod auth;
pub mod idp;
pub mod inbox;
pub mod markdown;
pub mod memo;
pub mod resource;
pub mod session;
pub mod user;
pub mod webhook;
pub mod workspace;

#[derive(Debug, Clone)]
pub struct EmptyService;

#[derive(Debug, Clone)]
pub struct Service<R> {
    repo: R,
}

impl<R> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait RequestExt {
    fn get_current_user(&self) -> Result<&User, CurrentUserError>;
}

impl<T> RequestExt for Request<T> {
    fn get_current_user(&self) -> Result<&User, CurrentUserError> {
        if let Some(AuthSession {
            user: Some(user), ..
        }) = self.extensions().get::<AuthSession>()
        {
            Ok(user)
        } else {
            Err(CurrentUserError)
        }
    }
}

macro_rules! into_status {
    ($e:path, $c:path) => {
        impl From<$e> for Status {
            fn from(value: $e) -> Self {
                error!("{value}");
                Status::new($c, value.to_string())
            }
        }
    };
}

#[derive(Debug, Snafu)]
#[snafu(display("Failed to get current user"), context(suffix(false)))]
pub struct CurrentUserError;

impl From<user::Error> for Status {
    fn from(value: user::Error) -> Self {
        error!("{value}");
        match value {
            user::Error::UserNotFound { .. } => Status::not_found(value.to_string()),
            _ => Status::internal(value.to_string()),
        }
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

into_status!(crate::api::prefix::Error, Code::InvalidArgument);
into_status!(CurrentUserError, Code::Unauthenticated);
into_status!(auth::Error, Code::Internal);
into_status!(CreateMemoError, Code::Internal);
into_status!(DeleteMemoError, Code::Internal);
into_status!(ListMemoError, Code::Internal);
into_status!(UpdateMemoError, Code::Internal);
into_status!(CreateResourceError, Code::Internal);
into_status!(DeleteResourceError, Code::Internal);
into_status!(GetResourceError, Code::Internal);
into_status!(ListResourceError, Code::Internal);
into_status!(RelateResourceError, Code::Internal);
into_status!(SetResourceError, Code::Internal);
into_status!(FindUserError, Code::Internal);
into_status!(FindUserSettingError, Code::Internal);
into_status!(GetHostUserError, Code::Internal);
into_status!(PetchUserError, Code::Internal);
into_status!(UpsertUserSettingError, Code::Internal);
into_status!(FindWorkspaceSettingError, Code::Internal);
