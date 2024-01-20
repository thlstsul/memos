use snafu::{OptionExt, ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::{
    api::{
        resource::WholeResource,
        v2::{
            resource_service_server, CreateResourceRequest, CreateResourceResponse,
            DeleteResourceRequest, DeleteResourceResponse, ListResourcesRequest,
            ListResourcesResponse, Resource, UpdateResourceRequest, UpdateResourceResponse,
        },
    },
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
            .context(CreateResourceFailed)?
            .context(MaybeCreateResourceFailed)
    }

    pub async fn get_resource(&self, id: i32) -> Result<WholeResource, Error> {
        self.dao
            .get_resource(id)
            .await
            .context(GetResourceFailed)?
            .context(ResourceNotFound { id })
    }
}

#[tonic::async_trait]
impl resource_service_server::ResourceService for ResourceService {
    async fn create_resource(
        &self,
        request: Request<CreateResourceRequest>,
    ) -> std::result::Result<Response<CreateResourceResponse>, Status> {
        todo!()
    }

    async fn list_resources(
        &self,
        request: Request<ListResourcesRequest>,
    ) -> std::result::Result<Response<ListResourcesResponse>, Status> {
        todo!()
    }

    async fn update_resource(
        &self,
        request: Request<UpdateResourceRequest>,
    ) -> std::result::Result<Response<UpdateResourceResponse>, Status> {
        todo!()
    }

    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> std::result::Result<Response<DeleteResourceResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Create resource failed: {source}"), context(suffix(false)))]
    CreateResourceFailed { source: crate::dao::Error },
    #[snafu(
        display("Maybe create resource failed, because return none"),
        context(suffix(false))
    )]
    MaybeCreateResourceFailed,
    #[snafu(display("Get resource failed: {source}"), context(suffix(false)))]
    GetResourceFailed { source: crate::dao::Error },
    #[snafu(display("Resource not found: {id}"), context(suffix(false)))]
    ResourceNotFound { id: i32 },
}
