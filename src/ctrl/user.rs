use actix_identity::Identity;
use actix_web::{
    get,
    web::{Data, Json, Path},
    Responder, Result,
};
use libsql_client::Client;

use crate::{api::memos_api_v1::user::UserResponse, svc::user::UserService};

#[get("/user/{id}")]
pub async fn user_detail(
    id: Path<i32>,
    client: Data<Client>,
    _ident: Identity,
) -> Result<impl Responder> {
    let svc = UserService::new(&client);
    let user = svc.petch_user(id.into_inner()).await?;
    Ok(Json(user))
}

#[get("/user/me")]
pub async fn me(client: Data<Client>, ident: Identity) -> Result<impl Responder> {
    let svc = UserService::new(&client);
    let id = ident.id()?.parse().unwrap_or_default();
    let user = svc.petch_user(id).await?;
    let settings = svc.find_setting(id).await?;
    Ok(Json(UserResponse { user, settings }))
}
