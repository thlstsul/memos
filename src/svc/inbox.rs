use tonic::{Request, Response, Status};

use crate::api::v2::{
    inbox_service_server::{self, InboxServiceServer},
    DeleteInboxRequest, DeleteInboxResponse, ListInboxesRequest, ListInboxesResponse,
    UpdateInboxRequest, UpdateInboxResponse,
};

pub struct InboxService {
    // TODO
}

impl InboxService {
    pub fn server() -> InboxServiceServer<InboxService> {
        InboxServiceServer::new(InboxService {})
    }
}

#[tonic::async_trait]
impl inbox_service_server::InboxService for InboxService {
    async fn list_inboxes(
        &self,
        request: Request<ListInboxesRequest>,
    ) -> Result<Response<ListInboxesResponse>, Status> {
        // TODO
        Ok(Response::new(ListInboxesResponse { inboxes: vec![] }))
    }
    async fn update_inbox(
        &self,
        request: Request<UpdateInboxRequest>,
    ) -> Result<Response<UpdateInboxResponse>, Status> {
        unimplemented!()
    }
    async fn delete_inbox(
        &self,
        request: Request<DeleteInboxRequest>,
    ) -> Result<Response<DeleteInboxResponse>, Status> {
        unimplemented!()
    }
}
