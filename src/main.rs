#![allow(unused_variables)]
use std::sync::Arc;

use axum::Router;
use hybrid::ShuttleGrpcWeb;
use libsql_client::client::Client;
use shuttle_secrets::SecretStore;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::services::ServeDir;

use crate::svc::ServiceFactory;

mod api;
mod ctrl;
mod dao;
mod hybrid;
mod svc;

#[shuttle_runtime::main]
async fn grpc_web(
    #[shuttle_turso::Turso(addr = "{secrets.TURSO_URL}", token = "{secrets.TURSO_TOKEN}")]
    client: Client,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> ShuttleGrpcWeb {
    // TODO

    // let config = move |cfg: &mut ServiceConfig| {
    //     cfg.app_data(client.clone())
    //         .app_data(Data::new(awc::Client::default()))
    //         .app_data(Data::new(forward_url))
    //         .service(index)
    //         .service(static_file)
    //         .service(root(key));
    // };

    let axum_router = Router::new()
        .with_state(client)
        .nest("/api/v1", router)
        .route_service(
            "/",
            ServeDir::new("web/dist").append_index_html_on_directories(true),
        );

    let client_arc = Arc::new(client);
    let user = ServiceFactory::get_user(&client_arc);
    let tag = ServiceFactory::get_tag(&client_arc);

    let tonic_router = Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .add_service(user)
        .add_service(tag);

    Ok(ShuttleGrpcWeb {
        axum_router,
        tonic_router,
    })
}
