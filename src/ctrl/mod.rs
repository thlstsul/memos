use std::fmt::Write;

use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    body::BoxBody,
    cookie::{time::Duration, Key},
    dev::{PeerAddr, ServiceFactory, ServiceRequest, ServiceResponse},
    error,
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    web::{self, BytesMut},
    Error, HttpRequest, HttpResponse, ResponseError, Scope,
};
use awc::Client;
use url::Url;

use self::{
    auth::*,
    system::{ping, status},
    user::*,
};

mod auth;
mod memo;
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
        .default_service(web::to(v2_forward))
}

async fn v2_forward(
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>,
    url: web::Data<Url>,
    client: web::Data<Client>,
    _ident: Identity,
) -> Result<HttpResponse, Error> {
    let mut new_url = (**url).clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();

    // TODO: This forwarded implementation is incomplete as it only handles the unofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => {
            forwarded_req.insert_header(("x-forwarded-for", addr.ip().to_string()))
        }
        None => forwarded_req,
    };

    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res))
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
