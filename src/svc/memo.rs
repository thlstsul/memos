use snafu::{OptionExt, ResultExt, Snafu};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

use crate::{
    api::{
        memo::{CreateMemo, FindMemo, UpdateMemo},
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
    dao::memo::MemoDao,
    state::AppState,
};

use super::{
    get_current_user, resource::ResourceService, system::SystemService, user::UserService,
};

pub struct MemoService {
    memo_dao: MemoDao,
    res_svc: ResourceService,
    user_svc: UserService,
    sys_svc: SystemService,
}

impl MemoService {
    pub fn new(state: &AppState) -> Self {
        Self {
            memo_dao: MemoDao {
                state: state.clone(),
            },
            res_svc: ResourceService::new(state),
            user_svc: UserService::new(state),
            sys_svc: SystemService::new(state),
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
        let memo = self
            .memo_dao
            .create_memo(create)
            .await
            .context(CreateMemoFailed)?
            .context(MaybeCreateMemoFailed)?;
        Ok(Response::new(memo.into()))
    }

    async fn get_memo(
        &self,
        request: Request<GetMemoRequest>,
    ) -> Result<Response<GetMemoResponse>, Status> {
        let mut memos = self
            .memo_dao
            .list_memos(FindMemo {
                id: Some(request.get_ref().id),
                ..Default::default()
            })
            .await
            .context(GetMemoFailed)?;

        let memo = memos.pop();
        Ok(Response::new(memo.into()))
    }

    async fn list_memos(
        &self,
        request: Request<ListMemosRequest>,
    ) -> Result<Response<ListMemosResponse>, Status> {
        let user = get_current_user(&request);
        let mut find: FindMemo = request.get_ref().try_into().context(InvalidMemoFilter)?;
        if let Some(creator) = find.creator.clone() {
            let user = self.user_svc.find_user(creator).await?;
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
        if find.id.is_none() {
            if let Some(SystemSetting { value, .. }) = self
                .sys_svc
                .find_setting(SystemSettingKey::MemoDisplayWithUpdatedTs)
                .await?
            {
                find.order_by_updated_ts = value == "true";
            }
        }

        let mut memos = self
            .memo_dao
            .list_memos(find)
            .await
            .context(ListMemoFailed)?;

        {
            let memo_ids: Vec<i32> = memos.iter().map(|m: &Memo| m.id).collect();
            let relate_resources = self.res_svc.relate_resources(memo_ids).await?;
            for mut memo in memos {
                let value = *relate_resources.get(&memo.id).unwrap_or_default();
                memo.resources = value;
            }
        }
        // TODO relate

        Ok(Response::new(memos.into()))
    }

    /// UpdateMemo updates a memo.
    async fn update_memo(
        &self,
        request: Request<UpdateMemoRequest>,
    ) -> Result<Response<UpdateMemoResponse>, Status> {
        let user = get_current_user(&request)?;
        let update: UpdateMemo = request.get_ref().into();
        let memo_id = update.id;

        let memo = self
            .memo_dao
            .update_memo(user.id, update)
            .await
            .context(UpdateMemoFailed)?;

        let mut memos = self
            .memo_dao
            .list_memos(FindMemo {
                id: Some(memo_id),
                ..Default::default()
            })
            .await
            .context(ListMemoFailed)?;
        Ok(Response::new(memos.pop().into()))
    }

    /// DeleteMemo deletes a memo by id.
    async fn delete_memo(
        &self,
        request: Request<DeleteMemoRequest>,
    ) -> Result<Response<DeleteMemoResponse>, Status> {
        self.memo_dao
            .delete_memo(request.get_ref().id)
            .await
            .context(DeleteMemoFailed)?;
        Ok(Response::new(DeleteMemoResponse {}))
    }

    /// GetUserMemosStats gets stats of memos for a user.
    async fn get_user_memos_stats(
        &self,
        request: Request<GetUserMemosStatsRequest>,
    ) -> Result<Response<GetUserMemosStatsResponse>, Status> {
        let user = get_current_user(&request)?;
        let count = self
            .memo_dao
            .count_memos(user.id)
            .await
            .context(CountMemoFailed)?;
        Ok(Response::new(GetUserMemosStatsResponse {
            // 简化，后面这个api一定会改
            memo_creation_stats: HashMap::from([("2024-01-01".to_owned(), count.count)]),
        }))
    }

    /// SetMemoResources sets resources for a memo.
    async fn set_memo_resources(
        &self,
        request: Request<SetMemoResourcesRequest>,
    ) -> Result<Response<SetMemoResourcesResponse>, Status> {
        let memo_id = request.get_ref().id;
        let resources = request.get_ref().resources;
        let relate_resources = self.res_svc.relate_resource(memo_id).await?;

        let new_res_ids = resources.iter().map(|s: &Resource| *s.id).collect();
        let old_res_ids = relate_resources.iter().map(|s: &Resource| *s.id).collect();

        self.res_svc
            .set_memo_resources(memo_id, new_res_ids, old_res_ids)
            .await?;
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
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create memo: {source}"), context(suffix(false)))]
    CreateMemoFailed { source: crate::dao::Error },
    #[snafu(display("Failed to get memo: {source}"), context(suffix(false)))]
    GetMemoFailed { source: crate::dao::Error },
    #[snafu(
        display("Maybe create memo failed, because return none"),
        context(suffix(false))
    )]
    MaybeCreateMemoFailed,

    #[snafu(display("Failed to update memo: {source}"), context(suffix(false)))]
    UpdateMemoFailed { source: crate::dao::Error },

    #[snafu(display("Failed to delete memo: {source}"), context(suffix(false)))]
    DeleteMemoFailed { source: crate::dao::Error },

    #[snafu(display("Failed to find memo list: {source}"), context(suffix(false)))]
    ListMemoFailed { source: crate::dao::Error },

    #[snafu(display("Failed to count memo: {source}"), context(suffix(false)))]
    CountMemoFailed { source: crate::dao::Error },

    #[snafu(display("Invalid memo filter: {source}"), context(suffix(false)))]
    InvalidMemoFilter { source: crate::api::memo::Error },
}
