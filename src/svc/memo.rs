use crate::api::prefix::{ExtractName, FormatName};
use crate::api::v1::gen::{GetMemoByUidRequest, MemoPropertyEntity, RowStatus};
use crate::dao::resource::ResourceRepository;
use crate::util::ast;
use crate::{
    api::v1::gen::{
        memo_service_server::{self, MemoServiceServer},
        CreateMemoCommentRequest, CreateMemoRequest, DeleteMemoReactionRequest, DeleteMemoRequest,
        DeleteMemoTagRequest, ExportMemosRequest, ExportMemosResponse, GetMemoRequest,
        ListMemoCommentsRequest, ListMemoCommentsResponse, ListMemoPropertiesRequest,
        ListMemoPropertiesResponse, ListMemoReactionsRequest, ListMemoReactionsResponse,
        ListMemoRelationsRequest, ListMemoRelationsResponse, ListMemoResourcesRequest,
        ListMemoResourcesResponse, ListMemoTagsRequest, ListMemoTagsResponse, ListMemosRequest,
        ListMemosResponse, Memo, Reaction, RebuildMemoPropertyRequest, RenameMemoTagRequest,
        SetMemoRelationsRequest, SetMemoResourcesRequest, UpdateMemoRequest,
        UpsertMemoReactionRequest,
    },
    dao::{memo::MemoRepository, user::UserRepository, workspace::WorkspaceRepository},
    model::{
        memo::{CreateMemo, FindMemo, UpdateMemo},
        pager::Paginator,
    },
    util,
};
use async_trait::async_trait;
use prost_types::Timestamp;
use snafu::{OptionExt, ResultExt, Snafu};
use std::{collections::HashMap, sync::Arc};
use tonic::{Request, Response, Status};

use super::resource::ResourceService;
use super::workspace::WorkspaceSettingService;
use super::{RequestExt, Service};

#[async_trait]
pub trait MemoService: memo_service_server::MemoService + Clone + Send + Sync + 'static {
    fn memo_server(self: Arc<Self>) -> MemoServiceServer<Self> {
        MemoServiceServer::from_arc(self)
    }
}

#[async_trait]
impl<T: MemoRepository + UserRepository + ResourceRepository + WorkspaceRepository> MemoService
    for Service<T>
{
}

#[tonic::async_trait]
impl<T: MemoRepository + UserRepository + ResourceRepository + WorkspaceRepository>
    memo_service_server::MemoService for Service<T>
{
    async fn create_memo(
        &self,
        request: Request<CreateMemoRequest>,
    ) -> Result<Response<Memo>, Status> {
        let user = request.get_current_user()?;
        let req = request.get_ref();
        let payload = ast::get_memo_property(&req.content);
        let create = CreateMemo {
            creator_id: user.id,
            uid: util::uuid(),
            content: req.content.clone(),
            visibility: req.visibility(),
            payload,
        };

        let memo: Memo = self
            .repo
            .create_memo(create)
            .await?
            .context(MaybeCreateMemo)?
            .into();

        Ok(Response::new(memo))
    }

    async fn get_memo(&self, request: Request<GetMemoRequest>) -> Result<Response<Memo>, Status> {
        let id = request.get_ref().get_id()?;
        let mut memos = self
            .repo
            .list_memos(FindMemo {
                id: Some(id),
                ..Default::default()
            })
            .await?;

        let mut memo: Memo = memos.pop().context(MemoNotFound)?.into();

        let relate_resources = self.relate_resource(id).await?;
        memo.resources = relate_resources.into_iter().map(|r| r.into()).collect();
        // TODO relate/reaction

        Ok(Response::new(memo))
    }

    async fn get_memo_by_uid(
        &self,
        request: Request<GetMemoByUidRequest>,
    ) -> Result<Response<Memo>, Status> {
        todo!()
    }

    async fn list_memos(
        &self,
        request: Request<ListMemosRequest>,
    ) -> Result<Response<ListMemosResponse>, Status> {
        let user = request.get_current_user();
        let mut find: FindMemo = request.get_ref().try_into().context(InvalidMemoFilter)?;
        find.completed(
            user.ok().map(|u| u.id),
            self.is_display_with_update_time().await,
        );
        let page_token = find.page_token;
        let mut memos = self.repo.list_memos(find).await?;

        // 是否有下一页
        let mut next_page_token = String::new();
        if let Some(page_token) = page_token {
            if let Some(next) = page_token.next_page(&mut memos) {
                next_page_token = serde_json::to_string(&next).unwrap_or_default();
            }
        }

        let memo_ids = memos.iter().map(|m| m.id).collect();
        let mut relate_resources = self.relate_resources(memo_ids).await?;
        let mut memo_list = Vec::new();
        for memo in memos {
            let resources = relate_resources.remove(&memo.id);
            let mut memo: Memo = memo.into();
            if let Some(resources) = resources {
                memo.resources = resources.into_iter().map(|r| r.into()).collect();
            }
            memo_list.push(memo);
        }
        // TODO relate/reaction

        Ok(Response::new(ListMemosResponse {
            memos: memo_list,
            next_page_token,
        }))
    }

    /// UpdateMemo updates a memo.
    async fn update_memo(
        &self,
        request: Request<UpdateMemoRequest>,
    ) -> Result<Response<Memo>, Status> {
        let user = request.get_current_user()?;
        let mut update: UpdateMemo = request.get_ref().into();
        update.creator_id = user.id;
        let memo_id = update.id;

        self.repo.update_memo(update).await?;

        let mut memos = self
            .repo
            .list_memos(FindMemo {
                id: Some(memo_id),
                ..Default::default()
            })
            .await?;
        let memo = memos.pop().context(MemoNotFound)?;

        let resources = self.relate_resource(memo.id).await?;
        let mut memo: Memo = memo.into();
        memo.resources = resources.into_iter().map(|r| r.into()).collect();
        // TODO relate/reaction

        Ok(Response::new(memo))
    }

    /// DeleteMemo deletes a memo by id.
    async fn delete_memo(
        &self,
        request: Request<DeleteMemoRequest>,
    ) -> Result<Response<()>, Status> {
        self.repo.delete_memo(request.get_ref().get_id()?).await?;
        Ok(Response::new(()))
    }

    async fn rebuild_memo_property(
        &self,
        request: Request<RebuildMemoPropertyRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn list_memo_tags(
        &self,
        request: Request<ListMemoTagsRequest>,
    ) -> Result<Response<ListMemoTagsResponse>, Status> {
        let mut tag_amounts = HashMap::new();
        let payloads = self
            .repo
            .list_memos(FindMemo {
                row_status: Some(RowStatus::Active),
                only_payload: true,
                exclude_content: true,
                exclude_comments: true,
                ..Default::default()
            })
            .await?;

        for p in payloads {
            if let Some(property) = p.payload.property {
                for tag in property.tags {
                    let count = tag_amounts.remove(&tag).unwrap_or(0);
                    tag_amounts.insert(tag, count + 1);
                }
            }
        }
        Ok(Response::new(ListMemoTagsResponse { tag_amounts }))
    }

    async fn rename_memo_tag(
        &self,
        request: Request<RenameMemoTagRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_memo_tag(
        &self,
        request: Request<DeleteMemoTagRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn list_memo_properties(
        &self,
        request: Request<ListMemoPropertiesRequest>,
    ) -> Result<Response<ListMemoPropertiesResponse>, Status> {
        let id = request.get_ref().get_id().ok();
        let payloads = self
            .repo
            .list_memos(FindMemo {
                id,
                row_status: Some(RowStatus::Active),
                only_payload: true,
                exclude_content: true,
                exclude_comments: true,
                ..Default::default()
            })
            .await?;

        let is_display_with_update_time = self.is_display_with_update_time().await;
        let entities = payloads
            .into_iter()
            .map(|p| {
                let name = p.get_name();
                let property = p.payload.property.map(|p| p.into());
                let display_time = if is_display_with_update_time {
                    p.updated_ts
                } else {
                    p.created_ts
                };
                MemoPropertyEntity {
                    name,
                    property,
                    display_time: Some(Timestamp {
                        seconds: display_time,
                        nanos: 0,
                    }),
                }
            })
            .collect();
        Ok(Response::new(ListMemoPropertiesResponse { entities }))
    }

    /// SetMemoResources sets resources for a memo.
    async fn set_memo_resources(
        &self,
        request: Request<SetMemoResourcesRequest>,
    ) -> Result<Response<()>, Status> {
        let memo_id = request.get_ref().get_id()?;
        let resources = &request.get_ref().resources;
        let relate_resources = self.relate_resource(memo_id).await?;

        let new_res_ids = resources
            .iter()
            .map(|s| s.get_id().unwrap_or_default())
            .collect();
        let old_res_ids = relate_resources.iter().map(|s| s.id).collect();

        self.set_resources_memo(memo_id, new_res_ids, old_res_ids)
            .await?;
        Ok(Response::new(()))
    }

    /// ListMemoResources lists resources for a memo.
    async fn list_memo_resources(
        &self,
        request: Request<ListMemoResourcesRequest>,
    ) -> Result<Response<ListMemoResourcesResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// SetMemoRelations sets relations for a memo.
    async fn set_memo_relations(
        &self,
        request: Request<SetMemoRelationsRequest>,
    ) -> Result<Response<()>, Status> {
        // TODO
        Ok(Response::new(()))
    }
    /// ListMemoRelations lists relations for a memo.
    async fn list_memo_relations(
        &self,
        request: Request<ListMemoRelationsRequest>,
    ) -> Result<Response<ListMemoRelationsResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// CreateMemoComment creates a comment for a memo.
    async fn create_memo_comment(
        &self,
        request: Request<CreateMemoCommentRequest>,
    ) -> Result<Response<Memo>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// ListMemoComments lists comments for a memo.
    async fn list_memo_comments(
        &self,
        request: Request<ListMemoCommentsRequest>,
    ) -> Result<Response<ListMemoCommentsResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// ExportMemos exports memos.
    async fn export_memos(
        &self,
        request: Request<ExportMemosRequest>,
    ) -> Result<Response<ExportMemosResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// ListMemoReactions lists reactions for a memo.
    async fn list_memo_reactions(
        &self,
        request: Request<ListMemoReactionsRequest>,
    ) -> Result<Response<ListMemoReactionsResponse>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// UpsertMemoReaction upserts a reaction for a memo.
    async fn upsert_memo_reaction(
        &self,
        request: Request<UpsertMemoReactionRequest>,
    ) -> Result<Response<Reaction>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
    /// DeleteMemoReaction deletes a reaction for a memo.
    async fn delete_memo_reaction(
        &self,
        request: Request<DeleteMemoReactionRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("unimplemented"))
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    CreateMemo {
        source: crate::dao::memo::CreateMemoError,
    },

    #[snafu(
        display("Maybe create memo failed, because return none"),
        context(suffix(false))
    )]
    MaybeCreateMemo,

    #[snafu(display("Memo not found"), context(suffix(false)))]
    MemoNotFound,

    #[snafu(context(false))]
    UpdateMemo {
        source: crate::dao::memo::UpdateMemoError,
    },

    #[snafu(context(false))]
    DeleteMemo {
        source: crate::dao::memo::DeleteMemoError,
    },

    #[snafu(context(false))]
    ListMemo {
        source: crate::dao::memo::ListMemoError,
    },

    #[snafu(display("Invalid memo filter: {source}"), context(suffix(false)))]
    InvalidMemoFilter { source: crate::api::memo::Error },
}
