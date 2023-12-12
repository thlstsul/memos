use actix_identity::Identity;
use actix_web::{
    http::StatusCode,
    post,
    web::{Data, Json, Redirect},
    HttpMessage, HttpRequest, Responder, Result,
};
use libsql_client::Client;

use crate::{api::v1::sign::SignRequest, svc::auth::AuthService};

#[post("/auth/signin")]
pub async fn signin(
    req: HttpRequest,
    body: Json<SignRequest>,
    client: Data<Client>,
) -> Result<impl Responder> {
    let sign_svc = AuthService::new(&client);
    let user = sign_svc
        .sign_in(body.username.clone(), body.password.clone())
        .await?;
    // 暂时不做JWT，因为不打算开发注册
    Identity::login(&req.extensions(), user.id.to_string())?;
    Ok(Json(user))
}

#[post("/auth/signout")]
pub async fn signout(ident: Identity) -> impl Responder {
    ident.logout();

    Redirect::to("/").using_status_code(StatusCode::FOUND)
}
