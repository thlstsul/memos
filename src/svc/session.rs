use async_trait::async_trait;
use axum_login::tower_sessions::session::{Id, Record};
use axum_login::tower_sessions::{
    self, session_store::Error as StoreError, ExpiredDeletion, MemoryStore,
};
use snafu::{ensure, Snafu};
use tracing::info;

use crate::dao::session::SessionRepository;

#[derive(Debug, Clone)]
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
    async fn delete_expired(&self) -> Result<(), StoreError> {
        self.repo
            .delete_expired_session()
            .await
            .map_err(|e| StoreError::Backend(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl<S: SessionRepository> tower_sessions::SessionStore for SessionStore<S> {
    async fn save(&self, session_record: &Record) -> Result<(), StoreError> {
        let create = session_record
            .try_into()
            .map_err(|e: rmp_serde::encode::Error| StoreError::Encode(e.to_string()))?;
        self.repo
            .create_session(create)
            .await
            .map_err(|e| StoreError::Backend(e.to_string()))?;
        let _ = self.memory.save(session_record).await;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>, StoreError> {
        let session = self.memory.load(session_id).await;
        if let Ok(Some(session)) = session {
            return Ok(Some(session));
        }
        let session = self
            .repo
            .get_session(session_id.to_string())
            .await
            .map_err(|e| StoreError::Backend(e.to_string()))?;

        if let Some(session) = session {
            info!("Got valid session");
            let session = rmp_serde::from_slice(&session.data)
                .map_err(|e| StoreError::Decode(e.to_string()))?;
            let _ = self.memory.save(&session).await;
            return Ok(Some(session));
        }
        Ok(None)
    }

    async fn delete(&self, session_id: &Id) -> Result<(), StoreError> {
        self.repo
            .delete_session(session_id.to_string())
            .await
            .map_err(|e| StoreError::Backend(e.to_string()))?;
        let _ = self.memory.delete(session_id).await;
        Ok(())
    }

    async fn create(&self, session_record: &mut Record) -> Result<(), StoreError> {
        self.save(session_record).await
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
}
