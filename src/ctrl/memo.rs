use actix_identity::Identity;
use actix_web::{
    get,
    web::{Data, Query},
    Responder, Result,
};
use libsql_client::Client;

use crate::api::v1::memo::GetMemoRequest;

#[get("/memo")]
pub async fn get_memo(
    qry: Query<GetMemoRequest>,
    client: Data<Client>,
    _ident: Identity,
) -> Result<impl Responder> {
}
