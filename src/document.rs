
use std::collections::HashMap;

use crate::parse::{ParserPosition, Token};

#[derive(Debug)]
pub enum EnvNodeHeaderKind {
    Eq,
    Code,
    Other(String)
}

#[derive(Debug)]
pub struct EnvNodeMetaAttrs {
    /**  */
    pub raw : bool
}

pub type EnvNodeAttrs = HashMap<String, String>;

#[derive(Debug)]
pub struct EnvNodeHeader {
    pub kind: EnvNodeHeaderKind,
    pub attrs: EnvNodeAttrs,
    /** Attributes about the node, relevant at parse time. */
    pub meta_attrs: EnvNodeMetaAttrs,
}

#[derive(Debug)]
pub struct EnvNodeOpen {
    pub header: EnvNodeHeader,
    pub children: Vec<Node>,
}

#[derive(Debug)]
pub struct EnvNodeSelfClosing {
    pub header: EnvNodeHeader,
}

#[derive(Debug)]
pub enum EnvNode {
    Root(EnvNodeOpen),
    Open(EnvNodeOpen),
    SelfClosing(EnvNodeSelfClosing),
}

#[derive(Debug)]
pub enum NodeKind{
    InlineEquation(String),
    Text(String),
    Env(EnvNode),
    Comment(Vec<Node>),
    Root(Vec<Node>),
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub position: ParserPosition,
}

impl EnvNode {

    /** Create new self closing tag. */
    pub fn new_self_closing(header : EnvNodeHeader) -> Self {

        Self::SelfClosing(EnvNodeSelfClosing{ header })
    }
}

impl EnvNodeMetaAttrs {

    pub fn new(header_kind : &EnvNodeHeaderKind) -> Self {
        EnvNodeMetaAttrs {
            raw: match header_kind {
                EnvNodeHeaderKind::Code | EnvNodeHeaderKind::Eq => true,
                _ => false
            }
        }
    }

}

impl EnvNodeHeaderKind {

    pub fn new(name : &str) -> Self {
        match name {
            "Eq" => Self::Eq,
            "Code" => Self::Code, 
            _ => Self::Other(String::from(name)),
        }
    }

}

impl EnvNodeHeader {

    /** Create new empty header with the specified nae */
    pub fn new_empty(parsed_name : &str) -> Self {

        let kind = EnvNodeHeaderKind::new(parsed_name);

        Self { 
            meta_attrs: EnvNodeMetaAttrs::new(&kind),
            kind, 
            attrs: HashMap::new(),
        }
    }
}

impl Node {

    pub fn new_text(token: &Token) -> Self {
        Node {
            kind: NodeKind::Text(String::from(token.value)),
            position: token.position.clone()
        }
    }

}
