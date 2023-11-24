use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, Scope,
};

use self::auth::sign_in;

mod auth;

const ONE_MONTH: Duration = Duration::days(30);

pub fn v1_scope_auth() -> Scope {
    web::scope("/api/v1").service(sign_in)
}

pub fn v1_scope(
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
    web::scope("/api/v1")
        .wrap(IdentityMiddleware::default())
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), key)
                .cookie_name("memos.access-token".to_owned())
                .cookie_secure(false)
                .session_lifecycle(PersistentSession::default().session_ttl(ONE_MONTH))
                .build(),
        )
}
