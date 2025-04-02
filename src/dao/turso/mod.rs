pub mod memo;
pub mod resource;
pub mod session;
pub mod user;
pub mod workspace;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use libsql::{de, params::IntoParams, Database, Rows, Statement, Transaction, TransactionBehavior};
use serde::de::DeserializeOwned;

pub trait ToCriteria {
    pub fn to_criteria(&self) -> (impl AsRef<str>, impl IntoParams);
}

#[derive(Debug, Clone)]
pub struct Turso {
    repo: Arc<Database>,
}

impl Turso {
    pub fn new(repo: Database) -> Self {
        Self { repo: repo.into() }
    }

    pub async fn execute(&self, sql: impl AsRef<str>, params: impl IntoParams) -> Result<u64> {
        info!("{}", sql.as_ref());
        let conn = self.repo.connect()?;
        Ok(conn.execute(sql.as_ref(), params).await?)
    }

    pub async fn execute_criteria(&self, criteria: impl ToCriteria) -> Result<u64> {
        let (sql, params) = criteria.to_criteria();
        self.execute(sql, params).await
    }

    #[allow(dead_code)]
    pub async fn execute_batch(&self, sql: impl AsRef<str>) -> Result<()> {
        info!("{}", sql.as_ref());
        let conn = self.repo.connect()?;
        let _ = conn.execute_batch(sql.as_ref()).await?;
        Ok(())
    }

    pub async fn query<T: DeserializeOwned + Send>(
        &self,
        sql: impl AsRef<str>,
        params: impl IntoParams + Send,
    ) -> Result<Vec<T>> {
        info!("{}", sql.as_ref());
        let conn = self.repo.connect()?;
        let rows = conn.query(sql.as_ref(), params).await?;

        de(rows).await
    }

    pub async fn query_criteria<T: DeserializeOwned + Send>(
        &self,
        criteria: impl ToCriteria,
    ) -> Result<Vec<T>> {
        let (sql, params) = criteria.to_criteria();
        self.query(sql, params).await
    }

    pub async fn query_rows(&self, sql: impl AsRef<str>, params: impl IntoParams) -> Result<Rows> {
        info!("{}", sql.as_ref());
        let conn = self.repo.connect()?;
        Ok(conn.query(sql.as_ref(), params).await?)
    }

    pub async fn prepare(&self, sql: impl AsRef<str>) -> Result<Statement> {
        info!("{}", sql.as_ref());
        let conn = self.repo.connect()?;
        Ok(conn.prepare(sql.as_ref()).await?)
    }

    pub async fn transaction(&self) -> Result<Transaction> {
        let conn = self.repo.connect()?;
        Ok(conn.transaction().await?)
    }

    #[allow(dead_code)]
    pub async fn transaction_with_behavior(
        &self,
        tx_behavior: TransactionBehavior,
    ) -> Result<Transaction> {
        let conn = self.repo.connect()?;
        Ok(conn.transaction_with_behavior(tx_behavior).await?)
    }

    pub async fn statement_query(stmt: &mut Statement, params: impl IntoParams) -> Result<Rows> {
        Ok(stmt.query(params).await?)
    }

    pub async fn statement_execute(stmt: &mut Statement, params: impl IntoParams) -> Result<usize> {
        Ok(stmt.execute(params).await?)
    }

    pub async fn tx_prepare(tx: &Transaction, sql: impl AsRef<str>) -> Result<Statement> {
        info!("{}", sql.as_ref());
        Ok(tx.prepare(sql.as_ref()).await?)
    }

    pub async fn commit(tx: Transaction) -> Result<()> {
        Ok(tx.commit().await?)
    }
}

pub async fn de<T: DeserializeOwned>(mut rows: Rows) -> Result<Vec<T>> {
    let mut rtn = Vec::new();
    while let Some(row) = rows.next().await? {
        rtn.push(de::from_row(&row)?);
    }
    Ok(rtn)
}
