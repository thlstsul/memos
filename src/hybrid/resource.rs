use axum::{
    body::StreamBody,
    extract::{Path, Query, State},
    response::{IntoResponse, Response, Result},
    routing::get,
    Router,
};
use hyper::StatusCode;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::error;

use crate::{model::resource::ResourceQry, svc::resource::ResourceService};

use super::AppState;

pub fn router<RS: ResourceService>() -> Router<AppState<RS>> {
    Router::new().route("/file/resources/:id/:filename", get(stream_resource))
}

/// /file/resources/:id/:filename
async fn stream_resource<RS: ResourceService>(
    State(state): State<AppState<RS>>,
    Path((id, filename)): Path<(i32, String)>,
    Query(ResourceQry { thumbnail }): Query<ResourceQry>,
) -> Result<Resource> {
    let res = state.res_service.get_resource_by_id(id).await?;
    let r#type = res.r#type.clone();
    let thumbnail = Some("1".to_owned()) == thumbnail && res.r#type.starts_with("image");

    let stream = state
        .res_service
        .get_resource_stream(id, res.filename, thumbnail)
        .await?;
    let body = StreamBody::new(stream);
    Ok(Resource {
        _filename: filename,
        r#type,
        body,
    })
}

struct Resource {
    pub _filename: String,
    pub r#type: String,
    pub body: StreamBody<ReaderStream<File>>,
}

impl IntoResponse for Resource {
    fn into_response(self) -> Response {
        let headers = [("Content-Type", self.r#type)];

        (headers, self.body).into_response()
    }
}

impl IntoResponse for crate::svc::resource::Error {
    fn into_response(self) -> Response {
        error!("{self}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            [("Content-Type", "text/json; charset=utf-8")],
            format!(
                r#"{{"error": "code={}, message={}", "message": "{}"}}"#,
                StatusCode::INTERNAL_SERVER_ERROR,
                self,
                self
            ),
        )
            .into_response()
    }
}
