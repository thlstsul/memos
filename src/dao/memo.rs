use async_trait::async_trait;
use snafu::Snafu;

use crate::model::memo::{CreateMemo, FindMemo, Memo, UpdateMemo};

#[async_trait]
pub trait MemoRepository: Clone + Send + Sync + 'static {
    async fn create_memo(&self, memo: CreateMemo) -> Result<Option<Memo>, CreateMemoError>;
    async fn list_memos(&self, find: FindMemo) -> Result<Vec<Memo>, ListMemoError>;
    async fn delete_memo(&self, memo_id: i32) -> Result<(), DeleteMemoError>;
    async fn update_memo(&self, update: UpdateMemo) -> Result<(), UpdateMemoError>;
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to create memo: {source}"))]
pub struct CreateMemoError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to list memo: {source}"))]
pub struct ListMemoError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to delete memo: {source}"))]
pub struct DeleteMemoError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to update memo: {source}"))]
pub struct UpdateMemoError {
    source: anyhow::Error,
}
