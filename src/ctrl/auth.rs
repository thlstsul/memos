use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use axum::response::Redirect;
use axum::routing::post;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Result},
};
use axum::{Json, Router};
use axum_login::tower_sessions::{MemoryStore, SessionManager};
use axum_login::{AuthManager, AuthUser, AuthnBackend, UserId};
use hyper::{header, Request, Response};
use libsql_client::Client;
use pin_project_lite::pin_project;
use tower::{Layer, Service};
use tower_cookies::CookieManager;
use tracing::info;

use crate::{
    api::{v1::sign::SignRequest, v2::User},
    svc::user::{Error, UserService},
};

pub fn router() -> Router<Arc<Client>> {
    Router::new()
        .route("/auth/signin", post(signin))
        .route("/auth/signout", post(logout))
}

/// /auth/signin post json
async fn signin(
    mut auth_session: AuthSession,
    Json(creds): Json<SignRequest>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let status_code = StatusCode::UNAUTHORIZED;
            let message = "Incorrect login credentials, please try again";
            return (
                status_code,
                [(header::CONTENT_TYPE, "text/json; charset=utf-8")],
                format!(
                    r#"{{
                    "error": "code={}, message={}",
                    "message": "{}"
                    }}"#,
                    status_code, message, message
                ),
            )
                .into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Json(user).into_response()
}

/// /auth/signout
async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout() {
        Ok(_) => Redirect::to("/").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    svc: UserService,
}

impl Backend {
    pub fn new(svc: UserService) -> Self {
        Self { svc }
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = SignRequest;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = self.svc.sign_in(creds.username, creds.password).await;
        match user {
            Ok(user) => Ok(Some(user)),
            Err(Error::Login { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = self.svc.petch_user(*user_id).await?;
        Ok(Some(user))
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Debug, Clone)]
pub struct AuthLayer {
    auth_manager_layer: axum_login::AuthManagerLayer<Backend, MemoryStore>,
    public_path: Vec<String>,
}

impl AuthLayer {
    pub fn new(
        auth_manager_layer: axum_login::AuthManagerLayer<Backend, MemoryStore>,
        public_path: Vec<String>,
    ) -> Self {
        Self {
            auth_manager_layer,
            public_path,
        }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = CookieManager<SessionManager<AuthManager<AuthService<S>, Backend>, MemoryStore>>;

    fn layer(&self, inner: S) -> Self::Service {
        let auth_service = AuthService {
            inner,
            public_path: self.public_path.clone(),
        };

        self.auth_manager_layer.layer(auth_service)
    }
}

#[derive(Clone, Debug)]
pub struct AuthService<S> {
    pub inner: S,
    pub public_path: Vec<String>,
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for AuthService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default + Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, ResBody>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let path = req.uri().path().to_owned();
        if self.public_path.contains(&path) {
            info!("public path: {path}");
            ResponseFuture::future(self.inner.call(req))
        } else if let Some(AuthSession { user: Some(_), .. }) =
            req.extensions().get::<AuthSession>()
        {
            ResponseFuture::future(self.inner.call(req))
        } else {
            let res = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(ResBody::default())
                .ok();
            ResponseFuture::unauthorized(res)
        }
    }
}

pin_project! {
    /// Response future for [`ValidateRequestHeader`].
    pub struct ResponseFuture<F, B> {
        #[pin]
        kind: Kind<F, B>,
    }
}

impl<F, B> ResponseFuture<F, B> {
    fn future(future: F) -> Self {
        Self {
            kind: Kind::Future { future },
        }
    }

    fn unauthorized(response: Option<Response<B>>) -> Self {
        Self {
            kind: Kind::Error { response },
        }
    }
}

pin_project! {
    #[project = KindProj]
    enum Kind<F, B> {
        Future {
            #[pin]
            future: F,
        },
        Error {
            response: Option<Response<B>>,
        },
    }
}

impl<F, B, E> Future for ResponseFuture<F, B>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().kind.project() {
            KindProj::Future { future } => future.poll(cx),
            KindProj::Error { response } => {
                let response = response.take().expect("future polled after completion");
                Poll::Ready(Ok(response))
            }
        }
    }
}
