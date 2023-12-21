use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct SignRequest {
    pub username: String,
    pub password: String,
}
