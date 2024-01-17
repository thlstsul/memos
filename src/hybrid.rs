/// TODO tonic 未升级hyper 1.0，待其升级
/// fork from https://github.com/snoyberg/tonic-example
/// fork from https://github.com/tokio-rs/axum/examples/rest-grpc-multiplex
/// axum::Router impl Service 可以省一层
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Poll;

use hyper::header::CONTENT_TYPE;
use hyper::HeaderMap;
use hyper::{body::HttpBody, Body, Request, Response};
use pin_project::pin_project;
use shuttle_runtime::{CustomError, Error};
use tonic_web::GrpcWebLayer;
use tower::layer::util::{Identity, Stack};
use tower::{BoxError, Service};

use crate::ctrl::auth::AuthLayer;

pub struct GrpcWebService {
    pub axum_router: axum::Router,
    pub tonic_router:
        tonic::transport::server::Router<Stack<AuthLayer, Stack<GrpcWebLayer, Identity>>>,
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
