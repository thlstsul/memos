use nanoid::{alphabet, nanoid};
use snafu::{ensure, Snafu};

pub mod ast;

pub fn get_name_parent_token(
    name: impl AsRef<str>,
    token: impl AsRef<str>,
) -> Result<String, Error> {
    let name = name.as_ref();
    let token = token.as_ref();
    let parts: Vec<_> = name.split('/').collect();
    ensure!(parts.len() == 2, InvalidRequest { name });
    ensure!(token == parts[0], InvalidPrefix { name });
    Ok(parts[1].to_owned())
}

pub fn uuid() -> String {
    nanoid!(16, &alphabet::SAFE)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid request : {name}"), context(suffix(false)))]
    InvalidRequest { name: String },
    #[snafu(display("Invalid prefix in request : {name}"), context(suffix(false)))]
    InvalidPrefix { name: String },
}
