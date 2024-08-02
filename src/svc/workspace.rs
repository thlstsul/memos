use std::sync::Arc;

use crate::{
    api::{
        prefix::{ExtractName, FormatName},
        v1::gen::{
            workspace_service_server::{self, WorkspaceServiceServer},
            workspace_setting_service_server::{self, WorkspaceSettingServiceServer},
            GetWorkspaceProfileRequest, GetWorkspaceSettingRequest, SetWorkspaceSettingRequest,
            WorkspaceProfile, WorkspaceSetting,
        },
    },
    dao::{user::UserRepository, workspace::WorkspaceRepository},
    model::gen::{workspace_setting::Value as WorkspaceSettingValue, WorkspaceSettingKey},
};
use async_trait::async_trait;
use tonic::{Request, Response, Status};

use super::Service;

const OWNER: &str = "users/546";
const VERSION: &str = "0.22.3";
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
pub trait WorkspaceSettingService:
    workspace_setting_service_server::WorkspaceSettingService + Clone + Send + Sync + 'static
{
    fn workspace_setting_server(self: Arc<Self>) -> WorkspaceSettingServiceServer<Self> {
        WorkspaceSettingServiceServer::from_arc(self)
    }

    async fn get_upload_size_limit(&self) -> usize;
    async fn is_display_with_update_time(&self) -> bool;
}

#[async_trait]
impl<U: UserRepository> WorkspaceService for Service<U> {}

#[tonic::async_trait]
impl<U: UserRepository> workspace_service_server::WorkspaceService for Service<U> {
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
            password_auth: true,
            ..Default::default()
        }))
    }
}

#[async_trait]
impl<W: WorkspaceRepository> WorkspaceSettingService for Service<W> {
    async fn get_upload_size_limit(&self) -> usize {
        if let Ok(Some(WorkspaceSettingValue::StorageSetting(setting))) = self
            .repo
            .find_workspace_setting(WorkspaceSettingKey::Storage)
            .await
        {
            if setting.upload_size_limit_mb <= 0 {
                DEFAULT_MAX_MIB
            } else {
                setting.upload_size_limit_mb as usize
            }
        } else {
            DEFAULT_MAX_MIB
        }
    }

    async fn is_display_with_update_time(&self) -> bool {
        if let Ok(Some(WorkspaceSettingValue::MemoRelatedSetting(setting))) = self
            .repo
            .find_workspace_setting(WorkspaceSettingKey::MemoRelated)
            .await
        {
            setting.display_with_update_time
        } else {
            false
        }
    }
}

#[tonic::async_trait]
impl<W: WorkspaceRepository> workspace_setting_service_server::WorkspaceSettingService
    for Service<W>
{
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
    async fn set_workspace_setting(
        &self,
        request: Request<SetWorkspaceSettingRequest>,
    ) -> Result<Response<WorkspaceSetting>, Status> {
        todo!()
    }
}
