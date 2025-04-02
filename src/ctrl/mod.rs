pub mod auth;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hyper::header;
use tracing::error;

use crate::dao::turso::Turso;
use crate::svc::Service;

use self::auth::Backend;

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend<Service<Turso>>>;
pub type AuthError = axum_login::Error<Backend<Service<Turso>>>;

impl IntoResponse for crate::svc::session::Error {
    fn into_response(self) -> Response {
        error_response(StatusCode::BAD_REQUEST, self)
    }
}

impl IntoResponse for crate::svc::user::Error {
    fn into_response(self) -> Response {
        let status_code = match self {
            crate::svc::user::Error::Login { .. } => StatusCode::UNAUTHORIZED,
            crate::svc::user::Error::UserNotFound { .. } => StatusCode::NOT_FOUND,
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
