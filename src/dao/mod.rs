use async_trait::async_trait;
use libsql_client::{de, Statement};
use serde::de::DeserializeOwned;

use crate::state::AppState;

pub mod memo;
pub mod resource;
pub mod system_setting;
pub mod tag;
pub mod user;
pub mod user_setting;

#[async_trait]
pub trait Dao {
    fn get_state(&self) -> &AppState;
    async fn execute<T: DeserializeOwned>(
        &self,
        stmt: impl Into<Statement> + Send,
    ) -> anyhow::Result<Vec<T>> {
        self.get_state()
            .execute(stmt)
            .await?
            .rows
            .iter()
            .map(de::from_row)
            .collect::<Result<Vec<T>, _>>()
    }
}
