use std::sync::Arc;

use libsql::{
    params::IntoParams, Database, Error, Rows, Statement, Transaction, TransactionBehavior,
};

#[derive(Clone)]
pub struct AppState {
    repo: Arc<Database>,
}

impl AppState {
    pub fn new(repo: Database) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }

    pub async fn execute(&self, sql: &str, params: impl IntoParams) -> Result<u64, Error> {
        let conn = self.repo.connect()?;
        conn.execute(sql, params).await
    }

    #[allow(dead_code)]
    pub async fn execute_batch(&self, sql: &str) -> Result<(), Error> {
        let conn = self.repo.connect()?;
        conn.execute_batch(sql).await
    }

    pub async fn query(&self, sql: &str, params: impl IntoParams) -> Result<Rows, Error> {
        let conn = self.repo.connect()?;
        conn.query(sql, params).await
    }

    pub async fn prepare(&self, sql: &str) -> Result<Statement, Error> {
        let conn = self.repo.connect()?;
        conn.prepare(sql).await
    }

    #[allow(dead_code)]
    pub async fn transaction(&self) -> Result<Transaction, Error> {
        let conn = self.repo.connect()?;
        conn.transaction().await
    }

    #[allow(dead_code)]
    pub async fn transaction_with_behavior(
        &self,
        tx_behavior: TransactionBehavior,
    ) -> Result<Transaction, Error> {
        let conn = self.repo.connect()?;
        conn.transaction_with_behavior(tx_behavior).await
    }
}
