use std::sync::Arc;

use async_trait::async_trait;
use axum_login::tower_sessions::session::Id;
use axum_login::tower_sessions::{ExpiredDeletion, MemoryStore, Session, SessionStore};
use libsql_client::{Client, Statement, Value};
use snafu::{ensure, ResultExt, Snafu};
use time::OffsetDateTime;
use tracing::info;

#[derive(Debug, Clone)]
pub struct TursoStore {
    client: Arc<Client>,
    memory: MemoryStore,
    table_name: String,
}

impl TursoStore {
    /// Create a new SQLite store with the provided connection pool.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tower_sessions::{sqlx::SqlitePool, SqliteStore};
    ///
    /// # tokio_test::block_on(async {
    /// let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    /// let session_store = SqliteStore::new(pool);
    /// # })
    /// ```
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            client: Arc::clone(client),
            memory: MemoryStore::default(),
            table_name: "sessions".into(),
        }
    }

    /// Set the session table name with the provided name.
    #[allow(dead_code)]
    pub fn with_table_name(mut self, table_name: impl AsRef<str>) -> Result<Self, Error> {
        let table_name = table_name.as_ref();
        ensure!(
            is_valid_table_name(table_name),
            InvalidTable {
                table_name: table_name.to_owned()
            }
        );

        self.table_name = table_name.to_owned();
        Ok(self)
    }

    /// Migrate the session schema.
    #[allow(dead_code)]
    pub async fn migrate(&self) -> Result<(), Error> {
        let query = format!(
            r#"
            create table if not exists {}
            (
                id text primary key not null,
                data blob not null,
                expiry_date integer not null
            )
            "#,
            self.table_name
        );
        self.client.execute(query).await.context(Database)?;
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for TursoStore {
    async fn delete_expired(&self) -> Result<(), Error> {
        let query = format!(
            r#"
            delete from {table_name}
            where expiry_date < datetime('now', 'utc')
            "#,
            table_name = self.table_name
        );
        self.client.execute(query).await.context(Database)?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for TursoStore {
    type Error = Error;

    async fn save(&self, session: &Session) -> Result<(), Self::Error> {
        let query = format!(
            r#"
            insert into {}
              (id, data, expiry_date) values (?, ?, ?)
            on conflict(id) do update set
              data = excluded.data,
              expiry_date = excluded.expiry_date
            "#,
            self.table_name
        );

        let data = rmp_serde::to_vec(session).context(EncodeSession)?;
        let stmt = Statement::with_args(
            query,
            &[
                Value::from(session.id().to_string()),
                Value::from(data),
                Value::from(session.expiry_date().unix_timestamp()),
            ],
        );
        self.client.execute(stmt).await.context(Database)?;

        let _ = self.memory.save(session).await;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Session>, Self::Error> {
        let session = self.memory.load(session_id).await;
        if let Ok(Some(session)) = session {
            return Ok(Some(session));
        }
        let query = format!(
            r#"
            select data from {}
            where id = ? and expiry_date > ?
            "#,
            self.table_name
        );
        let stmt = Statement::with_args(
            &query,
            &[
                Value::from(session_id.to_string()),
                Value::from(OffsetDateTime::now_utc().unix_timestamp()),
            ],
        );
        let data = self.client.execute(stmt).await.context(Database)?;

        if let Some(row) = data.rows.first() {
            if let Some(Value::Blob { value }) = row.values.first() {
                info!("Got valid session");
                return Ok(Some(rmp_serde::from_slice(value).context(DecodeSession)?));
            }
        }
        Ok(None)
    }

    async fn delete(&self, session_id: &Id) -> Result<(), Self::Error> {
        let query = format!(
            r#"
            delete from {} where id = ?
            "#,
            self.table_name
        );
        let stmt = Statement::with_args(&query, &[session_id.to_string()]);
        self.client.execute(stmt).await.context(Database)?;

        let _ = self.memory.delete(session_id).await;
        Ok(())
    }
}

#[allow(dead_code)]
fn is_valid_table_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Execute failed: {source}"), context(suffix(false)))]
    Database { source: anyhow::Error },
    #[snafu(
        display(
            "Invalid table name '{table_name}'. Table names must be alphanumeric and may contain \
                 hyphens or underscores."
        ),
        context(suffix(false))
    )]
    InvalidTable { table_name: String },
    #[snafu(display("Encode session failed: {source}"), context(suffix(false)))]
    EncodeSession { source: rmp_serde::encode::Error },
    #[snafu(display("Decode session failed: {source}"), context(suffix(false)))]
    DecodeSession { source: rmp_serde::decode::Error },
    #[snafu(
        display("Converts a vector of bytes to a String failed: {source}"),
        context(suffix(false))
    )]
    ConvertToString { source: std::string::FromUtf8Error },
}
