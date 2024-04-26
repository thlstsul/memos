#![allow(unused_variables)]
#![allow(clippy::enum_variant_names)]

use axum::{error_handling::HandleErrorLayer, routing::get, BoxError, Router};
use axum_login::{
    login_required,
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use ctrl::{auth::Backend, resource};
use hybrid::{GrpcWebService, ShuttleGrpcWeb};
use hyper::StatusCode;
use libsql::Database;
use shuttle_runtime::SecretStore;
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
    ctrl::{auth::AuthLayer, resource::stream_resource, store::TursoStore, system},
    state::AppState,
    svc::{
        auth::AuthService,
        inbox::InboxService,
        memo::MemoService,
        resource::ResourceService,
        tag::TagService,
        webhook::WebhookService,
        workspace::{WorkspaceService, WorkspaceSettingService},
    },
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
    repo: Database,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleGrpcWeb {
    let state = AppState::new(repo);
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

    let public = Router::new().merge(system::router());
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
        "/memos.api.v2.AuthService/SignIn",
        "/memos.api.v2.AuthService/GetAuthStatus",
        "/memos.api.v2.MemoService/ListMemos",
        "/memos.api.v2.MemoService/ListMemoRelations",
        "/memos.api.v2.MemoService/ListMemoResources",
        "/memos.api.v2.WorkspaceSettingService/GetWorkspaceSetting",
    ]
    .into_iter()
    .map(|s| s.to_owned())
    .collect();

    let user = UserService::server(&state);
    let tag = TagService::server(&state);
    let auth = AuthService::server();
    let memo = MemoService::server(&state);
    let resource = ResourceService::server(&state);
    let inbox = InboxService::server();
    let webhook = WebhookService::server();
    let workspace = WorkspaceService::server();
    let setting = WorkspaceSettingService::server(&state);

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
        .add_service(webhook)
        .add_service(workspace)
        .add_service(setting);

    Ok(GrpcWebService {
        axum_router,
        tonic_router,
    })
}
