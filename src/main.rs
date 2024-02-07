#![allow(unused_variables)]

use axum::{error_handling::HandleErrorLayer, routing::get, BoxError, Router};
use axum_login::{
    login_required,
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use ctrl::system;
use ctrl::{
    auth::{self, Backend},
    resource,
};
use hybrid::{GrpcWebService, ShuttleGrpcWeb};
use hyper::StatusCode;
use libsql::Connection;
use shuttle_secrets::SecretStore;
use svc::user::UserService;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::error;

use crate::{
    ctrl::{auth::AuthLayer, resource::stream_resource, store::TursoStore},
    state::AppState,
    svc::ServiceFactory,
};

mod api;
mod ctrl;
mod dao;
mod hybrid;
mod state;
mod svc;
mod util;

#[shuttle_runtime::main]
async fn grpc_web(
    #[shuttle_turso::Turso(addr = "{secrets.TURSO_URL}", token = "{secrets.TURSO_TOKEN}")]
    conn: Connection,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> ShuttleGrpcWeb {
    let state = AppState::new(conn);
    let session_store = TursoStore::new(&state);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let backend = Backend::new(UserService::new(&state));
    let auth_manager_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            error!("{e}");
            StatusCode::BAD_REQUEST
        }))
        .layer(auth_manager_layer.clone());

    let public = Router::new().merge(auth::router()).merge(system::router());
    let protected = Router::new()
        .merge(resource::router())
        .route_layer(login_required!(Backend));

    let api_v1 = Router::new().merge(public).merge(protected);

    let index_file = ServeFile::new("web/dist/index.html").precompressed_br();
    let axum_router = Router::new()
        .nest("/api/v1", api_v1)
        .route("/o/r/:id", get(stream_resource))
        .layer(auth_service)
        .route_service("/auth", index_file.clone())
        .route_service("/explore", index_file.clone())
        .route_service("/resource", index_file.clone())
        .route_service("/timeline", index_file.clone())
        .route_service("/setting", index_file)
        .nest_service(
            "/",
            ServeDir::new("web/dist")
                .precompressed_br()
                .append_index_html_on_directories(true),
        )
        .layer(TraceLayer::new_for_http());

    let public_path = vec![
        "/memos.api.v2.AuthService/GetAuthStatus",
        "/memos.api.v2.MemoService/ListMemos",
        "/memos.api.v2.MemoService/ListMemoRelations",
        "/memos.api.v2.MemoService/ListMemoResources",
    ]
    .into_iter()
    .map(|s| s.to_owned())
    .collect();

    let user = ServiceFactory::get_user(&state);
    let tag = ServiceFactory::get_tag(&state);
    let auth = ServiceFactory::get_auth();
    let memo = ServiceFactory::get_memo(&state);
    let resource = ServiceFactory::get_resource(&state);
    let inbox = ServiceFactory::get_inbox();
    let webhook = ServiceFactory::get_webhook();

    let axum_router = axum_router.with_state(state);

    let tonic_router = Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .layer(AuthLayer::new(auth_manager_layer, public_path))
        .add_service(user)
        .add_service(tag)
        .add_service(auth)
        .add_service(memo)
        .add_service(resource)
        .add_service(inbox)
        .add_service(webhook);

    Ok(GrpcWebService {
        axum_router,
        tonic_router,
    })
}
