use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SignResponse {}
