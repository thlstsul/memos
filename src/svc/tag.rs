use std::sync::Arc;

use libsql_client::Client;
use tonic::{Request, Response, Status};
use tracing::error;

use crate::api::v2::{
    tag_service_server, DeleteTagRequest, DeleteTagResponse, GetTagSuggestionsRequest,
    GetTagSuggestionsResponse, ListTagsRequest, ListTagsResponse, Tag, UpsertTagRequest,
    UpsertTagResponse,
};
use crate::dao::tag::TagDao;
use crate::svc::get_current_user;

pub struct TagService {
    dao: TagDao,
}

impl TagService {
    pub fn new(client: &Arc<Client>) -> Self {
        Self {
            dao: TagDao {
                client: Arc::clone(client),
            },
        }
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

        self.dao.upsert_tag(name.clone(), user.id).await?;
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
        let tags = self.dao.list_tags(user.id).await?;
        Ok(Response::new(tags.into()))
    }
    async fn delete_tag(
        &self,
        request: Request<DeleteTagRequest>,
    ) -> Result<Response<DeleteTagResponse>, Status> {
        if let Some(tag) = &request.get_ref().tag {
            let user = get_current_user(&request)?;
            self.dao.delete_tag(tag.name.clone(), user.id).await?;
        }

        Ok(Response::new(DeleteTagResponse {}))
    }
    async fn get_tag_suggestions(
        &self,
        request: Request<GetTagSuggestionsRequest>,
    ) -> Result<Response<GetTagSuggestionsResponse>, Status> {
        todo!()
    }
}

impl From<crate::dao::tag::Error> for Status {
    fn from(value: crate::dao::tag::Error) -> Self {
        error!("{value}");
        Status::internal(value.to_string())
    }
}
