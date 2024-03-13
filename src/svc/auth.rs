use snafu::{ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::{
    api::v2::{
        auth_service_server::{self, AuthServiceServer},
        GetAuthStatusRequest, GetAuthStatusResponse, SignInRequest, SignInResponse,
        SignInWithSsoRequest, SignInWithSsoResponse, SignOutRequest, SignOutResponse,
        SignUpRequest, SignUpResponse,
    },
    ctrl::auth::{AuthSession, Backend},
};

use super::get_current_user;

pub struct AuthService;

impl AuthService {
    pub fn server() -> AuthServiceServer<AuthService> {
        AuthServiceServer::new(AuthService)
    }
}

#[tonic::async_trait]
impl auth_service_server::AuthService for AuthService {
    async fn get_auth_status(
        &self,
        request: Request<GetAuthStatusRequest>,
    ) -> Result<Response<GetAuthStatusResponse>, Status> {
        let user = get_current_user(&request)?;
        Ok(Response::new(user.clone().into()))
    }

    async fn sign_in(
        &self,
        mut request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        let creds = request.get_ref().clone();
        let mut resp_user = None;
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
            resp_user = Some(user);
        }
        Ok(Response::new(SignInResponse { user: resp_user }))
    }
    /// SignInWithSSO signs in the user with the given SSO code.
    async fn sign_in_with_sso(
        &self,
        request: Request<SignInWithSsoRequest>,
    ) -> Result<Response<SignInWithSsoResponse>, Status> {
        unimplemented!()
    }
    /// SignUp signs up the user with the given username and password.
    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        unimplemented!()
    }
    /// SignOut signs out the user.
    async fn sign_out(
        &self,
        mut request: Request<SignOutRequest>,
    ) -> Result<Response<SignOutResponse>, Status> {
        if let Some(session) = request.extensions_mut().get_mut::<AuthSession>() {
            session.logout().context(Logout)?;
        }
        Ok(Response::new(SignOutResponse {}))
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to login: {source}"), context(suffix(false)))]
    Login { source: axum_login::Error<Backend> },
    #[snafu(display("Failed to logout: {source}"), context(suffix(false)))]
    Logout { source: axum_login::Error<Backend> },
}
