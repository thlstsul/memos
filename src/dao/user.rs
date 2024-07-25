use async_trait::async_trait;
use snafu::Snafu;

use crate::model::user::{User, UserSetting};

#[async_trait]
pub trait UserRepository: Clone + Send + Sync + 'static {
    async fn find_user(
        &self,
        name: &str,
        password_hash: Option<&str>,
    ) -> Result<Option<User>, FindUserError>;
    async fn petch_user(&self, id: i32) -> Result<Option<User>, PetchUserError>;
    async fn host_user(&self) -> Result<Option<User>, GetHostUserError>;
    async fn find_user_setting(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserSetting>, FindUserSettingError>;
    async fn upsert_user_setting(
        &self,
        settings: Vec<UserSetting>,
    ) -> Result<(), UpsertUserSettingError>;
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to find user: {source}"))]
pub struct FindUserError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to petch user: {source}"))]
pub struct PetchUserError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to get host user: {source}"))]
pub struct GetHostUserError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to find user setting: {source}"))]
pub struct FindUserSettingError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to upsert user setting: {source}"))]
pub struct UpsertUserSettingError {
    source: anyhow::Error,
}
