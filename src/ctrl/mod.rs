use axum::body::StreamBody;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hyper::header;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::error;

pub mod auth;
pub mod resource;
pub mod store;
pub mod system;

pub struct Resource {
    pub filename: String,
    pub r#type: String,
    pub body: StreamBody<ReaderStream<File>>,
}

impl IntoResponse for Resource {
    fn into_response(self) -> Response {
        let headers = [
            (header::CONTENT_TYPE, self.r#type),
            (header::CACHE_CONTROL, "max-age=3600".to_owned()),
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; script-src 'none'; img-src 'self'; media-src 'self'; sandbox;"
                    .to_owned(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", self.filename),
            ),
        ];

        (headers, self.body).into_response()
    }
}

impl IntoResponse for crate::ctrl::resource::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::BAD_REQUEST, self)
    }
}

impl IntoResponse for crate::svc::system::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, self)
    }
}

impl IntoResponse for crate::svc::resource::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, self)
    }
}

impl IntoResponse for crate::svc::user::Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            crate::svc::user::Error::Login { .. } => StatusCode::UNAUTHORIZED,
            crate::svc::user::Error::UserNotFound { .. } => StatusCode::NOT_FOUND,
            crate::svc::user::Error::InvalidUsername { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        error_response(status_code, self)
    }
}

pub fn error_response<T>(status_code: StatusCode, slf: T) -> Response
where
    T: std::error::Error,
{
    error!("{slf}");
    (
        status_code,
        [(header::CONTENT_TYPE, "text/json; charset=utf-8")],
        format!(
            r#"{{"error": "code={}, message={}", "message": "{}"}}"#,
            status_code, slf, slf
        ),
    )
        .into_response()
}
