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
use ctrl::user;
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

use crate::{
    ctrl::{auth::AuthLayer, memo},
    svc::ServiceFactory,
};

mod api;
mod ctrl;
mod dao;
mod hybrid;
mod svc;

#[shuttle_runtime::main]
async fn grpc_web(
    #[shuttle_turso::Turso(addr = "{secrets.TURSO_URL}", token = "{secrets.TURSO_TOKEN}")]
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

    let protected = Router::new()
        .merge(user::router())
        .route_layer(login_required!(Backend));

    let public = Router::new()
        .merge(auth::router())
        .merge(system::router())
        .merge(memo::router());

    let api_v1 = Router::new().merge(protected).merge(public);

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

    let tonic_router = Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .layer(AuthLayer::new(auth_manager_layer))
        .add_service(user)
        .add_service(tag);

    Ok(GrpcWebService {
        axum_router,
        tonic_router,
    })
}
