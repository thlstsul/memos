use axum::{
    body::StreamBody,
    extract::{Multipart, Path, Query, State},
    http::HeaderValue,
    response::Result,
    routing::post,
    Json, Router,
};
use hyper::header::CONTENT_TYPE;
use snafu::{ensure, Snafu};

use crate::{
    api::{
        resource::WholeResource,
        system::{SystemSetting, SystemSettingKey},
        v1::resource::{CreateResourceResponse, ResourceQry},
    },
    state::AppState,
    svc::{resource::ResourceService, system::SystemService},
};

use super::auth::AuthSession;

const MEBI_BYTE: usize = 1024 * 1024;
const DEFAULT_MAX_MIB: usize = 32;

pub fn router() -> Router<AppState> {
    Router::new().route("/resource/blob", post(upload))
}

/// /o/r/:id
pub async fn stream_resource(
    auth_session: AuthSession,
    state: State<AppState>,
    Path(name): Path<String>,
    Query(ResourceQry { thumbnail }): Query<ResourceQry>,
) -> Result<super::Resource> {
    let svc = ResourceService::new(&state);
    let res = svc.get_resource_by_name(name).await?;
    let filename = res.filename.clone();
    let r#type = res.r#type.clone();
    let thumbnail = Some("1".to_owned()) == thumbnail && res.r#type.starts_with("image");

    let stream = svc.get_resource_stream(res, thumbnail).await?;
    let body = StreamBody::new(stream);
    Ok(super::Resource {
        filename,
        r#type,
        body,
    })
}

async fn upload(
    auth_session: AuthSession,
    state: State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<CreateResourceResponse>> {
    let user = auth_session.user.unwrap();
    if let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().unwrap_or_default().to_owned();
        let r#type = field
            .headers()
            .get(CONTENT_TYPE)
            .unwrap_or(&HeaderValue::from_static("multipart/form-data"))
            .to_str()
            .unwrap_or_default()
            .to_owned();
        let data = field.bytes().await?;
        let size = data.len();

        let sys_svc = SystemService::new(&state);
        let max_upload_size_mib = SystemSetting {
            name: SystemSettingKey::MaxUploadSizeMiB,
            value: "32".to_owned(),
            description: "default max upload size".to_owned(),
        };
        let max_upload_size_mib = sys_svc
            .find_setting(SystemSettingKey::MaxUploadSizeMiB)
            .await?
            .unwrap_or(max_upload_size_mib);

        let max_upload_size_bytes: usize =
            max_upload_size_mib.value.parse().unwrap_or(DEFAULT_MAX_MIB) * MEBI_BYTE;

        ensure!(
            max_upload_size_bytes > size,
            FileSizeLimit {
                size: max_upload_size_mib.value
            }
        );

        // 默认保存到 turso
        let create = WholeResource {
            filename,
            r#type,
            size,
            creator_id: user.id,
            blob: data.to_vec(),
            ..Default::default()
        };

        let res_svc = ResourceService::new(&state);
        let res = res_svc.create_resource(create).await?;
        let created_ts = res.create_time.unwrap_or_default();
        let resp = CreateResourceResponse {
            id: res.id,
            creator_id: user.id,
            created_ts: created_ts.seconds,
            updated_ts: created_ts.seconds,
            filename: res.filename,
            external_link: res.external_link,
            r#type: res.r#type,
            size: res.size,
        };
        Ok(Json(resp))
    } else {
        Err(Error::UploadFileNotFound.into())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Upload file not found"), context(suffix(false)))]
    UploadFileNotFound,
    #[snafu(
        display("File size exceeds allowed limit of {size} MiB"),
        context(suffix(false))
    )]
    FileSizeLimit { size: String },
}
