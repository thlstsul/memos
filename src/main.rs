use actix_files::NamedFile;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    body::MessageBody,
    cookie::{time::Duration, Key},
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get, post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder, Scope,
};
use shuttle_actix_web::ShuttleActixWeb;

mod pb;

const ONE_MONTH: Duration = Duration::days(30);

#[get(r"/{filename:.*\..*}")]
async fn static_file(filename: web::Path<String>) -> impl Responder {
    let path = format!("{}/{}", "web/dist", filename);
    NamedFile::open_async(path).await
}

#[post("/auth/signin")]
async fn sign_in() -> impl Responder {
    todo!()
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let secret_key = Key::generate();

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/api/v1")
                .wrap(IdentityMiddleware::default())
                .wrap(
                    SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                        .cookie_name("memos.access-token".to_owned())
                        .cookie_secure(false)
                        .session_lifecycle(PersistentSession::default().session_ttl(ONE_MONTH))
                        .build(),
                ),
        )
        .service(static_file)
        .default_service(web::to(HttpResponse::NotFound));
    };

    Ok(config.into())
}

fn v1_scope() -> Scope {
    web::scope("/api/v1").service(sign_in)
}

fn auth_v1_scope(
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
