use actix_files::NamedFile;
use actix_web::{
    cookie::Key,
    get,
    web::{self, Data, ServiceConfig},
    HttpResponse, Responder,
};
use ctrl::root;
use libsql_client::client::Client;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_secrets::SecretStore;

mod api;
mod ctrl;
mod dao;
mod svc;

#[get(r"/{filename:.*\..*}")]
async fn static_file(filename: web::Path<String>) -> impl Responder {
    let path = format!("{}/{}", "web/dist", filename);
    NamedFile::open_async(path).await
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_turso::Turso(addr = "{secrets.TURSO_URL}", token = "{secrets.TURSO_TOKEN}")]
    client: Client,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let key = Key::generate();
    let client = Data::new(client);
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(client.clone())
            .service(web::redirect("/", "/index.html"))
            .service(static_file)
            .service(root(key))
            .default_service(web::to(HttpResponse::NotFound));
    };

    Ok(config.into())
}
