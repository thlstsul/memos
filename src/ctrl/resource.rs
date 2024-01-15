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
use snafu::{ensure, Snafu};

use crate::{
    api::{
        resource::CreateResource,
        system::{SystemSetting, SystemSettingKey},
        v1::resource::CreateResourceResponse,
    },
    svc::{resource::ResourceService, system::SystemService},
};

use super::auth::AuthSession;

const MEBI_BYTE: usize = 1024 * 1024;

pub fn router() -> Router<Arc<Client>> {
    Router::new().route("/resource/blob", post(upload))
}

async fn upload(
    auth_session: AuthSession,
    client: State<Arc<Client>>,
    mut multipart: Multipart,
) -> Result<Json<CreateResourceResponse>> {
    let user = auth_session.user.ok_or(crate::ctrl::Error::CurrentUser)?;
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

        let sys_svc = SystemService::new(&client);
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
            max_upload_size_mib.value.parse().unwrap_or(32) * MEBI_BYTE;

        ensure!(
            max_upload_size_bytes > size,
            FileSizeLimit {
                size: max_upload_size_mib.value
            }
        );

        // 默认保存到 turso
        let create = CreateResource {
            filename,
            r#type,
            size,
            creator_id: user.id,
            blob: Some(data.to_vec()),
            ..Default::default()
        };

        let res_svc = ResourceService::new(&client);
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
