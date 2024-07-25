use tracing::error;

use crate::{
    api::{
        prefix::{self, get_name_parent_token},
        to_timestamp,
    },
    impl_extract_name,
};

use super::{
    prefix::FormatName,
    v1::gen::{DeleteResourceRequest, GetResourceRequest, Resource},
};

impl_extract_name!(GetResourceRequest, prefix::RESOURCE_NAME_PREFIX);
impl_extract_name!(DeleteResourceRequest, prefix::RESOURCE_NAME_PREFIX);
impl_extract_name!(Resource, prefix::RESOURCE_NAME_PREFIX);

impl Resource {
    pub fn get_memo(&self) -> Option<i32> {
        self.memo
            .as_ref()
            .and_then(|m| {
                get_name_parent_token(m, prefix::MEMO_NAME_PREFIX)
                    .inspect_err(|e| error!("{e}"))
                    .ok()
            })
            .and_then(|s| s.parse().inspect_err(|e| error!("{e}")).ok())
    }
}

impl From<crate::model::resource::Resource> for Resource {
    fn from(value: crate::model::resource::Resource) -> Self {
        Self {
            name: value.get_name(),
            uid: value.uid,
            create_time: to_timestamp(value.created_ts),
            filename: value.filename,
            content: value.blob,
            external_link: value.reference,
            r#type: value.r#type,
            size: value.size as i64,
            memo: value
                .memo_id
                .map(|id| format!("{}/{}", prefix::MEMO_NAME_PREFIX, id)),
        }
    }
}

impl FormatName for crate::model::resource::Resource {
    fn get_name(&self) -> String {
        format!("{}/{}", prefix::RESOURCE_NAME_PREFIX, self.id)
    }
}
