use libsql_client::Client;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;

use crate::{
    api::{resource::WholeResource, v2::Resource},
    dao::resource::ResourceDao,
};

pub struct ResourceService {
    dao: ResourceDao,
}

impl ResourceService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            dao: ResourceDao {
                client: Arc::clone(client),
            },
        }
    }

    pub async fn create_resource(&self, create: WholeResource) -> Result<Resource, Error> {
        self.dao
            .create_resource(create)
            .await
            .context(CreateResourceFailed)
    }

    pub async fn get_resource(&self, id: i32) -> Result<WholeResource, Error> {
        self.dao
            .get_resource(id)
            .await
            .context(ResourceNotFound { id })
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Create resource failed: {source}"), context(suffix(false)))]
    CreateResourceFailed { source: crate::dao::resource::Error },
    #[snafu(display("Resource not found: {id} : {source}"), context(suffix(false)))]
    ResourceNotFound {
        id: i32,
        source: crate::dao::resource::Error,
    },
}
