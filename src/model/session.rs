use axum_login::tower_sessions;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Session {
    pub id: String,
    pub data: Vec<u8>,
    pub expiry_date: i64,
}

impl TryInto<Session> for &tower_sessions::Session {
    type Error = rmp_serde::encode::Error;

    fn try_into(self) -> Result<Session, Self::Error> {
        let data = rmp_serde::to_vec(self)?;
        Ok(Session {
            id: self.id().to_string(),
            data,
            expiry_date: self.expiry_date().unix_timestamp(),
        })
    }
}
