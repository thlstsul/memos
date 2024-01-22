use std::collections::VecDeque;

use async_trait::async_trait;
use libsql_client::{de, ResultSet, Statement};
use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};

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

    async fn execute(&self, stmt: impl Into<Statement> + Send) -> Result<ResultSet, Error> {
        self.get_state().execute(stmt).await.context(ExecuteFailed)
    }

    async fn query<T: DeserializeOwned>(
        &self,
        stmt: impl Into<Statement> + Send,
    ) -> Result<Vec<T>, Error> {
        self.get_state()
            .execute(stmt)
            .await
            .context(ExecuteFailed)?
            .rows
            .iter()
            .map(|row| de::from_row(row).context(DeserializeFailed))
            .collect::<Result<Vec<T>, _>>()
    }

    async fn batch<I>(&self, stmts: I) -> Result<Vec<ResultSet>, Error>
    where
        I: IntoIterator + Send,
        I::Item: Into<Statement> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        self.get_state().batch(stmts).await.context(ExecuteFailed)
    }

    async fn batch_query<I, T: DeserializeOwned>(&self, stmts: I) -> Result<VecDeque<Vec<T>>, Error>
    where
        I: IntoIterator + Send,
        I::Item: Into<Statement> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        self.get_state()
            .batch(stmts)
            .await
            .context(ExecuteFailed)?
            .iter()
            .map(|rs: &ResultSet| {
                rs.rows
                    .iter()
                    .map(|row| de::from_row(row).context(DeserializeFailed))
                    .collect::<Result<Vec<T>, _>>()
            })
            .collect()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    ExecuteFailed { source: anyhow::Error },
    #[snafu(display("Deserialize failed: {source}"), context(suffix(false)))]
    DeserializeFailed { source: anyhow::Error },
    #[snafu(display("Data does not exsit"), context(suffix(false)))]
    Inexistent,
}
