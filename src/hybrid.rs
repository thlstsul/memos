/// TODO tonic 未升级hyper 1.0，已可升级
/// fork from https://github.com/snoyberg/tonic-example
/// fork from https://github.com/tokio-rs/axum/examples/rest-grpc-multiplex
/// axum::Router impl Service 可以省一层
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Poll;

use std::{future::Future, sync::Arc};

use crate::ctrl::auth::Backend;
use crate::svc::markdown::MarkdownService;
use crate::svc::user::UserService;
use crate::svc::EmptyService;
use axum::{error_handling::HandleErrorLayer, BoxError, Router};
use axum_login::{
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use hyper::header::CONTENT_TYPE;
use hyper::HeaderMap;
use hyper::StatusCode;
use hyper::{body::HttpBody, Body, Request, Response};
use pin_project::pin_project;
use shuttle_runtime::{CustomError, Error};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower::layer::util::{Identity, Stack};
use tower::Service;
use tower::ServiceBuilder;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::error;

use crate::{
    ctrl::auth::AuthLayer,
    svc::{
        auth::AuthService,
        idp::IDPService,
        inbox::InboxService,
        memo::MemoService,
        resource::ResourceService,
        webhook::WebhookService,
        workspace::{WorkspaceService, WorkspaceSettingService},
    },
};

mod resource;

type Repo = crate::dao::turso::Turso;
type RepoService = crate::svc::Service<Repo>;
type SessionStore = crate::svc::session::SessionStore<Repo>;
type GrpcRouter = tonic::transport::server::Router<
    Stack<
        AuthLayer<RepoService, SessionStore>,
        Stack<GrpcWebLayer, Stack<TraceLayer<SharedClassifier<ServerErrorsAsFailures>>, Identity>>,
    >,
>;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
struct AppState<RS: ResourceService> {
    res_service: Arc<RS>,
}

pub struct GrpcWebService {
    axum_router: axum::Router,
    tonic_router: GrpcRouter,
}

impl GrpcWebService {
    pub fn new(repo: Repo) -> Self {
        let session_store = SessionStore::new(repo.clone());
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(30)));

        let svc = Arc::new(RepoService::new(repo));
        let backend = Backend::new(svc.clone());
        let auth_manager_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
        let auth_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|e: BoxError| async move {
                error!("{e}");
                StatusCode::BAD_REQUEST
            }))
            .layer(auth_manager_layer.clone());

        let index_file = ServeFile::new("web/dist/index.html").precompressed_br();
        let axum_router = Router::new()
            .merge(resource::router())
            .layer(auth_service)
            .route_service("/home", index_file.clone())
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
            "/memos.api.v1.AuthService/SignIn",
            "/memos.api.v1.AuthService/GetAuthStatus",
            "/memos.api.v1.MemoService/ListMemos",
            "/memos.api.v1.MemoService/ListMemoRelations",
            "/memos.api.v1.MemoService/ListMemoResources",
            "/memos.api.v1.WorkspaceSettingService/GetWorkspaceSetting",
            "/memos.api.v1.WorkspaceService/GetWorkspaceProfile",
        ]
        .into_iter()
        .map(|s| s.to_owned())
        .collect();

        let user = svc.clone().user_server();
        let memo = svc.clone().memo_server();
        let resource = svc.clone().resource_server();
        let setting = svc.clone().workspace_setting_server();
        let workspace = svc.clone().workspace_server();
        let state = AppState { res_service: svc };

        let empty_svc = Arc::new(EmptyService);
        let idp = empty_svc.clone().idp_server();
        let inbox = empty_svc.clone().inbox_server();
        let webhook = empty_svc.clone().webhook_server();
        let markdown = empty_svc.clone().markdown_server();
        let auth = empty_svc.auth_server();

        let axum_router = axum_router.with_state(state);

        let tonic_router = Server::builder()
            .accept_http1(true)
            .layer(TraceLayer::new_for_http())
            .layer(GrpcWebLayer::new())
            .layer(AuthLayer::new(auth_manager_layer, public_path))
            .add_service(user)
            .add_service(auth)
            .add_service(markdown)
            .add_service(memo)
            .add_service(resource)
            .add_service(idp)
            .add_service(inbox)
            .add_service(webhook)
            .add_service(workspace)
            .add_service(setting);

        Self {
            axum_router,
            tonic_router,
        }
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for GrpcWebService {
    /// Takes the router that is returned by the user in their [shuttle_runtime::main] function
    /// and binds to an address passed in by shuttle.
    async fn bind(mut self, addr: SocketAddr) -> Result<(), Error> {
        let web = self.axum_router;
        let grpc = self.tonic_router.into_service();
        let hybrid_service = HybridService { web, grpc };

        let server = hyper::Server::bind(&addr).serve(tower::make::Shared::new(hybrid_service));
        server.await.map_err(CustomError::new)?;
        Ok(())
    }
}

pub type ShuttleGrpcWeb = Result<GrpcWebService, Error>;

#[derive(Debug, Clone)]
struct HybridService<Web, Grpc> {
    web: Web,
    grpc: Grpc,
}

impl<Web, Grpc, WebBody, GrpcBody> Service<Request<Body>> for HybridService<Web, Grpc>
where
    Web: Service<Request<Body>, Response = Response<WebBody>>,
    Grpc: Service<Request<Body>, Response = Response<GrpcBody>>,
    Web::Error: Into<BoxError>,
    Grpc::Error: Into<BoxError>,
{
    type Response = Response<HybridBody<WebBody, GrpcBody>>;
    type Error = BoxError;
    type Future = HybridFuture<Web::Future, Grpc::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.web.poll_ready(cx) {
            Poll::Ready(Ok(())) => match self.grpc.poll_ready(cx) {
                Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                Poll::Pending => Poll::Pending,
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let content_type = req.headers().get(CONTENT_TYPE).map(|x| x.as_bytes());
        if content_type == Some(b"application/grpc-web+proto")
            || content_type == Some(b"application/grpc-web")
        {
            HybridFuture::Grpc(self.grpc.call(req))
        } else {
            HybridFuture::Web(self.web.call(req))
        }
    }
}

#[pin_project(project = HybridBodyProj)]
enum HybridBody<WebBody, GrpcBody> {
    Web(#[pin] WebBody),
    Grpc(#[pin] GrpcBody),
}

impl<WebBody, GrpcBody> HttpBody for HybridBody<WebBody, GrpcBody>
where
    WebBody: HttpBody + Send + Unpin,
    GrpcBody: HttpBody<Data = WebBody::Data> + Send + Unpin,
    WebBody::Error: std::error::Error + Send + Sync + 'static,
    GrpcBody::Error: std::error::Error + Send + Sync + 'static,
{
    type Data = WebBody::Data;
    type Error = BoxError;

    fn is_end_stream(&self) -> bool {
        match self {
            HybridBody::Web(b) => b.is_end_stream(),
            HybridBody::Grpc(b) => b.is_end_stream(),
        }
    }

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.project() {
            HybridBodyProj::Web(b) => b.poll_data(cx).map_err(|e| e.into()),
            HybridBodyProj::Grpc(b) => b.poll_data(cx).map_err(|e| e.into()),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        match self.project() {
            HybridBodyProj::Web(b) => b.poll_trailers(cx).map_err(|e| e.into()),
            HybridBodyProj::Grpc(b) => b.poll_trailers(cx).map_err(|e| e.into()),
        }
    }
}

#[pin_project(project = HybridFutureProj)]
enum HybridFuture<WebFuture, GrpcFuture> {
    Web(#[pin] WebFuture),
    Grpc(#[pin] GrpcFuture),
}

impl<WebFuture, GrpcFuture, WebBody, GrpcBody, WebError, GrpcError> Future
    for HybridFuture<WebFuture, GrpcFuture>
where
    WebFuture: Future<Output = Result<Response<WebBody>, WebError>>,
    GrpcFuture: Future<Output = Result<Response<GrpcBody>, GrpcError>>,
    WebError: Into<BoxError>,
    GrpcError: Into<BoxError>,
{
    type Output = Result<Response<HybridBody<WebBody, GrpcBody>>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        match self.project() {
            HybridFutureProj::Web(a) => match a.poll(cx) {
                Poll::Ready(Ok(res)) => Poll::Ready(Ok(res.map(HybridBody::Web))),
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                Poll::Pending => Poll::Pending,
            },
            HybridFutureProj::Grpc(b) => match b.poll(cx) {
                Poll::Ready(Ok(res)) => Poll::Ready(Ok(res.map(HybridBody::Grpc))),
                Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
