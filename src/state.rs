use std::sync::Arc;

use anyhow::Result;
use libsql_client::{BatchResult, Client, ResultSet, Statement, Transaction};

#[derive(Debug, Clone)]
pub struct AppState {
    repo: Arc<Client>,
}

impl AppState {
    pub fn new(repo: Client) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }

    pub async fn execute(&self, stmt: impl Into<Statement> + Send) -> Result<ResultSet> {
        self.repo.execute(stmt).await
    }

    pub async fn batch<I: IntoIterator<Item = impl Into<Statement> + Send> + Send>(
        &self,
        stmts: I,
    ) -> Result<Vec<ResultSet>>
    where
        <I as IntoIterator>::IntoIter: Send,
    {
        self.repo.batch(stmts).await
    }

    pub async fn raw_batch(
        &self,
        stmts: impl IntoIterator<Item = impl Into<Statement> + Send> + Send,
    ) -> Result<BatchResult> {
        self.repo.raw_batch(stmts).await
    }

    pub fn batch_sync<I: IntoIterator<Item = impl Into<Statement> + Send> + Send>(
        &self,
        stmts: I,
    ) -> Result<Vec<ResultSet>>
    where
        <I as std::iter::IntoIterator>::IntoIter: std::marker::Send,
    {
        self.repo.batch_sync(stmts)
    }

    pub async fn transaction(&self) -> Result<Transaction> {
        self.repo.transaction().await
    }
}
