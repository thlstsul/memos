use std::fmt::Write;

use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    body::BoxBody,
    cookie::{time::Duration, Key},
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    web::{self, BytesMut},
    HttpResponse, ResponseError, Scope,
};

use self::{
    auth::*,
    system::{ping, status},
    user::*,
};

mod auth;
mod system;
mod user;

const ONE_MONTH: Duration = Duration::days(30);

pub fn root(
    key: Key,
) -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("")
        .wrap(IdentityMiddleware::default())
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), key)
                .cookie_name("memos.access-token".to_owned())
                .cookie_secure(false)
                .session_lifecycle(PersistentSession::default().session_ttl(ONE_MONTH))
                .build(),
        )
        .service(v1_scope())
}

fn v1_scope() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope("/api/v1")
        .service(ping)
        .service(status)
        .service(signin)
        .service(signout)
        .service(me)
        .service(user_detail)
}

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
