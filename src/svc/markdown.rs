use std::sync::Arc;

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use crate::api::v1::gen::{
    markdown_service_server::{self, MarkdownServiceServer},
    GetLinkMetadataRequest, LinkMetadata, ParseMarkdownRequest, ParseMarkdownResponse,
    RestoreMarkdownNodesRequest, RestoreMarkdownNodesResponse, StringifyMarkdownNodesRequest,
    StringifyMarkdownNodesResponse,
};

use super::EmptyService;

#[async_trait]
pub trait MarkdownService:
    markdown_service_server::MarkdownService + Clone + Send + Sync + 'static
{
    fn markdown_server(self: Arc<Self>) -> MarkdownServiceServer<Self> {
        MarkdownServiceServer::from_arc(self)
    }
}

#[async_trait]
impl MarkdownService for EmptyService {}

#[tonic::async_trait]
impl markdown_service_server::MarkdownService for EmptyService {
    async fn parse_markdown(
        &self,
        request: Request<ParseMarkdownRequest>,
    ) -> Result<Response<ParseMarkdownResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn get_link_metadata(
        &self,
        request: Request<GetLinkMetadataRequest>,
    ) -> Result<Response<LinkMetadata>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn restore_markdown_nodes(
        &self,
        request: Request<RestoreMarkdownNodesRequest>,
    ) -> Result<Response<RestoreMarkdownNodesResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }

    async fn stringify_markdown_nodes(
        &self,
        request: Request<StringifyMarkdownNodesRequest>,
    ) -> Result<Response<StringifyMarkdownNodesResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
}
