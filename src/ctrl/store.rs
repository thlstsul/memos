use async_trait::async_trait;
use axum_login::tower_sessions::session::Id;
use axum_login::tower_sessions::{ExpiredDeletion, MemoryStore, Session, SessionStore};
use libsql::{params, Value};
use snafu::{ensure, ResultExt, Snafu};
use time::OffsetDateTime;
use tracing::info;

use crate::state::AppState;

#[derive(Clone)]
pub struct TursoStore {
    state: AppState,
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
    pub fn new(state: &AppState) -> Self {
        Self {
            state: state.clone(),
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
        let sql = format!(
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
        self.state.execute(&sql, ()).await.context(Execute)?;
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for TursoStore {
    async fn delete_expired(&self) -> Result<(), Error> {
        let sql = format!(
            r#"
            delete from {table_name}
            where expiry_date < datetime('now', 'utc')
            "#,
            table_name = self.table_name
        );
        self.state.execute(&sql, ()).await.context(Execute)?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for TursoStore {
    type Error = Error;

    async fn save(&self, session: &Session) -> Result<(), Self::Error> {
        let sql = format!(
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
        self.state
            .execute(
                &sql,
                params![
                    session.id().to_string(),
                    data,
                    session.expiry_date().unix_timestamp()
                ],
            )
            .await
            .context(Execute)?;

        let _ = self.memory.save(session).await;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Session>, Self::Error> {
        let session = self.memory.load(session_id).await;
        if let Ok(Some(session)) = session {
            return Ok(Some(session));
        }
        let sql = format!(
            r#"
            select data from {}
            where id = ? and expiry_date > ?
            "#,
            self.table_name
        );

        let mut rows = self
            .state
            .query(
                &sql,
                params![
                    session_id.to_string(),
                    OffsetDateTime::now_utc().unix_timestamp()
                ],
            )
            .await
            .context(Execute)?;

        if let Ok(Some(row)) = rows.next().await {
            if let Ok(Value::Blob(value)) = row.get_value(0) {
                info!("Got valid session");
                let session = rmp_serde::from_slice(&value).context(DecodeSession)?;
                let _ = self.memory.save(&session).await;
                return Ok(Some(session));
            }
        }
        Ok(None)
    }

    async fn delete(&self, session_id: &Id) -> Result<(), Self::Error> {
        let sql = format!(
            r#"
            delete from {} where id = ?
            "#,
            self.table_name
        );
        self.state
            .execute(&sql, [session_id.to_string()])
            .await
            .context(Execute)?;

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
    Execute { source: libsql::Error },
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
