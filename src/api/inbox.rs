use super::{get_name_parent_token, v2::Inbox, INBOX_NAME_PREFIX};
use snafu::{ResultExt, Snafu};

impl Inbox {
    pub fn get_id(&self) -> Result<i32, Error> {
        get_name_parent_token(self.name.clone(), INBOX_NAME_PREFIX)
            .context(InvalidRequest)?
            .parse()
            .context(InvalidInboxId {
                name: self.name.clone(),
            })
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid request: {source}"), context(suffix(false)))]
    InvalidRequest { source: super::Error },
    #[snafu(display("Invalid inbox id: {name}, {source}"), context(suffix(false)))]
    InvalidInboxId {
        name: String,
        source: std::num::ParseIntError,
    },
}
