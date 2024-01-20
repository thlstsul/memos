use snafu::{OptionExt, ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::{
    api::{
        resource::{FindResource, WholeResource},
        v2::{
            resource_service_server, CreateResourceRequest, CreateResourceResponse,
            DeleteResourceRequest, DeleteResourceResponse, ListResourcesRequest,
            ListResourcesResponse, Resource, UpdateResourceRequest, UpdateResourceResponse,
        },
    },
    dao::resource::ResourceDao,
    state::AppState,
};

use super::get_current_user;

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
    ) -> Result<Response<CreateResourceResponse>, Status> {
        todo!()
    }

    async fn list_resources(
        &self,
        request: Request<ListResourcesRequest>,
    ) -> Result<Response<ListResourcesResponse>, Status> {
        let user = get_current_user(&request)?;
        let resources = self
            .dao
            .list_resources(FindResource {
                creator_id: Some(user.id),
                ..Default::default()
            })
            .await
            .context(ListResourceFailed)?;

        Ok(Response::new(ListResourcesResponse { resources }))
    }

    async fn update_resource(
        &self,
        request: Request<UpdateResourceRequest>,
    ) -> Result<Response<UpdateResourceResponse>, Status> {
        todo!()
    }

    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> Result<Response<DeleteResourceResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create resource: {source}"), context(suffix(false)))]
    CreateResourceFailed { source: crate::dao::Error },
    #[snafu(
        display("Maybe create resource failed, because return none"),
        context(suffix(false))
    )]
    MaybeCreateResourceFailed,
    #[snafu(display("Failed to get resource: {source}"), context(suffix(false)))]
    GetResourceFailed { source: crate::dao::Error },
    #[snafu(display("Resource not found: {id}"), context(suffix(false)))]
    ResourceNotFound { id: i32 },
    #[snafu(display("Failed to list resource: {source}"), context(suffix(false)))]
    ListResourceFailed { source: crate::dao::Error },
}
