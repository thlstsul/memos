use snafu::Snafu;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::error;

use libsql_client::Client;

use crate::{
    api::{
        memo::CreateMemo,
        v2::{
            memo_service_server, CreateMemoCommentRequest, CreateMemoCommentResponse,
            CreateMemoRequest, CreateMemoResponse, DeleteMemoRequest, DeleteMemoResponse,
            GetMemoRequest, GetMemoResponse, GetUserMemosStatsRequest, GetUserMemosStatsResponse,
            ListMemoCommentsRequest, ListMemoCommentsResponse, ListMemoRelationsRequest,
            ListMemoRelationsResponse, ListMemoResourcesRequest, ListMemoResourcesResponse,
            ListMemosRequest, ListMemosResponse, SetMemoRelationsRequest, SetMemoRelationsResponse,
            SetMemoResourcesRequest, SetMemoResourcesResponse, UpdateMemoRequest,
            UpdateMemoResponse,
        },
    },
    dao::memo::MemoDao,
};

use super::get_current_user;

pub struct MemoService {
    dao: MemoDao,
}

impl MemoService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            dao: MemoDao {
                client: Arc::clone(client),
            },
        }
    }
}

#[tonic::async_trait]
impl memo_service_server::MemoService for MemoService {
    async fn create_memo(
        &self,
        request: Request<CreateMemoRequest>,
    ) -> Result<Response<CreateMemoResponse>, Status> {
        let user = get_current_user(&request)?;
        let req = request.get_ref();
        let create = CreateMemo {
            creator_id: user.id,
            content: req.content.clone(),
            visibility: req.visibility(),
        };
        let memo = self.dao.create_memo(create).await?;
        Ok(Response::new(memo.into()))
    }
    async fn list_memos(
        &self,
        request: Request<ListMemosRequest>,
    ) -> Result<Response<ListMemosResponse>, Status> {
        todo!()
    }
    async fn get_memo(
        &self,
        request: Request<GetMemoRequest>,
    ) -> Result<Response<GetMemoResponse>, Status> {
        todo!()
    }
    /// UpdateMemo updates a memo.
    async fn update_memo(
        &self,
        request: Request<UpdateMemoRequest>,
    ) -> Result<Response<UpdateMemoResponse>, Status> {
        todo!()
    }
    /// DeleteMemo deletes a memo by id.
    async fn delete_memo(
        &self,
        request: Request<DeleteMemoRequest>,
    ) -> Result<Response<DeleteMemoResponse>, Status> {
        todo!()
    }
    /// SetMemoResources sets resources for a memo.
    async fn set_memo_resources(
        &self,
        request: Request<SetMemoResourcesRequest>,
    ) -> Result<Response<SetMemoResourcesResponse>, Status> {
        todo!()
    }
    /// ListMemoResources lists resources for a memo.
    async fn list_memo_resources(
        &self,
        request: Request<ListMemoResourcesRequest>,
    ) -> Result<Response<ListMemoResourcesResponse>, Status> {
        todo!()
    }
    /// SetMemoRelations sets relations for a memo.
    async fn set_memo_relations(
        &self,
        request: Request<SetMemoRelationsRequest>,
    ) -> Result<Response<SetMemoRelationsResponse>, Status> {
        todo!()
    }
    /// ListMemoRelations lists relations for a memo.
    async fn list_memo_relations(
        &self,
        request: Request<ListMemoRelationsRequest>,
    ) -> Result<Response<ListMemoRelationsResponse>, Status> {
        todo!()
    }
    /// CreateMemoComment creates a comment for a memo.
    async fn create_memo_comment(
        &self,
        request: Request<CreateMemoCommentRequest>,
    ) -> Result<Response<CreateMemoCommentResponse>, Status> {
        todo!()
    }
    /// ListMemoComments lists comments for a memo.
    async fn list_memo_comments(
        &self,
        request: Request<ListMemoCommentsRequest>,
    ) -> Result<Response<ListMemoCommentsResponse>, Status> {
        todo!()
    }
    /// GetUserMemosStats gets stats of memos for a user.
    async fn get_user_memos_stats(
        &self,
        request: Request<GetUserMemosStatsRequest>,
    ) -> Result<Response<GetUserMemosStatsResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to find memo list: {source}"), context(suffix(false)))]
    ListMemoFailed { source: crate::dao::memo::Error },
}

impl From<crate::dao::memo::Error> for Status {
    fn from(value: crate::dao::memo::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}
