use crate::{api::prefix, impl_extract_name};

use super::v1::gen::Inbox;

impl_extract_name!(Inbox, prefix::INBOX_NAME_PREFIX);
