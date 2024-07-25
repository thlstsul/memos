use std::collections::HashMap;

use async_trait::async_trait;
use snafu::Snafu;

use crate::model::resource::{FindResource, Resource};

#[async_trait]
pub trait ResourceRepository: Clone + Send + Sync + 'static {
    async fn create_resource(
        &self,
        resource: Resource,
    ) -> Result<Option<Resource>, CreateResourceError>;
    async fn set_resources_memo(
        &self,
        memo_id: i32,
        add_res_ids: Vec<i32>,
        del_res_ids: Vec<i32>,
    ) -> Result<(), SetResourceError>;
    async fn get_resource(&self, id: i32) -> Result<Option<Resource>, GetResourceError>;
    async fn list_resources(&self, find: FindResource) -> Result<Vec<Resource>, ListResourceError>;
    async fn delete_resource(&self, id: i32, creator_id: i32) -> Result<(), DeleteResourceError>;
    async fn relate_resources(
        &self,
        memo_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<Resource>>, RelateResourceError>;
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to create resource: {source}"))]
pub struct CreateResourceError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to set resource: {source}"))]
pub struct SetResourceError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to get resource: {source}"))]
pub struct GetResourceError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to list resource: {source}"))]
pub struct ListResourceError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to delete resource: {source}"))]
pub struct DeleteResourceError {
    source: anyhow::Error,
}

#[derive(Debug, Snafu)]
#[snafu(context(false), display("Failed to relate resource: {source}"))]
pub struct RelateResourceError {
    source: anyhow::Error,
}
