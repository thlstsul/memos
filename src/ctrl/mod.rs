use std::fmt::Write;

use self::{
    auth::*,
    system::{ping, status},
    user::*,
};

mod auth;
mod memo;
mod system;
mod user;

impl ResponseError for crate::api::Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        error_response(self)
    }
}

impl ResponseError for crate::svc::system::Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        error_response(self)
    }
}

impl ResponseError for crate::svc::auth::Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        error_response(self)
    }
}

impl ResponseError for crate::svc::user::Error {
    fn status_code(&self) -> StatusCode {
        match self {
            crate::svc::user::Error::UserNotFound { .. } => StatusCode::NOT_FOUND,
            crate::svc::user::Error::QueryUserFailed { .. }
            | crate::svc::user::Error::QuerySettingFailed { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        error_response(self)
    }
}

pub fn error_response<T>(slf: &T) -> HttpResponse<BoxBody>
where
    T: ResponseError,
{
    let mut res = HttpResponse::new(slf.status_code());

    let mut buf = BytesMut::new();
    let _ = write!(
        &mut buf,
        r#"{{
        "error": "code={}, message={}",
        "message": "{}"
        }}"#,
        slf.status_code(),
        slf,
        slf
    );

    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/json; charset=utf-8"),
    );

    res.set_body(BoxBody::new(buf))
}
