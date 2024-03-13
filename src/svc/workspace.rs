use tonic::{Request, Response, Status};

use crate::{
    api::v2::{
        workspace_service_server::{self, WorkspaceServiceServer},
        workspace_setting,
        workspace_setting_service_server::{self, WorkspaceSettingServiceServer},
        GetWorkspaceProfileRequest, GetWorkspaceProfileResponse, GetWorkspaceSettingRequest,
        GetWorkspaceSettingResponse, SetWorkspaceSettingRequest, SetWorkspaceSettingResponse,
        WorkspaceGeneralSetting, WorkspaceProfile, WorkspaceSetting,
    },
    dao::system_setting::SystemSettingDao,
    state::AppState,
};

const VERSION: &str = "0.21.1";
const MODE: &str = "prod";

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn server() -> WorkspaceServiceServer<WorkspaceService> {
        WorkspaceServiceServer::new(WorkspaceService)
    }
}

#[tonic::async_trait]
impl workspace_service_server::WorkspaceService for WorkspaceService {
    /// GetWorkspaceProfile returns the workspace profile.
    async fn get_workspace_profile(
        &self,
        request: Request<GetWorkspaceProfileRequest>,
    ) -> Result<Response<GetWorkspaceProfileResponse>, Status> {
        Ok(Response::new(GetWorkspaceProfileResponse {
            workspace_profile: Some(WorkspaceProfile {
                version: VERSION.to_owned(),
                mode: MODE.to_owned(),
                ..Default::default()
            }),
        }))
    }
}

pub struct WorkspaceSettingService {
    #[allow(dead_code)]
    dao: SystemSettingDao,
}

impl WorkspaceSettingService {
    pub fn new(state: &AppState) -> Self {
        Self {
            dao: SystemSettingDao {
                state: state.clone(),
            },
        }
    }

    pub fn server(state: &AppState) -> WorkspaceSettingServiceServer<WorkspaceSettingService> {
        WorkspaceSettingServiceServer::new(WorkspaceSettingService::new(state))
    }
}

#[tonic::async_trait]
impl workspace_setting_service_server::WorkspaceSettingService for WorkspaceSettingService {
    /// GetWorkspaceSetting returns the setting by name.
    async fn get_workspace_setting(
        &self,
        request: Request<GetWorkspaceSettingRequest>,
    ) -> Result<Response<GetWorkspaceSettingResponse>, Status> {
        // TODO
        Ok(Response::new(GetWorkspaceSettingResponse {
            setting: Some(WorkspaceSetting {
                name: request.into_inner().name,
                value: Some(workspace_setting::Value::GeneralSetting(
                    WorkspaceGeneralSetting::default(),
                )),
            }),
        }))
    }
    /// SetWorkspaceSetting updates the setting.
    async fn set_workspace_setting(
        &self,
        request: Request<SetWorkspaceSettingRequest>,
    ) -> Result<Response<SetWorkspaceSettingResponse>, Status> {
        todo!()
    }
}
