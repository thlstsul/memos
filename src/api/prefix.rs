use snafu::{ensure, ResultExt, Snafu};

pub const INBOX_NAME_PREFIX: &str = "inboxes";
pub const USER_NAME_PREFIX: &str = "users";
pub const WORKSPACE_SETTING_NAME_PREFIX: &str = "settings";
pub const MEMO_NAME_PREFIX: &str = "memos";
pub const RESOURCE_NAME_PREFIX: &str = "resources";
#[allow(dead_code)]
pub const STORAGE_NAME_PREFIX: &str = "storages";
#[allow(dead_code)]
pub const IDENTITY_PROVIDER_NAME_PREFIX: &str = "identityProviders";

#[macro_export]
macro_rules! impl_extract_name {
    ($s:path,$prefix:expr) => {
        impl $crate::api::prefix::ExtractName for $s {
            fn get_name(&self) -> String {
                $crate::api::prefix::get_name_parent_token(self.name.clone(), $prefix)
                    .inspect_err(|e| tracing::error!("{e}"))
                    .unwrap_or(self.name.to_owned())
            }
        }
    };
}

pub trait ExtractName {
    fn get_name(&self) -> String;
    fn get_id(&self) -> Result<i32, Error> {
        self.get_name().parse().context(Id)
    }
}

pub trait FormatName {
    fn get_name(&self) -> String;
}

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

pub fn get_id_parent_token(name: impl AsRef<str>, token: impl AsRef<str>) -> Result<i32, Error> {
    let name = get_name_parent_token(name, token)?;
    name.parse().context(Id)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid request : {name}"), context(suffix(false)))]
    InvalidRequest { name: String },
    #[snafu(display("Invalid prefix in request : {name}"), context(suffix(false)))]
    InvalidPrefix { name: String },

    #[snafu(display("Failed to parse id: {source}"), context(suffix(false)))]
    Id { source: std::num::ParseIntError },
}
