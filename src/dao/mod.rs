use async_trait::async_trait;
use libsql::{de, params::IntoParams, Rows};
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

    async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, Error>
    where
        P: IntoParams + Send,
    {
        self.get_state().execute(sql, params).await.context(Execute)
    }

    async fn query<T: DeserializeOwned + Send, P>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, Error>
    where
        P: IntoParams + Send,
    {
        let rows = self.get_state().query(sql, params).await.context(Execute)?;

        de(rows).await
    }
}

pub async fn de<T: DeserializeOwned>(mut rows: Rows) -> Result<Vec<T>, Error> {
    let mut rtn = Vec::new();
    while let Some(row) = rows.next().await.context(GetNextRow)? {
        rtn.push(de::from_row(&row).context(Deserialize)?);
    }
    Ok(rtn)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Execute { source: libsql::Error },
    #[snafu(display("Deserialize failed: {source}"), context(suffix(false)))]
    Deserialize { source: serde::de::value::Error },
    #[snafu(display("Failed to get next row: {source}"), context(suffix(false)))]
    GetNextRow { source: libsql::Error },
}
