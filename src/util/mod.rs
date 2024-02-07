use std::collections::HashSet;

use crate::api::v2::{node, Node, TagNode};
use nanoid::{alphabet, nanoid};
use snafu::{ensure, Snafu};

pub mod ast;

pub trait IntoNode {
    fn into(self) -> Vec<Node>;
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

pub fn uuid() -> String {
    nanoid!(16, &alphabet::SAFE)
}

pub fn parse_tag(content: impl AsRef<str>) -> HashSet<String> {
    let mut rtn = HashSet::new();
    let tags = ast::parse_document(content, true);
    for tag in tags {
        if let Some(node::Node::TagNode(TagNode { content })) = tag.node {
            rtn.insert(content);
        }
    }
    rtn
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Invalid request : {name}"), context(suffix(false)))]
    InvalidRequest { name: String },
    #[snafu(display("Invalid prefix in request : {name}"), context(suffix(false)))]
    InvalidPrefix { name: String },
}
