use crate::api::v2::{
    webhook_service_server::{self, WebhookServiceServer},
    CreateWebhookRequest, CreateWebhookResponse, DeleteWebhookRequest, DeleteWebhookResponse,
    GetWebhookRequest, GetWebhookResponse, ListWebhooksRequest, ListWebhooksResponse,
    UpdateWebhookRequest, UpdateWebhookResponse,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};
pub struct WebhookService;

impl WebhookService {
    pub fn server() -> WebhookServiceServer<WebhookService> {
        WebhookServiceServer::new(WebhookService)
    }
}

#[async_trait]
impl webhook_service_server::WebhookService for WebhookService {
    async fn create_webhook(
        &self,
        request: Request<CreateWebhookRequest>,
    ) -> Result<Response<CreateWebhookResponse>, Status> {
        unimplemented!()
    }
    async fn get_webhook(
        &self,
        request: Request<GetWebhookRequest>,
    ) -> Result<Response<GetWebhookResponse>, Status> {
        unimplemented!()
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
    ) -> Result<Response<UpdateWebhookResponse>, Status> {
        unimplemented!()
    }
    async fn delete_webhook(
        &self,
        request: Request<DeleteWebhookRequest>,
    ) -> Result<Response<DeleteWebhookResponse>, Status> {
        unimplemented!()
    }
}
