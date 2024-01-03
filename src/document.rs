
use std::collections::HashMap;

use crate::parse::{ParserPosition, Token};

#[derive(Debug)]
pub struct EnvNodeHeader {
    pub name: String, 
    pub attrs: HashMap<String, String>,
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

impl EnvNodeHeader {

    /** Create new empty header with the specified nae */
    pub fn new_empty(name : String) -> Self {
        Self { name, attrs: HashMap::new() }
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
