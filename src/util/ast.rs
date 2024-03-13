use syn::{Expr, ExprArray, ExprLit, Lit};

use crate::api::v2::{RowStatus, Visibility};

pub fn get_string(lit: Expr) -> Option<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = lit
    {
        Some(s.value())
    } else {
        None
    }
}

pub fn get_string_list(lit: Expr) -> Option<Vec<String>> {
    if let Expr::Array(ExprArray { elems, .. }) = lit {
        elems.into_iter().map(get_string).collect()
    } else {
        None
    }
}

pub fn get_i64(lit: Expr) -> Option<i64> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = lit
    {
        i.base10_parse().ok()
    } else {
        None
    }
}

pub fn get_bool(lit: Expr) -> Option<bool> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Bool(b), ..
    }) = lit
    {
        Some(b.value())
    } else {
        None
    }
}

/// RowStatus no match
pub fn get_row_status(lit: Expr) -> Option<RowStatus> {
    let row_status = get_string(lit);
    row_status.and_then(|s| s.parse().ok())
}

pub fn get_visibility(lit: Expr) -> Option<Visibility> {
    let visibility = get_string(lit);
    visibility.and_then(|s| Visibility::from_str_name(&s))
}

pub fn get_visibilities(lit: Expr) -> Option<Vec<Visibility>> {
    if let Expr::Array(ExprArray { elems, .. }) = lit {
        elems.into_iter().map(get_visibility).collect()
    } else {
        None
    }
}
