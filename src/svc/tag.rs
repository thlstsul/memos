use std::sync::Arc;

use libsql_client::Client;
use tonic::{Request, Response, Status};

use crate::api::v2::{
    tag_service_server, DeleteTagRequest, DeleteTagResponse, ListTagsRequest, ListTagsResponse,
    UpsertTagRequest, UpsertTagResponse,
};
use crate::dao::tag::TagDao;

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
        todo!()
    }
    async fn list_tags(
        &self,
        request: Request<ListTagsRequest>,
    ) -> Result<Response<ListTagsResponse>, Status> {
        let creator = request.into_inner().get_creator()?;
        let tags = self.dao.list_tags(creator).await?;
        Ok(Response::new(tags.into()))
    }
    async fn delete_tag(
        &self,
        request: Request<DeleteTagRequest>,
    ) -> Result<Response<DeleteTagResponse>, Status> {
        todo!()
    }
}

impl From<crate::dao::tag::Error> for Status {
    fn from(value: crate::dao::tag::Error) -> Self {
        Status::internal(value.to_string())
    }
}
