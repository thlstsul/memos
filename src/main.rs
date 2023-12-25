#![allow(unused_variables)]
use std::sync::Arc;

use axum::{error_handling::HandleErrorLayer, BoxError, Router};
use axum_login::{
    login_required,
    tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use ctrl::auth::{self, Backend};
use ctrl::system;
use hybrid::{GrpcWebService, ShuttleGrpcWeb};
use hyper::StatusCode;
use libsql_client::client::Client;
use shuttle_secrets::SecretStore;
use svc::user::UserService;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{ctrl::auth::AuthLayer, svc::ServiceFactory};

mod api;
mod ctrl;
mod dao;
mod hybrid;
mod svc;

#[shuttle_runtime::main]
async fn grpc_web(
    #[shuttle_turso::Turso(
        addr = "{secrets.TURSO_URL}",
        token = "{secrets.TURSO_TOKEN}",
        local_addr = "{secrets.TURSO_URL}"
    )]
    client: Client,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> ShuttleGrpcWeb {
    let client = Arc::new(client);
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let backend = Backend::new(UserService::new(&client));
    let auth_manager_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(auth_manager_layer.clone());

    let public = Router::new().merge(auth::router()).merge(system::router());

    let api_v1 = Router::new().merge(public);

    let axum_router = Router::new()
        .nest("/api/v1", api_v1)
        .layer(auth_service)
        .layer(TraceLayer::new_for_http())
        .route_service("/auth", ServeFile::new("web/dist/index.html"))
        .nest_service(
            "/",
            ServeDir::new("web/dist").append_index_html_on_directories(true),
        );

    let axum_router = axum_router.with_state(client.clone());

    let user = ServiceFactory::get_user(&client);
    let tag = ServiceFactory::get_tag(&client);
    let auth = ServiceFactory::get_auth();
    let memo = ServiceFactory::get_memo(&client);

    let public_path = vec![
        "/memos.api.v2.AuthService/GetAuthStatus",
        "/memos.api.v2.MemoService/ListMemos",
        "/memos.api.v2.MemoService/ListMemoRelations",
        "/memos.api.v2.MemoService/ListMemoResources",
    ]
    .into_iter()
    .map(|s| s.to_owned())
    .collect();

    let tonic_router = Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .layer(AuthLayer::new(auth_manager_layer, public_path))
        .add_service(user)
        .add_service(tag)
        .add_service(auth)
        .add_service(memo);

    Ok(GrpcWebService {
        axum_router,
        tonic_router,
    })
}
