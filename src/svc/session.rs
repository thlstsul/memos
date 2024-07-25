use async_trait::async_trait;
use axum_login::tower_sessions::session::Id;
use axum_login::tower_sessions::{self, ExpiredDeletion, MemoryStore};
use snafu::{ensure, ResultExt, Snafu};
use tracing::info;

use crate::dao::session::SessionRepository;

#[derive(Clone)]
pub struct SessionStore<S: SessionRepository> {
    repo: S,
    memory: MemoryStore,
    table_name: String,
}

impl<S: SessionRepository> SessionStore<S> {
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
    pub fn new(state: S) -> Self {
        Self {
            repo: state,
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
                table_name: table_name.to_string()
            }
        );

        self.table_name = table_name.to_string();
        Ok(self)
    }

    /// Migrate the session schema.
    #[allow(dead_code)]
    pub async fn migrate(&self) -> Result<(), Error> {
        self.repo.migrate_session_table().await?;
        Ok(())
    }
}

#[async_trait]
impl<S: SessionRepository> ExpiredDeletion for SessionStore<S> {
    async fn delete_expired(&self) -> Result<(), Error> {
        self.repo.delete_expired_session().await?;
        Ok(())
    }
}

#[async_trait]
impl<S: SessionRepository> tower_sessions::SessionStore for SessionStore<S> {
    type Error = Error;

    async fn save(&self, session: &tower_sessions::Session) -> Result<(), Self::Error> {
        let create = session.try_into().context(EncodeSession)?;
        self.repo.create_session(create).await?;
        let _ = self.memory.save(session).await;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<tower_sessions::Session>, Self::Error> {
        let session = self.memory.load(session_id).await;
        if let Ok(Some(session)) = session {
            return Ok(Some(session));
        }
        let session = self.repo.get_session(session_id.to_string()).await?;

        if let Some(session) = session {
            info!("Got valid session");
            let session = rmp_serde::from_slice(&session.data).context(DecodeSession)?;
            let _ = self.memory.save(&session).await;
            return Ok(Some(session));
        }
        Ok(None)
    }

    async fn delete(&self, session_id: &Id) -> Result<(), Self::Error> {
        self.repo.delete_session(session_id.to_string()).await?;
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
    #[snafu(context(false))]
    CreateSession {
        source: crate::dao::session::CreateSessionError,
    },
    #[snafu(context(false))]
    GetSession {
        source: crate::dao::session::GetSessionError,
    },
    #[snafu(context(false))]
    DeleteSession {
        source: crate::dao::session::DeleteSessionError,
    },
    #[snafu(context(false))]
    DeleteExpiredSession {
        source: crate::dao::session::DeleteExpiredSessionError,
    },
    #[snafu(context(false))]
    MigrateSessionTable {
        source: crate::dao::session::MigrateSessionTableError,
    },

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
}
