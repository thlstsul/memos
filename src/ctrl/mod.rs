use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hyper::header;
use mime_guess::mime;
use tracing::error;

pub mod auth;
pub mod resource;
pub mod store;
pub mod system;

pub struct Resource {
    pub filename: String,
    pub blob: Vec<u8>,
}

impl IntoResponse for Resource {
    fn into_response(self) -> Response {
        let mime = mime_guess::from_path(&self.filename)
            .first_raw()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM.as_ref());
        let headers = [
            (header::CONTENT_TYPE, mime),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", self.filename),
            ),
        ];

        (headers, self.blob).into_response()
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
