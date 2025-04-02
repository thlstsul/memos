use std::{fmt::Display, str::FromStr};

use syn::{Expr, ExprArray, ExprLit, Lit};

use crate::api::v1::gen::{State, Visibility};

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

pub fn get_int<N>(lit: Expr) -> Option<N>
where
    N: FromStr,
    N::Err: Display,
{
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
pub fn get_state(lit: Expr) -> Option<State> {
    let state = get_string(lit);
    state.and_then(|s| s.parse().ok())
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
