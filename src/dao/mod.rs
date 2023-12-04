use std::sync::Arc;

use async_trait::async_trait;
use libsql_client::{de, Client, Statement};
use serde::de::DeserializeOwned;

pub mod system_setting;
pub mod user;
pub mod user_setting;

#[async_trait]
pub trait Dao {
    fn get_client(&self) -> Arc<Client>;
    async fn execute<T: DeserializeOwned>(
        &self,
        stmt: impl Into<Statement> + Send,
    ) -> anyhow::Result<Vec<T>> {
        self.get_client()
            .execute(stmt)
            .await?
            .rows
            .iter()
            .map(de::from_row)
            .collect::<Result<Vec<T>, _>>()
    }
}
