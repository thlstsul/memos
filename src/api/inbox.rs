use crate::{api::prefix, impl_extract_name};

use super::v1::generate::Inbox;

impl_extract_name!(Inbox, prefix::INBOX_NAME_PREFIX);
