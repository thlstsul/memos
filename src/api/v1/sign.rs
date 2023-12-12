use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignRequest {
    pub username: String,
    pub password: String,
}
