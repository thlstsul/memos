use std::sync::Arc;

use async_trait::async_trait;
use snafu::{ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::api::v1::generate::create_session_request::Credentials;
use crate::api::v1::generate::{CreateSessionResponse, GetCurrentSessionResponse};
use crate::{
    api::v1::generate::{
        CreateSessionRequest, DeleteSessionRequest, GetCurrentSessionRequest, User,
        auth_service_server::{self, AuthServiceServer},
    },
    ctrl::AuthSession,
};

use super::{EmptyService, RequestExt};

#[async_trait]
pub trait AuthService: auth_service_server::AuthService + Clone + Send + Sync + 'static {
    fn auth_server(self: Arc<Self>) -> AuthServiceServer<Self> {
        AuthServiceServer::from_arc(self)
    }
}

#[async_trait]
impl AuthService for EmptyService {}

#[tonic::async_trait]
impl auth_service_server::AuthService for EmptyService {
    async fn get_current_session(
        &self,
        request: Request<GetCurrentSessionRequest>,
    ) -> Result<Response<GetCurrentSessionResponse>, Status> {
        let user = request.get_current_user()?;
        let mut user: User = user.clone().into();
        user.password = "".to_string();
        Ok(Response::new(GetCurrentSessionResponse {
            user: Some(user),
            last_accessed_at: None,
        }))
    }

    async fn create_session(
        &self,
        mut request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionResponse>, Status> {
        let Some(Credentials::PasswordCredentials(ref creds)) = request.get_ref().credentials
        else {
            return Err(Status::unimplemented("unimplemented"));
        };

        if let Some(session) = request.extensions_mut().get_mut::<AuthSession>() {
            let user = match session.authenticate(creds.clone()).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(Status::unauthenticated(
                        "Incorrect login credentials, please try again",
                    ));
                }
                Err(e) => return Err(Status::internal(e.to_string())),
            };

            session.login(&user).await.context(Login)?;
            Ok(Response::new(CreateSessionResponse {
                user: Some(user.into()),
                last_accessed_at: None,
            }))
        } else {
            Err(Status::internal("Auth layer uninitialized"))
        }
    }

    /// SignOut signs out the user.
    async fn delete_session(
        &self,
        mut request: Request<DeleteSessionRequest>,
    ) -> Result<Response<()>, Status> {
        if let Some(session) = request.extensions_mut().get_mut::<AuthSession>() {
            session.logout().await.context(Logout)?;
        }
        Ok(Response::new(()))
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to login: {source}"), context(suffix(false)))]
    Login { source: crate::ctrl::AuthError },
    #[snafu(display("Failed to logout: {source}"), context(suffix(false)))]
    Logout { source: crate::ctrl::AuthError },
}
