mod resource;

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Poll;

use std::{future::Future, sync::Arc};

use crate::ctrl::auth::Backend;
use crate::svc::markdown::MarkdownService;
use crate::svc::user::UserService;
use crate::svc::EmptyService;
use axum::{BoxError, Router};
use axum_login::tower_sessions::SessionManager;
use axum_login::AuthManager;
use axum_login::{
    tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use http_body::Body;
use http_body_util::BodyExt;
use hyper::server::conn::http1::Builder;
use hyper::{body::Incoming, header::CONTENT_TYPE, Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use pin_project::pin_project;
use shuttle_runtime::Error;
use tokio::net::TcpListener;
use tonic::service::Routes;
use tonic::transport::Server;
use tonic::{body::BoxBody, Status};
use tonic_web::{GrpcWebLayer, GrpcWebService};
use tower::Service;
use tower_cookies::CookieManager;
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

pub type ShuttleGrpcWeb = Result<GrpcRestService, Error>;
type Repo = crate::dao::turso::Turso;
type RepoService = crate::svc::Service<Repo>;
type SessionStore = crate::svc::session::SessionStore<Repo>;
type RestService = axum::Router;
type GrpcService = tower_http::trace::Trace<
    GrpcWebService<
        CookieManager<
            SessionManager<
                AuthManager<crate::ctrl::auth::AuthService<Routes>, Backend<RepoService>>,
                SessionStore,
            >,
        >,
    >,
    SharedClassifier<ServerErrorsAsFailures>,
>;

#[derive(Debug, Clone)]
/// The global application state shared between all request handlers.
struct AppState<RS: ResourceService> {
    res_service: Arc<RS>,
}

pub struct GrpcRestService {
    rest: RestService,
    grpc: GrpcService,
}

impl GrpcRestService {
    pub fn new(repo: Repo) -> Self {
        let session_store = SessionStore::new(repo.clone());
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(30)));

        let svc = Arc::new(RepoService::new(repo));
        let backend = Backend::new(svc.clone());
        let auth_manager_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let index_file = ServeFile::new("web/dist/index.html").precompressed_br();
        let axum_router = Router::new()
            .merge(resource::router())
            .layer(auth_manager_layer.clone())
            .route_service("/home", index_file.clone())
            .route_service("/auth", index_file.clone())
            .route_service("/explore", index_file.clone())
            .route_service("/resources", index_file.clone())
            .route_service("/setting", index_file)
            .fallback_service(
                ServeDir::new("web/dist")
                    .precompressed_br()
                    .append_index_html_on_directories(true),
            )
            .layer(TraceLayer::new_for_http());

        let public_path = vec![
            "/memos.api.v1.AuthService/SignIn".to_string(),
            "/memos.api.v1.AuthService/GetAuthStatus".to_string(),
            "/memos.api.v1.MemoService/ListMemos".to_string(),
            "/memos.api.v1.MemoService/ListMemoRelations".to_string(),
            "/memos.api.v1.MemoService/ListMemoResources".to_string(),
            "/memos.api.v1.WorkspaceSettingService/GetWorkspaceSetting".to_string(),
            "/memos.api.v1.WorkspaceService/GetWorkspaceProfile".to_string(),
        ];

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
            .add_service(setting)
            .into_service();

        Self {
            rest: axum_router,
            grpc: tonic_router,
        }
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for GrpcRestService {
    /// Takes the router that is returned by the user in their [shuttle_runtime::main] function
    /// and binds to an address passed in by shuttle.
    async fn bind(mut self, addr: SocketAddr) -> Result<(), Error> {
        let hybrid_service = HybridService {
            rest: self.rest,
            grpc: self.grpc,
        };

        let listener = TcpListener::bind(addr).await?;

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let io = TokioIo::new(stream);
                    let service = TowerToHyperService::new(hybrid_service.clone());
                    tokio::task::spawn(async move {
                        if let Err(err) = Builder::new().serve_connection(io, service).await {
                            error!("Failed to serve connection: {err}");
                        }
                    });
                }
                Err(err) => error!("Failed to accept: {err}"),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct HybridService<Rest, Grpc> {
    rest: Rest,
    grpc: Grpc,
}

impl<Rest, Grpc, WebBody, GrpcBody> Service<Request<Incoming>> for HybridService<Rest, Grpc>
where
    Rest: Service<Request<Incoming>, Response = Response<WebBody>>,
    Grpc: Service<Request<BoxBody>, Response = Response<GrpcBody>>,
    Rest::Error: Into<BoxError>,
    Grpc::Error: Into<BoxError>,
{
    type Response = Response<HybridBody<WebBody, GrpcBody>>;
    type Error = BoxError;
    type Future = HybridFuture<Rest::Future, Grpc::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match (self.rest.poll_ready(cx), self.grpc.poll_ready(cx)) {
            (Poll::Ready(Ok(())), Poll::Ready(Ok(()))) => Poll::Ready(Ok(())),
            (_, Poll::Ready(Err(e))) => Poll::Ready(Err(e.into())),
            (Poll::Ready(Err(e)), _) => Poll::Ready(Err(e.into())),
            (_, Poll::Pending) => Poll::Pending,
            (Poll::Pending, _) => Poll::Pending,
        }
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let content_type = req.headers().get(CONTENT_TYPE).map(|x| x.as_bytes());
        if content_type == Some(b"application/grpc-web+proto")
            || content_type == Some(b"application/grpc-web")
        {
            let req = req.map(|b| {
                b.map_err(|e| Status::from_error(Box::new(e)))
                    .boxed_unsync()
            });
            HybridFuture::Grpc(self.grpc.call(req))
        } else {
            HybridFuture::Rest(self.rest.call(req))
        }
    }
}

#[pin_project(project = HybridBodyProj)]
enum HybridBody<RestBody, GrpcBody> {
    Rest(#[pin] RestBody),
    Grpc(#[pin] GrpcBody),
}

impl<RestBody, GrpcBody> Body for HybridBody<RestBody, GrpcBody>
where
    RestBody: Body + Send + Unpin,
    GrpcBody: Body<Data = RestBody::Data> + Send + Unpin,
    RestBody::Error: std::error::Error + Send + Sync + 'static,
    GrpcBody::Error: std::error::Error + Send + Sync + 'static,
{
    type Data = RestBody::Data;
    type Error = BoxError;

    fn is_end_stream(&self) -> bool {
        match self {
            HybridBody::Rest(b) => b.is_end_stream(),
            HybridBody::Grpc(b) => b.is_end_stream(),
        }
    }

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.project() {
            HybridBodyProj::Rest(b) => b.poll_frame(cx).map_err(|e| e.into()),
            HybridBodyProj::Grpc(b) => b.poll_frame(cx).map_err(|e| e.into()),
        }
    }
}

#[pin_project(project = HybridFutureProj)]
enum HybridFuture<RestFuture, GrpcFuture> {
    Rest(#[pin] RestFuture),
    Grpc(#[pin] GrpcFuture),
}

impl<RestFuture, GrpcFuture, RestBody, GrpcBody, RestError, GrpcError> Future
    for HybridFuture<RestFuture, GrpcFuture>
where
    RestFuture: Future<Output = Result<Response<RestBody>, RestError>>,
    GrpcFuture: Future<Output = Result<Response<GrpcBody>, GrpcError>>,
    RestError: Into<BoxError>,
    GrpcError: Into<BoxError>,
{
    type Output = Result<Response<HybridBody<RestBody, GrpcBody>>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        match self.project() {
            HybridFutureProj::Rest(a) => match a.poll(cx) {
                Poll::Ready(Ok(res)) => Poll::Ready(Ok(res.map(HybridBody::Rest))),
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
