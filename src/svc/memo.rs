use snafu::Snafu;
use std::{collections::HashMap, sync::Arc};
use tonic::{Request, Response, Status};
use tracing::info;

use libsql_client::Client;

use crate::{
    api::{
        memo::{CreateMemo, FindMemo},
        system::{SystemSetting, SystemSettingKey},
        v2::{
            memo_service_server, CreateMemoCommentRequest, CreateMemoCommentResponse,
            CreateMemoRequest, CreateMemoResponse, DeleteMemoRequest, DeleteMemoResponse,
            GetMemoRequest, GetMemoResponse, GetUserMemosStatsRequest, GetUserMemosStatsResponse,
            ListMemoCommentsRequest, ListMemoCommentsResponse, ListMemoRelationsRequest,
            ListMemoRelationsResponse, ListMemoResourcesRequest, ListMemoResourcesResponse,
            ListMemosRequest, ListMemosResponse, SetMemoRelationsRequest, SetMemoRelationsResponse,
            SetMemoResourcesRequest, SetMemoResourcesResponse, UpdateMemoRequest,
            UpdateMemoResponse, Visibility,
        },
    },
    dao::{memo::MemoDao, system_setting::SystemSettingDao, user::UserDao},
};

use super::get_current_user;

pub struct MemoService {
    memo_dao: MemoDao,
    user_dao: UserDao,
    sys_dao: SystemSettingDao,
}

impl MemoService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            memo_dao: MemoDao {
                client: Arc::clone(client),
            },
            user_dao: UserDao {
                client: Arc::clone(client),
            },
            sys_dao: SystemSettingDao {
                client: Arc::clone(client),
            },
        }
    }
}

#[tonic::async_trait]
impl memo_service_server::MemoService for MemoService {
    async fn create_memo(
        &self,
        request: Request<CreateMemoRequest>,
    ) -> Result<Response<CreateMemoResponse>, Status> {
        let user = get_current_user(&request)?;
        let req = request.get_ref();
        let create = CreateMemo {
            creator_id: user.id,
            content: req.content.clone(),
            visibility: req.visibility(),
        };
        let memo = self.memo_dao.create_memo(create).await?;
        Ok(Response::new(memo.into()))
    }

    async fn list_memos(
        &self,
        request: Request<ListMemosRequest>,
    ) -> Result<Response<ListMemosResponse>, Status> {
        let user = get_current_user(&request);
        let mut find: FindMemo = request.get_ref().try_into()?;
        if let Some(creator) = find.creator.clone() {
            let user = self.user_dao.find_user(creator, None).await?;
            find.creator_id = Some(user.id);
        }
        if let Ok(user) = user {
            if find.creator_id.is_some() {
                if Some(user.id) != find.creator_id {
                    find.visibility_list = vec![Visibility::Public, Visibility::Protected];
                }
            } else {
                find.creator_id = Some(user.id);
            }
        } else {
            find.visibility_list = vec![Visibility::Public];
        }
        if let Some(SystemSetting { value, .. }) = self
            .sys_dao
            .find_setting(SystemSettingKey::MemoDisplayWithUpdatedTs)
            .await?
        {
            find.order_by_updated_ts = value == "true";
        }
        let memos = self.memo_dao.list_memos(find).await?;
        // TODO relate,resource

        info!("{memos:?}");
        Ok(Response::new(memos.into()))
    }

    async fn get_memo(
        &self,
        request: Request<GetMemoRequest>,
    ) -> Result<Response<GetMemoResponse>, Status> {
        todo!()
    }
    /// UpdateMemo updates a memo.
    async fn update_memo(
        &self,
        request: Request<UpdateMemoRequest>,
    ) -> Result<Response<UpdateMemoResponse>, Status> {
        todo!()
    }
    /// DeleteMemo deletes a memo by id.
    async fn delete_memo(
        &self,
        request: Request<DeleteMemoRequest>,
    ) -> Result<Response<DeleteMemoResponse>, Status> {
        self.memo_dao.delete_memo(request.get_ref().id).await?;
        Ok(Response::new(DeleteMemoResponse {}))
    }
    /// SetMemoResources sets resources for a memo.
    async fn set_memo_resources(
        &self,
        request: Request<SetMemoResourcesRequest>,
    ) -> Result<Response<SetMemoResourcesResponse>, Status> {
        // TODO
        Ok(Response::new(SetMemoResourcesResponse {}))
    }
    /// ListMemoResources lists resources for a memo.
    async fn list_memo_resources(
        &self,
        request: Request<ListMemoResourcesRequest>,
    ) -> Result<Response<ListMemoResourcesResponse>, Status> {
        todo!()
    }
    /// SetMemoRelations sets relations for a memo.
    async fn set_memo_relations(
        &self,
        request: Request<SetMemoRelationsRequest>,
    ) -> Result<Response<SetMemoRelationsResponse>, Status> {
        // TODO
        Ok(Response::new(SetMemoRelationsResponse {}))
    }
    /// ListMemoRelations lists relations for a memo.
    async fn list_memo_relations(
        &self,
        request: Request<ListMemoRelationsRequest>,
    ) -> Result<Response<ListMemoRelationsResponse>, Status> {
        todo!()
    }
    /// CreateMemoComment creates a comment for a memo.
    async fn create_memo_comment(
        &self,
        request: Request<CreateMemoCommentRequest>,
    ) -> Result<Response<CreateMemoCommentResponse>, Status> {
        todo!()
    }
    /// ListMemoComments lists comments for a memo.
    async fn list_memo_comments(
        &self,
        request: Request<ListMemoCommentsRequest>,
    ) -> Result<Response<ListMemoCommentsResponse>, Status> {
        todo!()
    }
    /// GetUserMemosStats gets stats of memos for a user.
    async fn get_user_memos_stats(
        &self,
        request: Request<GetUserMemosStatsRequest>,
    ) -> Result<Response<GetUserMemosStatsResponse>, Status> {
        let user = get_current_user(&request)?;
        let count = self.memo_dao.count_memos(user.id).await?;
        Ok(Response::new(GetUserMemosStatsResponse {
            // 简化，后面这个api一定会改
            memo_creation_stats: HashMap::from([("2024-01-01".to_owned(), count.count)]),
        }))
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to find memo list: {source}"), context(suffix(false)))]
    ListMemoFailed { source: crate::dao::memo::Error },
}
