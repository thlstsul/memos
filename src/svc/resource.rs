use snafu::{ResultExt, Snafu};

use crate::{
    api::{resource::WholeResource, v2::Resource},
    dao::resource::ResourceDao,
    state::AppState,
};

pub struct ResourceService {
    dao: ResourceDao,
}

impl ResourceService {
    pub fn new(state: &AppState) -> Self {
        Self {
            dao: ResourceDao {
                state: state.clone(),
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
