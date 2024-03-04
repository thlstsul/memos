use snafu::{ResultExt, Snafu};
use tonic::{Request, Response, Status};

use crate::api::v2::tag_service_server::TagServiceServer;
use crate::api::v2::{
    tag_service_server, DeleteTagRequest, DeleteTagResponse, GetTagSuggestionsRequest,
    GetTagSuggestionsResponse, ListTagsRequest, ListTagsResponse, RenameTagRequest,
    RenameTagResponse, Tag, UpsertTagRequest, UpsertTagResponse,
};
use crate::dao::tag::TagDao;
use crate::state::AppState;
use crate::svc::get_current_user;

pub struct TagService {
    dao: TagDao,
}

impl TagService {
    pub fn new(state: &AppState) -> Self {
        Self {
            dao: TagDao {
                state: state.clone(),
            },
        }
    }

    pub fn server(state: &AppState) -> TagServiceServer<TagService> {
        TagServiceServer::new(TagService::new(state))
    }
}

#[tonic::async_trait]
impl tag_service_server::TagService for TagService {
    async fn upsert_tag(
        &self,
        request: Request<UpsertTagRequest>,
    ) -> Result<Response<UpsertTagResponse>, Status> {
        let user = get_current_user(&request)?;
        let creator = user.name.clone();
        let creator_id = user.id;
        let name = &request.get_ref().name;

        self.dao
            .upsert_tag(name, creator_id)
            .await
            .context(UpsertTag)?;
        Ok(Response::new(UpsertTagResponse {
            tag: Some(Tag {
                name: name.clone(),
                creator,
            }),
        }))
    }
    async fn list_tags(
        &self,
        request: Request<ListTagsRequest>,
    ) -> Result<Response<ListTagsResponse>, Status> {
        let user = get_current_user(&request)?;
        let tags = self.dao.list_tags(user.id).await.context(ListTag)?;
        Ok(Response::new(tags.into()))
    }
    async fn delete_tag(
        &self,
        request: Request<DeleteTagRequest>,
    ) -> Result<Response<DeleteTagResponse>, Status> {
        if let Some(tag) = &request.get_ref().tag {
            let user = get_current_user(&request)?;
            self.dao
                .delete_tag(&tag.name, user.id)
                .await
                .context(DeleteTag)?;
        }

        Ok(Response::new(DeleteTagResponse {}))
    }
    async fn get_tag_suggestions(
        &self,
        request: Request<GetTagSuggestionsRequest>,
    ) -> Result<Response<GetTagSuggestionsResponse>, Status> {
        todo!()
    }
    async fn rename_tag(
        &self,
        request: Request<RenameTagRequest>,
    ) -> Result<Response<RenameTagResponse>, Status> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[snafu(display("Failed to get tag list: {source}"), context(suffix(false)))]
    ListTag { source: crate::dao::Error },

    #[snafu(
        display("Failed to update/insert tag: {source}"),
        context(suffix(false))
    )]
    UpsertTag { source: crate::dao::Error },

    #[snafu(display("Failed to delete tag: {source}"), context(suffix(false)))]
    DeleteTag { source: crate::dao::Error },
}
