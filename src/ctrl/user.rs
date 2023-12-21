use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Result,
    routing::get,
    Json, Router,
};
use libsql_client::Client;

use crate::{
    api::{v1::user::UserInfo, v2::User},
    svc::user::UserService,
};

use super::auth::AuthSession;

pub fn router() -> Router<Arc<Client>> {
    Router::new()
        .route("/user/:id", get(user_detail))
        .route("/user/me", get(me))
}

async fn user_detail(Path(id): Path<i32>, client: State<Arc<Client>>) -> Result<Json<User>> {
    let svc = UserService::new(&client);
    let user = svc.petch_user(id).await?;
    Ok(Json(user))
}

async fn me(session: AuthSession, client: State<Arc<Client>>) -> Result<Json<UserInfo>> {
    let svc = UserService::new(&client);
    let id = session.user.unwrap().id;
    let (user, settings) = tokio::try_join!(svc.petch_user(id), svc.find_setting(id))?;
    Ok(Json(UserInfo { user, settings }))
}
