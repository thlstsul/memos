use crate::util::{ast, get_name_parent_token};

use super::{
    v2::{
        CreateMemoResponse, GetMemoResponse, ListMemosRequest, ListMemosResponse, Memo, PageToken,
        RowStatus, UpdateMemoRequest, UpdateMemoResponse, Visibility,
    },
    USER_NAME_PREFIX,
};
use snafu::{ResultExt, Snafu};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, BinOp, Expr,
};

#[derive(Debug, Default)]
pub struct FindMemo {
    pub id: Option<i32>,
    pub name: Option<String>,

    // Temp fields
    pub creator: Option<String>,

    // Standard fields
    pub creator_id: Option<i32>,
    pub row_status: Option<RowStatus>,
    pub display_time_after: Option<i64>,
    pub display_time_before: Option<i64>,

    // Domain specific fields
    pub pinned: bool,
    pub content_search: Vec<String>,
    pub visibility_list: Vec<Visibility>,
    pub exclude_content: bool,

    // Pagination
    pub page_token: Option<PageToken>,
    pub order_by_updated_ts: bool,
    pub order_by_pinned: bool,
}

pub struct CreateMemo {
    pub creator_id: i32,
    pub resource_name: String,
    pub content: String,
    pub visibility: Visibility,
}

#[derive(Debug, Default)]
pub struct UpdateMemo {
    pub id: i32,
    pub creator_id: i32,
    pub content: Option<String>,
    pub visibility: Option<Visibility>,
    pub row_status: Option<RowStatus>,
    pub pinned: Option<bool>,
}

#[derive(Debug, Default)]
pub struct Filter {
    pub content_search: Option<Vec<String>>,
    pub visibilities: Option<Vec<Visibility>>,
    pub order_by_pinned: Option<bool>,
    pub display_time_before: Option<i64>,
    pub display_time_after: Option<i64>,
    pub creator: Option<String>,
    pub row_status: Option<RowStatus>,
}

impl Parse for Filter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut binary = Expr::parse(input)?;

        let content_search: Expr = parse_quote!(content_search);
        let visibilities: Expr = parse_quote!(visibilities);
        let order_by_pinned: Expr = parse_quote!(order_by_pinned);
        let display_time_before: Expr = parse_quote!(display_time_before);
        let display_time_after: Expr = parse_quote!(display_time_after);
        let creator: Expr = parse_quote!(creator);
        let row_status: Expr = parse_quote!(row_status);

        let mut filter = Filter::default();

        while let Expr::Binary(ref bin) = binary {
            let mut get_value = |ident: Expr, lit: Expr| {
                if ident == creator {
                    filter.creator = ast::get_string(lit);
                } else if ident == row_status {
                    filter.row_status = ast::get_row_status(lit);
                } else if ident == visibilities {
                    filter.visibilities = ast::get_visibilities(lit);
                } else if ident == order_by_pinned {
                    filter.order_by_pinned = ast::get_bool(lit);
                } else if ident == content_search {
                    filter.content_search = ast::get_string_list(lit);
                } else if ident == display_time_before {
                    filter.display_time_before = ast::get_i64(lit);
                } else if ident == display_time_after {
                    filter.display_time_after = ast::get_i64(lit);
                }
            };

            let left = *bin.left.clone();
            let right = *bin.right.clone();

            if matches!(bin.op, BinOp::And(_)) {
                if let Expr::Binary(bin) = right {
                    let ident = *bin.left;
                    let lit = *bin.right;
                    get_value(ident, lit);
                }
                binary = left;
            } else {
                get_value(left, right);
                break;
            }
        }

        Ok(filter)
    }
}

impl TryInto<FindMemo> for &ListMemosRequest {
    type Error = Error;

    fn try_into(self) -> Result<FindMemo, Self::Error> {
        let filter = &self.filter.replace('\'', "\"");
        let filter = syn::parse_str::<Filter>(filter).context(FilterDecode)?;
        let creator = filter
            .creator
            .map(|s| get_name_parent_token(s, USER_NAME_PREFIX))
            .transpose()
            .context(InvalidUsername)?;
        let page_token = if !self.page_token.is_empty() {
            serde_json::from_str(&self.page_token).context(PageTokenDecode)?
        } else {
            PageToken {
                limit: self.page_size,
                offset: 0,
            }
        };
        Ok(FindMemo {
            id: None,
            name: None,
            creator,
            creator_id: None,
            row_status: filter.row_status,
            display_time_after: filter.display_time_after,
            display_time_before: filter.display_time_before,
            pinned: false,
            content_search: filter.content_search.unwrap_or_default(),
            visibility_list: filter.visibilities.unwrap_or_default(),
            exclude_content: false,
            page_token: Some(page_token),
            order_by_updated_ts: false,
            order_by_pinned: filter.order_by_pinned.unwrap_or_default(),
        })
    }
}

impl From<Memo> for CreateMemoResponse {
    fn from(val: Memo) -> Self {
        let mut memo = val;
        convert_memo(&mut memo);
        CreateMemoResponse { memo: Some(memo) }
    }
}

impl From<Vec<Memo>> for ListMemosResponse {
    fn from(val: Vec<Memo>) -> Self {
        let mut memos = val;
        for memo in memos.iter_mut() {
            convert_memo(memo)
        }
        ListMemosResponse {
            memos,
            ..Default::default()
        }
    }
}

impl From<Option<Memo>> for UpdateMemoResponse {
    fn from(val: Option<Memo>) -> Self {
        UpdateMemoResponse {
            memo: if let Some(mut memo) = val {
                convert_memo(&mut memo);
                Some(memo)
            } else {
                None
            },
        }
    }
}

impl From<Option<Memo>> for GetMemoResponse {
    fn from(val: Option<Memo>) -> Self {
        GetMemoResponse {
            memo: if let Some(mut memo) = val {
                convert_memo(&mut memo);
                Some(memo)
            } else {
                None
            },
        }
    }
}

fn convert_memo(memo: &mut Memo) {
    memo.creator = format!("{}/{}", USER_NAME_PREFIX, memo.creator);
}

impl From<&UpdateMemoRequest> for UpdateMemo {
    fn from(value: &UpdateMemoRequest) -> Self {
        let mut update = UpdateMemo::default();

        if let UpdateMemoRequest {
            memo: Some(memo),
            update_mask: Some(field_mask),
        } = value
        {
            update.id = memo.id;
            for path in &field_mask.paths {
                match path.as_str() {
                    "content" => update.content = Some(memo.content.clone()),
                    "visibility" => update.visibility = Visibility::try_from(memo.visibility).ok(),
                    "row_status" => update.row_status = RowStatus::try_from(memo.row_status).ok(),
                    "pinned" => update.pinned = Some(memo.pinned),
                    _ => (),
                }
            }
        }
        update
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid username : {source}"), context(suffix(false)))]
    InvalidUsername { source: crate::util::Error },
    #[snafu(display("Failed to decode filter : {source}"), context(suffix(false)))]
    FilterDecode { source: syn::Error },
    #[snafu(
        display("Failed to decode page token : {source}"),
        context(suffix(false))
    )]
    PageTokenDecode { source: serde_json::Error },
}

mod test {
    #[test]
    fn parse_filter() {
        use crate::api::v2::RowStatus;
        let filter =
        r#"visibilities == ['PUBLIC'] && row_status == "NORMAL" && creator == "users/THELOSTSOUL" && order_by_pinned == true && display_time_before == 123"#
            .replace("'", "\"");
        let filter = syn::parse_str::<crate::api::memo::Filter>(&filter).unwrap();
        assert_eq!(filter.row_status, Some(RowStatus::Active));
        assert_eq!(
            filter.visibilities,
            Some(vec![crate::api::v2::Visibility::Public])
        );
        assert_eq!(filter.creator, Some("users/THELOSTSOUL".to_owned()));
        assert_eq!(filter.order_by_pinned, Some(true));
        assert_eq!(filter.display_time_before, Some(123_i64));
    }
}
