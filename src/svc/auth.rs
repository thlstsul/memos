use tonic::{Request, Response, Status};

use crate::api::v2::{
    auth_service_server::{self, AuthServiceServer},
    GetAuthStatusRequest, GetAuthStatusResponse,
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
}
