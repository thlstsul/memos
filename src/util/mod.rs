use crate::api::v2::Node;

pub mod ast;

pub trait IntoNode {
    fn into(self) -> Vec<Node>;
}
