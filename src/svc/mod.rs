use std::sync::Arc;

use libsql_client::Client;
use tonic_web::CorsGrpcWeb;

use crate::api::v2::{
    tag_service_server::TagServiceServer, user_service_server::UserServiceServer,
};

use self::{tag::TagService, user::UserService};

pub mod auth;
pub mod system;
pub mod tag;
pub mod user;

pub struct ServiceFactory;

impl ServiceFactory {
    pub fn get_user(client: &Arc<Client>) -> CorsGrpcWeb<UserServiceServer<UserService>> {
        let user = UserService::new(client);
        let user = UserServiceServer::new(user);
        tonic_web::enable(user)
    }

    pub fn get_tag(client: &Arc<Client>) -> CorsGrpcWeb<TagServiceServer<TagService>> {
        let tag = TagService::new(client);
        let tag = TagServiceServer::new(tag);
        tonic_web::enable(tag)
    }
}
