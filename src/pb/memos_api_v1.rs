use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SignRequest {
    username: String,
    password: String,
}
