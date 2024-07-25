use std::sync::Arc;

use crate::api::v1::gen::{
    webhook_service_server::{self, WebhookServiceServer},
    CreateWebhookRequest, DeleteWebhookRequest, GetWebhookRequest, ListWebhooksRequest,
    ListWebhooksResponse, UpdateWebhookRequest, Webhook,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

use super::EmptyService;

#[async_trait]
pub trait WebhookService:
    webhook_service_server::WebhookService + Clone + Send + Sync + 'static
{
    fn webhook_server(self: Arc<Self>) -> WebhookServiceServer<Self> {
        WebhookServiceServer::from_arc(self)
    }
}

#[async_trait]
impl WebhookService for EmptyService {}

#[tonic::async_trait]
impl webhook_service_server::WebhookService for EmptyService {
    async fn create_webhook(
        &self,
        request: Request<CreateWebhookRequest>,
    ) -> Result<Response<Webhook>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn get_webhook(
        &self,
        request: Request<GetWebhookRequest>,
    ) -> Result<Response<Webhook>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn list_webhooks(
        &self,
        request: Request<ListWebhooksRequest>,
    ) -> Result<Response<ListWebhooksResponse>, Status> {
        Ok(Response::new(ListWebhooksResponse { webhooks: vec![] }))
    }
    async fn update_webhook(
        &self,
        request: Request<UpdateWebhookRequest>,
    ) -> Result<Response<Webhook>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn delete_webhook(
        &self,
        request: Request<DeleteWebhookRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
}
