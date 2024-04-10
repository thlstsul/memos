use std::sync::Arc;

use libsql::{
    params::IntoParams, Connection, Error, Rows, Statement, Transaction, TransactionBehavior,
};

#[derive(Clone)]
pub struct AppState {
    repo: Arc<Connection>,
}

impl AppState {
    pub fn new(repo: Connection) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }

    pub async fn execute(&self, sql: &str, params: impl IntoParams) -> Result<u64, Error> {
        self.repo.execute(sql, params).await
    }

    #[allow(dead_code)]
    pub async fn execute_batch(&self, sql: &str) -> Result<(), Error> {
        self.repo.execute_batch(sql).await
    }

    pub async fn query(&self, sql: &str, params: impl IntoParams) -> Result<Rows, Error> {
        self.repo.query(sql, params).await
    }

    pub async fn prepare(&self, sql: &str) -> Result<Statement, Error> {
        self.repo.prepare(sql).await
    }

    #[allow(dead_code)]
    pub async fn transaction(&self) -> Result<Transaction, Error> {
        self.repo.transaction().await
    }

    #[allow(dead_code)]
    pub async fn transaction_with_behavior(
        &self,
        tx_behavior: TransactionBehavior,
    ) -> Result<Transaction, Error> {
        self.repo.transaction_with_behavior(tx_behavior).await
    }
}
