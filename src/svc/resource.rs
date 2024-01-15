use libsql_client::Client;
use snafu::{ResultExt, Snafu};
use std::sync::Arc;

use crate::{
    api::{resource::CreateResource, v2::Resource},
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

    pub async fn create_resource(&self, create: CreateResource) -> Result<Resource, Error> {
        self.dao
            .create_resource(create)
            .await
            .context(CreateResourceFailed)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Create resource failed: {source}"), context(suffix(false)))]
    CreateResourceFailed { source: crate::dao::resource::Error },
}
