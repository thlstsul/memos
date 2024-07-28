use crate::model::session::Session;
use async_trait::async_trait;
use snafu::Snafu;
use std::fmt::Debug;

#[async_trait]
pub trait SessionRepository: Debug + Clone + Send + Sync + 'static {
    async fn create_session(&self, session: Session) -> Result<(), CreateSessionError>;
    async fn get_session(&self, session_id: String) -> Result<Option<Session>, GetSessionError>;
    async fn delete_session(&self, session_id: String) -> Result<(), DeleteSessionError>;
    async fn delete_expired_session(&self) -> Result<(), DeleteExpiredSessionError>;
    async fn migrate_session_table(&self) -> Result<(), MigrateSessionTableError>;
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to save session: {source}"))]
pub struct CreateSessionError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to get session: {source}"))]
pub struct GetSessionError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to delete session: {source}"))]
pub struct DeleteSessionError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to delete expired session: {source}"))]
pub struct DeleteExpiredSessionError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to migrate session table: {source}"))]
pub struct MigrateSessionTableError {
    source: anyhow::Error,
}
