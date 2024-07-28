use std::sync::Arc;

use async_trait::async_trait;
use snafu::{ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::{
    api::v1::gen::{
        auth_service_server::{self, AuthServiceServer},
        GetAuthStatusRequest, SignInRequest, SignInWithSsoRequest, SignOutRequest, SignUpRequest,
        User,
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
    async fn get_auth_status(
        &self,
        request: Request<GetAuthStatusRequest>,
    ) -> Result<Response<User>, Status> {
        let user = request.get_current_user()?;
        let mut user: User = user.clone().into();
        user.password = "".to_string();
        Ok(Response::new(user))
    }

    async fn sign_in(&self, mut request: Request<SignInRequest>) -> Result<Response<User>, Status> {
        let creds = request.get_ref().clone();
        if let Some(session) = request.extensions_mut().get_mut::<AuthSession>() {
            let user = match session.authenticate(creds).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(Status::unauthenticated(
                        "Incorrect login credentials, please try again",
                    ));
                }
                Err(e) => return Err(Status::internal(e.to_string())),
            };

            session.login(&user).await.context(Login)?;
            Ok(Response::new(user.into()))
        } else {
            Err(Status::internal("Auth layer uninitialized"))
        }
    }
    /// SignInWithSSO signs in the user with the given SSO code.
    async fn sign_in_with_sso(
        &self,
        request: Request<SignInWithSsoRequest>,
    ) -> Result<Response<User>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// SignUp signs up the user with the given username and password.
    async fn sign_up(&self, request: Request<SignUpRequest>) -> Result<Response<User>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// SignOut signs out the user.
    async fn sign_out(&self, mut request: Request<SignOutRequest>) -> Result<Response<()>, Status> {
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
