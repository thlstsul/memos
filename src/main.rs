#![allow(unused_variables)]

use actix_files::NamedFile;
use actix_web::{
    cookie::Key,
    get,
    web::{self, Data, ServiceConfig},
    Responder,
};
use ctrl::root;
use libsql_client::client::Client;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_secrets::SecretStore;
use tokio::net::TcpListener;
use tonic::transport::{server::TcpIncoming, Server};
use url::Url;

use crate::{
    api::v2::user_service_server::UserServiceServer,
    svc::{user::UserService, ServiceFactory},
};

mod api;
mod ctrl;
mod dao;
mod svc;

#[get(r"/{filename:.*\.html|.*\.js|.*\.json|.*\.css|.*\.woff2|.*\.woff|.*\.ttf|.*\.png|.*\.webp}")]
async fn static_file(filename: web::Path<String>) -> impl Responder {
    let path = format!("{}/{}", "web/dist", filename);
    NamedFile::open_async(path).await
}

#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("web/dist/index.html").await
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_turso::Turso(addr = "{secrets.TURSO_URL}", token = "{secrets.TURSO_TOKEN}")]
    client: Client,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let key = Key::generate();
    let client = Data::new(client);
    let clinet_copy = client.clone();
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let forward_socket_addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let user = ServiceFactory::get_user(&clinet_copy);
        let tag = ServiceFactory::get_tag(&clinet_copy);

        let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
        Server::builder()
            .accept_http1(true)
            .add_service(user)
            .add_service(tag)
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    let forward_url = format!("http://{forward_socket_addr}");
    let forward_url = Url::parse(&forward_url).unwrap();

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(client.clone())
            .app_data(Data::new(awc::Client::default()))
            .app_data(Data::new(forward_url))
            .service(index)
            .service(static_file)
            .service(root(key));
    };

    Ok(config.into())
}
