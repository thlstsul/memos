use actix_web::{
    get,
    web::{Data, Json},
    Responder, Result,
};
use libsql_client::Client;

use crate::{
    api::v1::system::{Host, SystemStatus},
    svc::{system::SystemService, user::UserService},
};

#[get("/ping")]
pub async fn ping() -> impl Responder {
    "true"
}

#[get("/status")]
pub async fn status(client: Data<Client>) -> Result<impl Responder> {
    let user_svc = UserService::new(&client);
    let sys_svc = SystemService::new(&client);
    let host: Host = user_svc.host_user().await?.into();
    let mut status: SystemStatus = sys_svc.list_setting().await?.into();
    status.host = host;
    Ok(Json(status))
}
