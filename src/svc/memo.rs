use snafu::{ResultExt, Snafu};
use std::sync::Arc;

use libsql_client::Client;

use crate::{
    api::memo::{FindMemo, Memo},
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

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to find memo list"), context(suffix(false)))]
    ListMemoFailed { source: crate::dao::memo::Error },
}
