use axum::{extract::State, response::IntoResponse, response::Result, routing::get, Json, Router};

use crate::{
    api::v1::system::{Host, SystemStatus},
    state::AppState,
    svc::{system::SystemService, user::UserService},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ping", get(ping))
        .route("/status", get(status))
}

async fn ping() -> impl IntoResponse {
    "true"
}

async fn status(state: State<AppState>) -> Result<Json<SystemStatus>> {
    let user_svc = UserService::new(&state);
    let sys_svc = SystemService::new(&state);
    let host: Host = user_svc.host_user().await?.into();
    let mut status: SystemStatus = sys_svc.list_setting().await?.into();
    status.host = host;
    Ok(Json(status))
}
