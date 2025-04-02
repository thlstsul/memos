use std::sync::Arc;

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use crate::api::v1::gen::{
    inbox_service_server::{self, InboxServiceServer},
    DeleteInboxRequest, Inbox, ListInboxesRequest, ListInboxesResponse, UpdateInboxRequest,
};

use super::EmptyService;

#[async_trait]
pub trait InboxService: inbox_service_server::InboxService + Clone + Send + Sync + 'static {
    fn inbox_server(self: Arc<Self>) -> InboxServiceServer<Self> {
        InboxServiceServer::from_arc(self)
    }
}

#[async_trait]
impl InboxService for EmptyService {}

#[tonic::async_trait]
impl inbox_service_server::InboxService for EmptyService {
    async fn list_inboxes(
        &self,
        request: Request<ListInboxesRequest>,
    ) -> Result<Response<ListInboxesResponse>, Status> {
        // TODO
        Ok(Response::new(ListInboxesResponse::default()))
    }
    async fn update_inbox(
        &self,
        request: Request<UpdateInboxRequest>,
    ) -> Result<Response<Inbox>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    async fn delete_inbox(
        &self,
        request: Request<DeleteInboxRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
}
