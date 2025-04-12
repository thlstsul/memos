mod create;
mod find;
mod update;

use async_trait::async_trait;

use crate::dao::memo::{
    CreateMemoError, DeleteMemoError, ListMemoError, MemoRepository, UpdateMemoError,
};
use crate::model::memo::{CreateMemo, FindMemo, Memo, UpdateMemo};

use super::Turso;

#[async_trait]
impl MemoRepository for Turso {
    async fn create_memo(&self, creator: CreateMemo) -> Result<Option<Memo>, CreateMemoError> {
        let mut memos: Vec<Memo> = self.query_criteria(creator).await?;
        Ok(memos.pop())
    }

    async fn list_memos(&self, finder: FindMemo) -> Result<Vec<Memo>, ListMemoError> {
        Ok(self.query_criteria(finder).await?)
    }

    async fn delete_memo(&self, memo_id: i32) -> Result<(), DeleteMemoError> {
        self.execute("delete from memo where id = ?", [memo_id])
            .await?;
        Ok(())
    }

    async fn update_memo(&self, updator: UpdateMemo) -> Result<(), UpdateMemoError> {
        self.execute_criteria(updator).await?;
        Ok(())
    }
}
