use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use libsql_client::Client;

use crate::{
    api::{
        memo::{FindMemo, Memo},
        v2::{
            memo_service_server, CreateMemoCommentRequest, CreateMemoCommentResponse,
            CreateMemoRequest, CreateMemoResponse, GetMemoRequest, GetMemoResponse,
            ListMemoCommentsRequest, ListMemoCommentsResponse, ListMemosRequest, ListMemosResponse,
        },
    },
    dao::memo::MemoDao,
};

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

    pub async fn list_memos(&self, cond: FindMemo) -> Result<Vec<Memo>, Error> {
        self.dao.list_memos(cond).await.context(ListMemoFailed)
    }
}

#[tonic::async_trait]
impl memo_service_server::MemoService for MemoService {
    async fn create_memo(
        &self,
        request: Request<CreateMemoRequest>,
    ) -> Result<Response<CreateMemoResponse>, Status> {
        todo!()
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
    async fn create_memo_comment(
        &self,
        request: Request<CreateMemoCommentRequest>,
    ) -> Result<Response<CreateMemoCommentResponse>, Status> {
        todo!()
    }
    async fn list_memo_comments(
        &self,
        request: Request<ListMemoCommentsRequest>,
    ) -> Result<Response<ListMemoCommentsResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to find memo list"), context(suffix(false)))]
    ListMemoFailed { source: crate::dao::memo::Error },
}
