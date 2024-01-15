use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    http::HeaderValue,
    response::Result,
    routing::post,
    Json, Router,
};
use hyper::header::CONTENT_TYPE;
use libsql_client::Client;
use snafu::Snafu;

use crate::api::{resource::CreateResource, v2::Resource};

use super::auth::AuthSession;

pub fn router() -> Router<Arc<Client>> {
    Router::new().route("/resource/blob", post(upload))
}

async fn upload(
    auth_session: AuthSession,
    client: State<Arc<Client>>,
    mut multipart: Multipart,
) -> Result<Json<Resource>> {
    if let Some(field) = multipart.next_field().await? {
        let filename = field.name().unwrap_or_default().to_owned();
        let r#type = field
            .headers()
            .get(CONTENT_TYPE)
            .unwrap_or(&HeaderValue::from_static("multipart/form-data"))
            .to_str()
            .unwrap_or_default()
            .to_owned();
        let data = field.bytes().await?;

        // 默认保存到 turso
        let create = CreateResource {
            filename,
            r#type,
            size: todo!(),
            creator_id: todo!(),
            blob: todo!(),
            external_link: todo!(),
            internal_path: todo!(),
            id: todo!(),
            created_ts: todo!(),
            updated_ts: todo!(),
            memo_id: todo!(),
        };
    }
    todo!()
}
