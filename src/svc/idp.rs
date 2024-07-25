use std::sync::Arc;

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use crate::api::v1::gen::{
    identity_provider_service_server::{self, IdentityProviderServiceServer},
    CreateIdentityProviderRequest, DeleteIdentityProviderRequest, GetIdentityProviderRequest,
    IdentityProvider, ListIdentityProvidersRequest, ListIdentityProvidersResponse,
    UpdateIdentityProviderRequest,
};

use super::EmptyService;

#[async_trait]
pub trait IDPService:
    identity_provider_service_server::IdentityProviderService + Clone + Send + Sync + 'static
{
    fn idp_server(self: Arc<Self>) -> IdentityProviderServiceServer<Self> {
        IdentityProviderServiceServer::from_arc(self)
    }
}

#[async_trait]
impl IDPService for EmptyService {}

#[tonic::async_trait]
impl identity_provider_service_server::IdentityProviderService for EmptyService {
    async fn list_identity_providers(
        &self,
        request: Request<ListIdentityProvidersRequest>,
    ) -> Result<Response<ListIdentityProvidersResponse>, Status> {
        Ok(Response::new(ListIdentityProvidersResponse {
            identity_providers: vec![],
        }))
    }

    async fn get_identity_provider(
        &self,
        request: Request<GetIdentityProviderRequest>,
    ) -> Result<Response<IdentityProvider>, Status> {
        todo!()
    }

    async fn create_identity_provider(
        &self,
        request: Request<CreateIdentityProviderRequest>,
    ) -> Result<Response<IdentityProvider>, Status> {
        todo!()
    }

    async fn update_identity_provider(
        &self,
        request: Request<UpdateIdentityProviderRequest>,
    ) -> Result<Response<IdentityProvider>, Status> {
        todo!()
    }

    async fn delete_identity_provider(
        &self,
        request: Request<DeleteIdentityProviderRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }
}
