use crate::util::{
    ast::{self, parse_document},
    get_name_parent_token,
};

use super::{
    v2::{
        CreateMemoResponse, ListMemosRequest, ListMemosResponse, Memo, RowStatus,
        UpdateMemoRequest, UpdateMemoResponse, Visibility,
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

    // Temp fields
    pub creator: Option<String>,

    // Standard fields
    pub creator_id: Option<i32>,
    pub row_status: Option<String>,
    pub created_ts_after: Option<i64>,
    pub created_ts_before: Option<i64>,

    // Domain specific fields
    pub pinned: bool,
    pub content_search: Vec<String>,
    pub visibility_list: Vec<Visibility>,
    pub exclude_content: bool,

    // Pagination
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub order_by_updated_ts: bool,
    pub order_by_pinned: bool,
}

pub struct CreateMemo {
    pub creator_id: i32,
    pub content: String,
    pub visibility: Visibility,
}

#[derive(Debug, Default)]
pub struct UpdateMemo {
    pub id: i32,
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
    pub created_ts_before: Option<i64>,
    pub created_ts_after: Option<i64>,
    pub creator: Option<String>,
    // TODO wait api fix
    pub row_status: Option<String>,
}

impl Parse for Filter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut binary = Expr::parse(input)?;

        let content_search: Expr = parse_quote!(content_search);
        let visibilities: Expr = parse_quote!(visibilities);
        let order_by_pinned: Expr = parse_quote!(order_by_pinned);
        let created_ts_before: Expr = parse_quote!(created_ts_before);
        let created_ts_after: Expr = parse_quote!(created_ts_after);
        let creator: Expr = parse_quote!(creator);
        let row_status: Expr = parse_quote!(row_status);

        let mut filter = Filter::default();

        while let Expr::Binary(ref bin) = binary {
            let mut get_value = |ident: Expr, lit: Expr| {
                if ident == creator {
                    filter.creator = ast::get_string(lit);
                } else if ident == row_status {
                    filter.row_status = ast::get_string(lit);
                } else if ident == visibilities {
                    filter.visibilities = ast::get_visibilities(lit);
                } else if ident == order_by_pinned {
                    filter.order_by_pinned = ast::get_bool(lit);
                } else if ident == content_search {
                    filter.content_search = ast::get_string_list(lit);
                } else if ident == created_ts_before {
                    filter.created_ts_before = ast::get_i64(lit);
                } else if ident == created_ts_after {
                    filter.created_ts_after = ast::get_i64(lit);
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
        let filter = &self.filter.replace("'", "\"");
        let filter = syn::parse_str::<Filter>(&filter).context(FilterDecodeFailed)?;
        let creator = filter
            .creator
            .map(|s| get_name_parent_token(s, USER_NAME_PREFIX))
            .transpose()
            .context(InvalidUsername)?;
        Ok(FindMemo {
            id: None,
            creator,
            creator_id: None,
            row_status: filter.row_status,
            created_ts_after: filter.created_ts_after,
            created_ts_before: filter.created_ts_before,
            pinned: false,
            content_search: filter.content_search.unwrap_or_default(),
            visibility_list: filter.visibilities.unwrap_or_default(),
            exclude_content: false,
            limit: Some(self.limit),
            offset: Some(self.offset),
            order_by_updated_ts: false,
            order_by_pinned: filter.order_by_pinned.unwrap_or_default(),
        })
    }
}

impl Into<CreateMemoResponse> for Memo {
    fn into(self) -> CreateMemoResponse {
        let mut memo = self;
        convert_memo(&mut memo);
        CreateMemoResponse { memo: Some(memo) }
    }
}

impl Into<ListMemosResponse> for Vec<Memo> {
    fn into(self) -> ListMemosResponse {
        let mut memos = self;
        for memo in memos.iter_mut() {
            convert_memo(memo)
        }
        ListMemosResponse { memos }
    }
}

impl Into<UpdateMemoResponse> for Option<Memo> {
    fn into(self) -> UpdateMemoResponse {
        UpdateMemoResponse {
            memo: if let Some(mut memo) = self {
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
    memo.nodes = parse_document(&memo.content);
}

impl From<&UpdateMemoRequest> for UpdateMemo {
    fn from(value: &UpdateMemoRequest) -> Self {
        let mut update = UpdateMemo {
            id: value.id,
            ..Default::default()
        };

        if let UpdateMemoRequest {
            id,
            memo: Some(memo),
            update_mask: Some(field_mask),
        } = value
        {
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
    #[snafu(display("Invalid username : {source}"), context(suffix(false)))]
    FilterDecodeFailed { source: syn::Error },
}

mod test {

    #[test]
    fn parse_filter() {
        let filter =
        r#"visibilities == ['PUBLIC'] && row_status == "NORMAL" && creator == "users/THELOSTSOUL" && order_by_pinned == true && created_ts_before == 123"#
            .replace("'", "\"");
        let filter = syn::parse_str::<crate::api::memo::Filter>(&filter).unwrap();
        assert_eq!(filter.row_status, Some("NORMAL".to_owned()));
        assert_eq!(
            filter.visibilities,
            Some(vec![crate::api::v2::Visibility::Public])
        );
        assert_eq!(filter.creator, Some("users/THELOSTSOUL".to_owned()));
        assert_eq!(filter.order_by_pinned, Some(true));
        assert_eq!(filter.created_ts_before, Some(123_i64));
    }
}
