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
    pub fn get_user(client: &Arc<Client>) -> UserServiceServer<UserService> {
        let user = UserService::new(client);
        UserServiceServer::new(user)
    }

    pub fn get_tag(client: &Arc<Client>) -> TagServiceServer<TagService> {
        let tag = TagService::new(client);
        TagServiceServer::new(tag)
    }
}
