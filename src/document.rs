
use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

use crate::parse::{ParserPosition, Token};

#[derive(Debug, Clone)]
pub enum EquationKind {
    Inline, 
    Block,
}

#[derive(Debug, Clone)]
pub enum EnvNodeHeaderKind {
    Eq(EquationKind),
    Code,
    Module,
    Heading(u8),
    Other(String)
}

#[derive(Debug, Clone)]
pub struct EnvNodeMetaAttrs {
    /** Indicates that anything inside this environment will be parsed as text. */
    pub raw : bool
}

pub type EnvNodeAttrs = HashMap<String, Option<String>>;

#[derive(Debug, Clone)]
pub struct EnvNodeHeader {
    pub kind: EnvNodeHeaderKind,
    pub attrs: EnvNodeAttrs,
    /** Attributes about the node, relevant at parse time. */
    pub meta_attrs: EnvNodeMetaAttrs,
}

#[derive(Debug, Clone)]
pub enum EnvNodeKind {
    Open(Vec<Node>),
    SelfClosing,
}

#[derive(Debug, Clone)]
pub struct EnvNode {
    pub kind: EnvNodeKind,
    pub header: EnvNodeHeader,
}

#[derive(Debug, Clone)]
pub enum LeafNode {
    Text(String),
    VariableExpression(String),
    Comment(String),
    RawBytes(Vec<u8>)
}

#[derive(Debug, Clone)]
pub enum NodeKind{
    Leaf(LeafNode),
    Env(EnvNode),
}

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub enum NodePosition {
    Source(ParserPosition),
    Inserted
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id : NodeId,
    pub kind: NodeKind,
    pub position: NodePosition,
}

impl EnvNode {

    /** Create new self closing tag. */
    pub fn new_self_closing(header : EnvNodeHeader) -> Self {
        Self { kind: EnvNodeKind::SelfClosing, header }
    }

    /** Create new open tag. */
    pub fn new_open(header : EnvNodeHeader, children: Vec<Node>) -> Self {
        Self { kind: EnvNodeKind::Open(children), header }
    }

    /** Create new module environment. */
    pub fn new_module(children: Vec<Node>) -> Self {
        Self { 
            kind: EnvNodeKind::Open(children), 
            header: EnvNodeHeader {
                kind: EnvNodeHeaderKind::Module,
                attrs: EnvNodeAttrs::new(),
                meta_attrs: EnvNodeMetaAttrs { raw: false },
            }
        }
    }
}



impl EnvNodeMetaAttrs {

    pub fn new(header_kind : &EnvNodeHeaderKind) -> Self {
        EnvNodeMetaAttrs {
            raw: match header_kind {
                EnvNodeHeaderKind::Code | EnvNodeHeaderKind::Eq(_) => true,
                _ => false
            }
        }
    }

}

impl EnvNodeHeaderKind {

    pub fn new(name : &str) -> Self {
        match name {
            "Eq" => Self::Eq(EquationKind::Block),
            "Code" => Self::Code, 
            _ => Self::Other(String::from(name)),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            EnvNodeHeaderKind::Eq(_) => "Eq",
            EnvNodeHeaderKind::Code => "Code",
            EnvNodeHeaderKind::Module => "",
            EnvNodeHeaderKind::Heading(0) => "h1",
            EnvNodeHeaderKind::Heading(1) => "h2",
            EnvNodeHeaderKind::Heading(_) => "h3",
            EnvNodeHeaderKind::Other(name) => &name
        }
    }

    pub fn get_closing_string(&self) -> String {
        match self {
            EnvNodeHeaderKind::Module => "".to_string(),
            _ => format!("</{}>", self.get_name()),
        }
    }

}


impl EnvNodeHeader {

    /** Create new empty header with the specified name */
    pub fn new(parsed_name : &str, attrs : EnvNodeAttrs) -> Self {

        let kind = EnvNodeHeaderKind::new(parsed_name);

        Self { 
            meta_attrs: EnvNodeMetaAttrs::new(&kind),
            kind, 
            attrs,
        }
    }

    pub fn new_default(parsed_name : &str) -> Self {

        Self::new(parsed_name, Self::default_attrs(parsed_name))
    }

    pub fn default_attrs(parsed_name : &str) -> EnvNodeAttrs {
        match parsed_name {
            "Eq" => EnvNodeAttrs::from([("block".to_string(), None)]),
            _ => EnvNodeAttrs::new()
        }
    }
}

static NODE_ID_COUNTER : AtomicUsize = AtomicUsize::new(0);

impl Node {

    pub fn new(kind : NodeKind, position : NodePosition) -> Node {
        Node {
            id: Self::generate_id(),
            kind,
            position
        }
    }

    pub fn new_text(token: &Token) -> Self {
        Self::new(
            NodeKind::Leaf(LeafNode::Text(String::from(token.value))),
            NodePosition::Source(token.position.clone())
        )
    }

    pub fn generate_id() -> NodeId {
        
        NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)

    }
}

impl NodeKind {

    pub fn heading(level: u8, children:  Vec<Node>) -> Self {
        NodeKind::Env(
            EnvNode {
                kind: EnvNodeKind::Open(children),
                header: EnvNodeHeader {
                    kind: EnvNodeHeaderKind::Heading(level),
                    attrs: EnvNodeAttrs::new(),
                    meta_attrs: EnvNodeMetaAttrs { raw: false }
                }
            }
        )
    }

}
