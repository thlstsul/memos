use std::sync::Arc;

use actix_identity::Identity;
use actix_web::{
    post,
    web::{Data, Json},
    HttpMessage, HttpRequest, Responder, Result,
};
use libsql_client::Client;

use crate::{pb::memos_api_v1::SignRequest, svc::auth::AuthService};

#[post("/auth/signin")]
pub async fn sign_in(
    req: HttpRequest,
    body: Json<SignRequest>,
    client: Data<Client>,
) -> Result<impl Responder> {
    let sign_svc = AuthService::new(Arc::clone(&client));
    let user = sign_svc
        .sign_in(body.username.clone(), body.password.clone())
        .await?;
    // 暂时不做JWT，因为不打算开发注册
    Identity::login(&req.extensions(), user.id.to_string())?;
    Ok(Json(user))
}
