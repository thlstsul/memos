use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hyper::header;
use tracing::error;

pub mod auth;
pub mod store;
pub mod system;

impl IntoResponse for crate::api::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::BAD_REQUEST, self)
    }
}

impl IntoResponse for crate::svc::system::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, self)
    }
}

impl IntoResponse for crate::svc::memo::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, self)
    }
}

impl IntoResponse for crate::svc::user::Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            crate::svc::user::Error::Login { .. } => StatusCode::UNAUTHORIZED,
            crate::svc::user::Error::UserNotFound { .. } => StatusCode::NOT_FOUND,
            crate::svc::user::Error::QueryUserFailed { .. }
            | crate::svc::user::Error::QuerySettingFailed { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
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
