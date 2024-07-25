#![allow(unused_variables)]
#![allow(clippy::enum_variant_names)]

use hybrid::{GrpcWebService, ShuttleGrpcWeb};
use libsql::Database;
use shuttle_runtime::SecretStore;

use crate::dao::turso::Turso;

mod google {
    pub mod api {
        include!("api/v1/google.api.rs");
    }
}

mod api;
mod ctrl;
mod dao;
mod hybrid;
mod model;
mod svc;
mod util;

#[shuttle_runtime::main]
async fn grpc_web(
    #[shuttle_turso::Turso(addr = "{secrets.TURSO_URL}", token = "{secrets.TURSO_TOKEN}")]
    repo: Database,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleGrpcWeb {
    let repo = Turso::new(repo);

    Ok(GrpcWebService::new(repo))
}
