use std::sync::OnceLock;

use comrak::{
    nodes::{AstNode, ListType, NodeValue},
    Arena,
};
use regex::Regex;
use syn::{Expr, ExprArray, ExprLit, Lit};

use crate::api::v2::{
    node, BlockquoteNode, BoldItalicNode, BoldNode, CodeBlockNode, CodeNode, HeadingNode,
    HorizontalRuleNode, ImageNode, ItalicNode, LineBreakNode, LinkNode, Node, NodeType,
    OrderedListNode, ParagraphNode, StrikethroughNode, TagNode, TaskListNode, TextNode,
    UnorderedListNode, Visibility,
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
        elems.into_iter().map(|lit| get_string(lit)).collect()
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
// pub fn get_row_status(lit: Expr) -> Option<RowStatus> {
//     let row_status = get_string(lit);
//     row_status.map(|s| RowStatus::from_str_name(&s)).flatten()
// }

pub fn get_visibility(lit: Expr) -> Option<Visibility> {
    let visibility = get_string(lit);
    visibility.map(|s| Visibility::from_str_name(&s)).flatten()
}

pub fn get_visibilities(lit: Expr) -> Option<Vec<Visibility>> {
    if let Expr::Array(ExprArray { elems, .. }) = lit {
        elems.into_iter().map(|lit| get_visibility(lit)).collect()
    } else {
        None
    }
}

static TAG_REGEX: OnceLock<Regex> = OnceLock::new();
pub fn parse_tag(content: &str) -> Vec<&str> {
    let mut rtn = Vec::new();
    let re = TAG_REGEX.get_or_init(|| Regex::new(r"#\S+$|#\S+\s").unwrap());
    let matches = re.find_iter(content);
    let mut i;
    for mat in matches {
        i = if mat.end() == content.len() {
            mat.end()
        } else {
            mat.end() - 1
        };
        if let Some(tag) = content.get(mat.start() + 1..i) {
            rtn.push(tag);
        }
    }
    rtn
}

pub fn parse_document(buffer: impl AsRef<str>) -> Vec<Node> {
    let buffer = buffer.as_ref();
    let arena = Arena::new();
    let mut options = comrak::Options::default();
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.render.unsafe_ = true;
    let root = comrak::parse_document(&arena, buffer, &options);
    parse_node(root)
}

fn parse_node<'a>(node: &'a AstNode<'a>) -> Vec<Node> {
    match &node.data.borrow().value {
        NodeValue::Document => parse_child(node),
        NodeValue::FrontMatter(content) => vec![Node {
            r#type: NodeType::Text.into(),
            node: Some(node::Node::TextNode(TextNode {
                content: content.to_owned(),
            })),
        }],
        NodeValue::BlockQuote => vec![Node {
            r#type: NodeType::Blockquote.into(),
            node: Some(node::Node::BlockquoteNode(BlockquoteNode {
                children: parse_child(node),
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
                children: parse_child(node),
            })),
        }],
        NodeValue::LineBreak => vec![Node {
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
            let matches = re.find_iter(&content);
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
                children: parse_child(node),
            })),
        }],
        NodeValue::List(list) => parse_child(node),
        NodeValue::Item(list) => match list.list_type {
            ListType::Bullet => vec![Node {
                r#type: NodeType::UnorderedList.into(),
                node: Some(node::Node::UnorderedListNode(UnorderedListNode {
                    symbol: (list.bullet_char as char).to_string(),
                    children: parse_child(node),
                })),
            }],
            ListType::Ordered => vec![Node {
                r#type: NodeType::OrderedList.into(),
                node: Some(node::Node::OrderedListNode(OrderedListNode {
                    number: list.start.to_string(),
                    children: parse_child(node),
                })),
            }],
        },
        NodeValue::TaskItem(checked) => vec![Node {
            r#type: NodeType::TaskList.into(),
            node: Some(node::Node::TaskListNode(TaskListNode {
                // 先默认"-”
                symbol: "-".to_owned(),
                complete: checked.is_some(),
                children: parse_child(node),
            })),
        }],
        NodeValue::Strong => vec![Node {
            r#type: NodeType::Bold.into(),
            node: Some(node::Node::BoldNode(BoldNode {
                symbol: Default::default(),
                children: parse_child(node),
            })),
        }],
        NodeValue::Link(link) => vec![Node {
            r#type: NodeType::Link.into(),
            node: Some(node::Node::LinkNode(LinkNode {
                text: link.title.clone(),
                url: link.url.clone(),
            })),
        }],
        NodeValue::Image(link) => vec![Node {
            r#type: NodeType::Image.into(),
            node: Some(node::Node::ImageNode(ImageNode {
                alt_text: link.title.clone(),
                url: link.url.clone(),
            })),
        }],
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

fn parse_child<'a>(node: &'a AstNode<'a>) -> Vec<Node> {
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
    fn parse_tag() {
        let re = regex::Regex::new(r"#\S+$|#\S+\s").unwrap();
        let hay = "#TEST 你好，世界！ #test";
        let matches = re.find_iter(hay);
        let mut i = 0;
        for mat in matches {
            if i < mat.start() {
                println!("text: {:?}", hay.get(i..mat.start()));
            }
            i = if mat.end() == hay.len() {
                mat.end()
            } else {
                mat.end() - 1
            };
            println!("tag: {:?}", hay.get(mat.start() + 1..i));
        }
        println!("text: {:?}", hay.get(i..hay.len()));

        let tags = super::parse_tag(hay);
        println!("tags: {tags:?}")
    }

    #[test]
    fn parse_ast() {
        let buffer = r#"
#LIST 
- a
- b
- c
        "#;
        let arena = comrak::Arena::new();
        let mut options = comrak::Options::default();
        options.extension.tasklist = true;
        options.extension.strikethrough = true;
        options.render.unsafe_ = true;
        let root = comrak::parse_document(&arena, buffer, &options);
        println!("{root:?}");
    }
}
