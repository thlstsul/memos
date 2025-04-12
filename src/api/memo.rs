use serde::{Deserialize, Deserializer, Serialize, Serializer};
use snafu::{ResultExt, Snafu};

use crate::{
    api::{prefix, to_timestamp},
    impl_extract_name,
    model::memo::{CreateMemo, FindMemo, FindMemoPayload, SearchMemosFilter, UpdateMemo},
    util::{self, md},
};

use super::v1::{
    gen::Direction,
    r#gen::{memo::Property, CreateMemoRequest, State},
};
use super::{
    prefix::{get_id_parent_token, ExtractName, FormatName},
    v1::gen::{
        DeleteMemoRequest, GetMemoRequest, ListMemosRequest, Memo, PageToken,
        SetMemoResourcesRequest, UpdateMemoRequest, Visibility,
    },
};

impl_extract_name!(Memo, prefix::MEMO_NAME_PREFIX);
impl_extract_name!(GetMemoRequest, prefix::MEMO_NAME_PREFIX);
impl_extract_name!(DeleteMemoRequest, prefix::MEMO_NAME_PREFIX);
impl_extract_name!(SetMemoResourcesRequest, prefix::MEMO_NAME_PREFIX);

impl TryInto<CreateMemo> for &CreateMemoRequest {
    type Error = Error;

    fn try_into(self) -> Result<CreateMemo, Self::Error> {
        let memo = self.memo.as_ref().ok_or(Error::MemoDataLoss)?;
        let content = memo.content.clone();
        let payload = md::get_memo_property(&content);

        Ok(CreateMemo {
            creator_id: 0,
            visibility: memo.visibility(),
            content,
            payload,
            uid: util::uuid(),
        })
    }
}

impl TryInto<FindMemo> for &ListMemosRequest {
    type Error = Error;

    fn try_into(self) -> Result<FindMemo, Self::Error> {
        let filter = if !self.old_filter.is_empty() {
            let old_filter = &self.old_filter.replace('\'', "\"");
            syn::parse_str::<SearchMemosFilter>(old_filter).context(FilterDecode)?
        } else {
            SearchMemosFilter::default()
        };

        let creator_id = filter
            .creator
            .map(|s| get_id_parent_token(s, prefix::USER_NAME_PREFIX))
            .transpose()
            .context(InvalidUsername)?;

        let owner_id = get_id_parent_token(self.parent.clone(), prefix::USER_NAME_PREFIX).ok();
        let page_token = if !self.page_token.is_empty() {
            serde_json::from_str(&self.page_token).context(PageTokenDecode)?
        } else {
            PageToken {
                limit: self.page_size,
                offset: 0,
            }
        };

        let mut direction = Direction::try_from(self.direction).unwrap_or_default();
        if direction == Direction::Unspecified {
            direction = Direction::Desc;
        }
        Ok(FindMemo {
            creator_id: creator_id.or(owner_id),
            state: State::try_from(self.state).ok().or(filter.state),
            content_search: filter.content_search.unwrap_or_default(),
            visibility_list: filter.visibilities.unwrap_or_default(),
            exclude_content: false,
            page_token: Some(page_token),
            //  默认使用更新时间过滤
            order_by_updated_ts: true,
            updated_ts_after: filter.display_time_after,
            updated_ts_before: filter.display_time_before,
            payload_find: if filter.tag_search.is_some()
                || filter.has_code.is_some()
                || filter.has_link.is_some()
                || filter.has_task_list.is_some()
                || filter.has_incomplete_tasks.is_some()
            {
                Some(FindMemoPayload {
                    raw: None,
                    tags: filter.tag_search,
                    has_link: filter.has_link.unwrap_or_default(),
                    has_task_list: filter.has_task_list.unwrap_or_default(),
                    has_code: filter.has_code.unwrap_or_default(),
                    has_incomplete_tasks: filter.has_incomplete_tasks.unwrap_or_default(),
                })
            } else {
                None
            },
            exclude_comments: true,
            only_payload: false,
            sort: self.sort.clone(),
            direction,
            filter: self.filter.clone(),
            ..Default::default()
        })
    }
}

impl From<&UpdateMemoRequest> for UpdateMemo {
    fn from(value: &UpdateMemoRequest) -> Self {
        let mut update = UpdateMemo::default();

        if let UpdateMemoRequest {
            memo: Some(memo),
            update_mask: Some(field_mask),
        } = value
        {
            update.id = memo.get_id().unwrap_or_default();
            for path in &field_mask.paths {
                match path.as_str() {
                    "content" => {
                        let content = memo.content.clone();
                        update.payload = Some(md::get_memo_property(&content));
                        update.content = Some(content);
                    }
                    "visibility" => update.visibility = Visibility::try_from(memo.visibility).ok(),
                    "state" => update.state = State::try_from(memo.state).ok(),
                    "pinned" => update.pinned = Some(memo.pinned),
                    _ => (),
                }
            }
        }
        update
    }
}

impl From<crate::model::memo::Memo> for Memo {
    fn from(value: crate::model::memo::Memo) -> Self {
        let name = value.get_name();
        let content = value.content;
        let snippet = format!("{}...", content.get(0..99).unwrap_or(""));
        let nodes = md::parse_document(&content);
        #[allow(deprecated)]
        Self {
            name,
            state: value.state as i32,
            creator: format!("{}/{}", prefix::USER_NAME_PREFIX, value.creator_id),
            create_time: to_timestamp(value.created_ts),
            update_time: to_timestamp(value.updated_ts),
            display_time: to_timestamp(value.updated_ts),
            content,
            nodes,
            visibility: value.visibility as i32,
            tags: value.payload.tags,
            pinned: value.pinned,
            resources: vec![],
            relations: vec![],
            reactions: vec![],
            property: value.payload.property.map(|p| p.into()),
            parent: None,
            snippet,
            location: None,
        }
    }
}

impl FormatName for crate::model::memo::Memo {
    fn get_name(&self) -> String {
        format!("{}/{}", prefix::MEMO_NAME_PREFIX, self.id)
    }
}

impl From<crate::model::gen::memo_payload::Property> for Property {
    fn from(value: crate::model::gen::memo_payload::Property) -> Self {
        Self {
            has_link: value.has_link,
            has_task_list: value.has_task_list,
            has_code: value.has_code,
            has_incomplete_tasks: value.has_incomplete_tasks,
        }
    }
}

impl Serialize for Visibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let visibility = self.as_str_name();
        serializer.serialize_str(visibility)
    }
}

impl<'de> Deserialize<'de> for Visibility {
    fn deserialize<D>(deserializer: D) -> Result<Visibility, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visibility = String::deserialize(deserializer)?;
        let visibility = Visibility::from_str_name(&visibility).unwrap_or_default();
        Ok(visibility)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid username : {source}"), context(suffix(false)))]
    InvalidUsername { source: prefix::Error },
    #[snafu(display("Failed to decode filter : {source}"), context(suffix(false)))]
    FilterDecode { source: syn::Error },
    #[snafu(
        display("Failed to decode page token : {source}"),
        context(suffix(false))
    )]
    PageTokenDecode { source: serde_json::Error },
    #[snafu(display("Memo data loss"), context(suffix(false)))]
    MemoDataLoss,
}
