use std::{fmt::Display, str::FromStr, sync::OnceLock};

use comrak::{
    nodes::{AstNode, ListType, NodeValue},
    Arena,
};
use regex::Regex;
use syn::{Expr, ExprArray, ExprLit, Lit};

use crate::{
    api::v1::gen::{
        node, BlockquoteNode, BoldItalicNode, BoldNode, CodeBlockNode, CodeNode, HeadingNode,
        HorizontalRuleNode, ImageNode, ItalicNode, LineBreakNode, LinkNode, Node, NodeType,
        OrderedListNode, ParagraphNode, RowStatus, StrikethroughNode, TagNode, TaskListNode,
        TextNode, UnorderedListNode, Visibility,
    },
    model::gen::MemoPayload,
};

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

static TAG_REGEX: OnceLock<Regex> = OnceLock::new();
pub fn parse_document(content: impl AsRef<str>) -> Vec<Node> {
    let content = content.as_ref();
    let arena = Arena::new();
    let mut options = comrak::Options::default();
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.render.unsafe_ = true;
    let root = comrak::parse_document(&arena, content, &options);
    parse_node(root)
}

pub fn get_memo_property(content: impl AsRef<str>) -> MemoPayload {
    let content = content.as_ref();
    let arena = Arena::new();
    let mut options = comrak::Options::default();
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.render.unsafe_ = true;
    let root = comrak::parse_document(&arena, content, &options);
    parse_property(root)
}

fn parse_property<'a>(node: &'a AstNode<'a>) -> MemoPayload {
    match &node.data.borrow().value {
        NodeValue::Document => parse_property_child(node),
        NodeValue::BlockQuote => parse_property_child(node),
        NodeValue::CodeBlock(code) => MemoPayload::code(),
        NodeValue::Paragraph => parse_property_child(node),
        NodeValue::Code(content) => MemoPayload::code(),
        NodeValue::Text(content) => {
            let re = TAG_REGEX.get_or_init(|| Regex::new(r"#\S+$|#\S+\s").unwrap());
            let matches = re.find_iter(content);
            let mut i = 0;
            let mut tags = Vec::new();
            for mat in matches {
                i = if mat.end() == content.len() {
                    mat.end()
                } else {
                    mat.end() - 1
                };
                if let Some(tag) = content.get(mat.start() + 1..i) {
                    tags.push(tag.to_string());
                }
            }
            if i != content.len() {
                if let Some(tag) = content.get(i..content.len()) {
                    tags.push(tag.to_string());
                }
            }
            MemoPayload::tags(tags)
        }
        NodeValue::Heading(head) => parse_property_child(node),
        NodeValue::List(list) => parse_property_child(node),
        NodeValue::Item(list) => parse_property_child(node),
        NodeValue::TaskItem(checked) => {
            let mut payload = if checked.is_none() {
                MemoPayload::incomplete_task()
            } else {
                MemoPayload::task()
            };
            let p = parse_property_child(node);
            payload.merge(p);
            payload
        }
        NodeValue::Strong => parse_property_child(node),
        NodeValue::Link(link) => MemoPayload::link(),
        _ => MemoPayload::default(),
    }
}
fn parse_property_child<'a>(node: &'a AstNode<'a>) -> MemoPayload {
    let mut payload = MemoPayload { property: None };

    for n in node.children() {
        let p = parse_property(n);
        payload.merge(p);
    }
    payload
}

/// commonmark to memomark
/// TODO
/// Highlight = 23,
/// Math = 22,
/// MathBlock = 10,
/// AutoLink = 18,
fn parse_node<'a>(node: &'a AstNode<'a>) -> Vec<Node> {
    match &node.data.borrow().value {
        NodeValue::Document => parse_node_child(node),
        NodeValue::FrontMatter(content) => vec![Node {
            r#type: NodeType::Text.into(),
            node: Some(node::Node::TextNode(TextNode {
                content: content.to_owned(),
            })),
        }],
        NodeValue::BlockQuote => vec![Node {
            r#type: NodeType::Blockquote.into(),
            node: Some(node::Node::BlockquoteNode(BlockquoteNode {
                children: parse_node_child(node),
            })),
        }],
        NodeValue::CodeBlock(code) => vec![Node {
            r#type: NodeType::CodeBlock.into(),
            node: Some(node::Node::CodeBlockNode(CodeBlockNode {
                language: code.info.clone(),
                content: code.literal.clone(),
            })),
        }],
        NodeValue::Paragraph => vec![Node {
            r#type: NodeType::Paragraph.into(),
            node: Some(node::Node::ParagraphNode(ParagraphNode {
                children: parse_node_child(node),
            })),
        }],
        NodeValue::LineBreak | NodeValue::SoftBreak => vec![Node {
            r#type: NodeType::LineBreak.into(),
            node: Some(node::Node::LineBreakNode(LineBreakNode {})),
        }],
        NodeValue::Code(content) => vec![Node {
            r#type: NodeType::Code.into(),
            node: Some(node::Node::CodeNode(CodeNode {
                content: content.literal.clone(),
            })),
        }],
        NodeValue::Text(content) => {
            let mut nodes = Vec::new();
            let re = TAG_REGEX.get_or_init(|| Regex::new(r"#\S+$|#\S+\s").unwrap());
            let matches = re.find_iter(content);
            let mut i = 0;
            for mat in matches {
                if i < mat.start() {
                    nodes.push(Node {
                        r#type: NodeType::Text.into(),
                        node: content.get(i..mat.start()).map(|c| {
                            node::Node::TextNode(TextNode {
                                content: c.to_owned(),
                            })
                        }),
                    });
                }
                i = if mat.end() == content.len() {
                    mat.end()
                } else {
                    mat.end() - 1
                };
                nodes.push(Node {
                    r#type: NodeType::Tag.into(),
                    node: content.get(mat.start() + 1..i).map(|c| {
                        node::Node::TagNode(TagNode {
                            content: c.to_owned(),
                        })
                    }),
                });
            }
            if i != content.len() {
                nodes.push(Node {
                    r#type: NodeType::Text.into(),
                    node: content.get(i..content.len()).map(|c| {
                        node::Node::TextNode(TextNode {
                            content: c.to_owned(),
                        })
                    }),
                });
            }
            nodes
        }
        NodeValue::Heading(head) => vec![Node {
            r#type: NodeType::Heading.into(),
            node: Some(node::Node::HeadingNode(HeadingNode {
                level: head.level as i32,
                children: parse_node_child(node),
            })),
        }],
        NodeValue::List(list) => parse_node_child(node),
        NodeValue::Item(list) => match list.list_type {
            ListType::Bullet => vec![Node {
                r#type: NodeType::UnorderedList.into(),
                node: Some(node::Node::UnorderedListNode(UnorderedListNode {
                    symbol: (list.bullet_char as char).to_string(),
                    indent: list.marker_offset as i32,
                    children: parse_node_child(node),
                })),
            }],
            ListType::Ordered => vec![Node {
                r#type: NodeType::OrderedList.into(),
                node: Some(node::Node::OrderedListNode(OrderedListNode {
                    number: list.start.to_string(),
                    indent: list.marker_offset as i32,
                    children: parse_node_child(node),
                })),
            }],
        },
        NodeValue::TaskItem(checked) => vec![Node {
            r#type: NodeType::TaskList.into(),
            node: Some(node::Node::TaskListNode(TaskListNode {
                // 先默认"-”
                symbol: "-".to_owned(),
                indent: 0,
                complete: checked.is_some(),
                children: parse_node_child(node),
            })),
        }],
        NodeValue::Strong => vec![Node {
            r#type: NodeType::Bold.into(),
            node: Some(node::Node::BoldNode(BoldNode {
                symbol: Default::default(),
                children: parse_node_child(node),
            })),
        }],
        NodeValue::Link(link) => {
            let mut title = append_text(node);
            if title.is_empty() {
                title.clone_from(&link.url);
            }
            vec![Node {
                r#type: NodeType::Link.into(),
                node: Some(node::Node::LinkNode(LinkNode {
                    text: title,
                    url: link.url.clone(),
                })),
            }]
        }
        NodeValue::Image(link) => {
            let alt_text = append_text(node);
            vec![Node {
                r#type: NodeType::Image.into(),
                node: Some(node::Node::ImageNode(ImageNode {
                    alt_text,
                    url: link.url.clone(),
                })),
            }]
        }
        NodeValue::Emph => {
            let mut nodes = Vec::new();
            for n in node.children() {
                if n.data.borrow().value == NodeValue::Strong {
                    for nn in n.children() {
                        if let NodeValue::Text(content) = &nn.data.borrow().value {
                            nodes.push(Node {
                                r#type: NodeType::BoldItalic.into(),
                                node: Some(node::Node::BoldItalicNode(BoldItalicNode {
                                    symbol: Default::default(),
                                    content: content.to_owned(),
                                })),
                            });
                        }
                    }
                } else if let NodeValue::Text(content) = &n.data.borrow().value {
                    nodes.push(Node {
                        r#type: NodeType::Italic.into(),
                        node: Some(node::Node::ItalicNode(ItalicNode {
                            symbol: Default::default(),
                            content: content.to_owned(),
                        })),
                    });
                }
            }
            nodes
        }
        NodeValue::ThematicBreak => vec![Node {
            r#type: NodeType::HorizontalRule.into(),
            node: Some(node::Node::HorizontalRuleNode(HorizontalRuleNode {
                // 先默认"-”
                symbol: "-".to_owned(),
            })),
        }],
        NodeValue::Strikethrough => vec![Node {
            r#type: NodeType::Strikethrough.into(),
            node: Some(node::Node::StrikethroughNode(StrikethroughNode {
                content: append_text(node),
            })),
        }],
        NodeValue::HtmlBlock(block) => vec![Node {
            r#type: NodeType::Text.into(),
            node: Some(node::Node::TextNode(TextNode {
                content: block.literal.to_owned(),
            })),
        }],
        NodeValue::HtmlInline(content) => vec![Node {
            r#type: NodeType::Text.into(),
            node: Some(node::Node::TextNode(TextNode {
                content: content.to_owned(),
            })),
        }],
        _ => vec![],
    }
}
fn parse_node_child<'a>(node: &'a AstNode<'a>) -> Vec<Node> {
    let mut nodes = Vec::new();
    for n in node.children() {
        nodes.append(&mut parse_node(n));
    }
    nodes
}

fn append_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut rtn = String::new();
    for n in node.children() {
        if let NodeValue::Text(content) = &n.data.borrow().value {
            rtn.push_str(content);
        }
    }
    rtn
}

mod test {

    #[test]
    fn parse_ast() {
        let buffer = r#"
#LIST
aaaaaa"#;
        let nodes = super::parse_document(buffer);
        println!("{nodes:?}");
    }
}
