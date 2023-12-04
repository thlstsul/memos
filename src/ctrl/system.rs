use actix_web::{get, web::Data, Responder, Result};
use libsql_client::Client;

use crate::{api::memos_api_v1::system::Host, svc::user::UserService};

#[get("/ping")]
pub async fn ping() -> impl Responder {
    "true"
}

#[get("/status")]
pub async fn status(client: Data<Client>) -> Result<impl Responder> {
    let userSvc = UserService::new(&client);
    let host: Host = userSvc.host_user().await?.into();
    todo!()
}
