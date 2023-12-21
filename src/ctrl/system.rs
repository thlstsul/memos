use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, response::Result, routing::get, Json, Router};
use libsql_client::Client;

use crate::{
    api::v1::system::{Host, SystemStatus},
    svc::{system::SystemService, user::UserService},
};

pub fn router() -> Router<Arc<Client>> {
    Router::new()
        .route("/ping", get(ping))
        .route("/status", get(status))
}

async fn ping() -> impl IntoResponse {
    "true"
}

async fn status(client: State<Arc<Client>>) -> Result<Json<SystemStatus>> {
    let user_svc = UserService::new(&client);
    let sys_svc = SystemService::new(&client);
    let host: Host = user_svc.host_user().await?.into();
    let mut status: SystemStatus = sys_svc.list_setting().await?.into();
    status.host = host;
    Ok(Json(status))
}
