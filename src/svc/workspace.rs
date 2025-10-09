use std::sync::Arc;

use crate::{
    api::{
        prefix::{ExtractName, FormatName},
        v1::generate::{
            GetWorkspaceProfileRequest, GetWorkspaceSettingRequest, UpdateWorkspaceSettingRequest,
            WorkspaceProfile, WorkspaceSetting,
            workspace_service_server::{self, WorkspaceServiceServer},
        },
    },
    dao::{user::UserRepository, workspace::WorkspaceRepository},
    model::generate::WorkspaceSettingKey,
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

use super::Service;

const OWNER: &str = "users/546";
const VERSION: &str = "0.25.1";
const MODE: &str = "prod";
const DEFAULT_MAX_MIB: usize = 32;

#[async_trait]
pub trait WorkspaceService:
    workspace_service_server::WorkspaceService + Clone + Send + Sync + 'static
{
    fn workspace_server(self: Arc<Self>) -> WorkspaceServiceServer<Self> {
        WorkspaceServiceServer::from_arc(self)
    }
}

#[async_trait]
impl<U: UserRepository + WorkspaceRepository> WorkspaceService for Service<U> {}

#[tonic::async_trait]
impl<U: UserRepository + WorkspaceRepository> workspace_service_server::WorkspaceService
    for Service<U>
{
    /// GetWorkspaceProfile returns the workspace profile.
    async fn get_workspace_profile(
        &self,
        request: Request<GetWorkspaceProfileRequest>,
    ) -> Result<Response<WorkspaceProfile>, Status> {
        let host = self.repo.host_user().await?;
        let owner = host.map(|h| h.get_name()).unwrap_or(OWNER.to_owned());
        Ok(Response::new(WorkspaceProfile {
            owner,
            version: VERSION.to_owned(),
            mode: MODE.to_owned(),
            ..Default::default()
        }))
    }

    /// GetWorkspaceSetting returns the setting by name.
    async fn get_workspace_setting(
        &self,
        request: Request<GetWorkspaceSettingRequest>,
    ) -> Result<Response<WorkspaceSetting>, Status> {
        let key = request.get_ref().get_name();
        let key =
            WorkspaceSettingKey::from_str_name(&key).unwrap_or(WorkspaceSettingKey::Unspecified);

        let name = request.into_inner().name;
        let value = self.repo.find_workspace_setting(key).await?;
        let value = value.map(|v| v.into());
        Ok(Response::new(WorkspaceSetting { name, value }))
    }

    /// SetWorkspaceSetting updates the setting.
    async fn update_workspace_setting(
        &self,
        request: Request<UpdateWorkspaceSettingRequest>,
    ) -> Result<Response<WorkspaceSetting>, Status> {
        todo!()
    }
}
