
use std::{collections::{HashMap, VecDeque}, sync::atomic::{AtomicUsize, Ordering}};

use crate::parse::{ParserPosition, Token};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EquationKind {
    Inline, 
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EnvNodeHeaderKind {
    Eq(EquationKind),
    Code,
    Module,
    Heading(u8),
    Other(String),
    // container for a list of child nodes
    Fragment
}

#[derive(Debug, Clone)]
pub struct EnvNodeMetaAttrs {
    /** Indicates that anything inside this environment will be parsed as text. */
    pub raw : bool
}

pub type EnvNodeAttrs = HashMap<String, Option<Node>>;

#[derive(Debug, Clone)]
pub struct EnvNodeHeader {
    pub kind: EnvNodeHeaderKind,
    pub attrs: EnvNodeAttrs,
}

#[derive(Debug, Clone)]
pub enum EnvNodeKind {
    Open(VecDeque<Node>),
    SelfClosing,
}

#[derive(Debug, Clone)]
pub struct EnvNode {
    pub kind: EnvNodeKind,
    pub header: EnvNodeHeader,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeafNode {
    Text(String),
    VariableExpression(String),
    Comment(String),
    RawBytes(Vec<u8>),
    // TODO: better error representation
    Error(String)
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

#[derive(Debug)]
pub struct Node {
    pub id : NodeId,
    pub kind: NodeKind,
    pub position: NodePosition,
}

// TODO: this isn't really great: the user would expect a cloned node to have the same id? 
// but it is required for TransformerOnce to work...
// --> change the way TransformerOnce works...
impl Clone for Node {
    fn clone(&self) -> Self {
        Self { 
            id: Node::generate_id(), 
            kind: self.kind.clone(), 
            position: self.position.clone()
        }
    }
}

// TODO: This is only required in order to compare attrs in testing. Remove
impl PartialEq for Node {

    fn eq(&self, other: &Self) -> bool {
        // TODO
        match &self.kind {
            NodeKind::Leaf(LeafNode::Text(text)) => {
                match &other.kind {
                    NodeKind::Leaf(LeafNode::Text(other_text)) => {
                        other_text == text
                    },
                    _ => false
                }
            },
            _ => false,
        }
    }

}

impl NodeKind {

    pub fn new_fragment(children: VecDeque<Node>) -> Self {
        NodeKind::Env(
            EnvNode {
                kind: EnvNodeKind::Open(children),
                header: EnvNodeHeader {
                    kind: EnvNodeHeaderKind::Fragment,
                    attrs: EnvNodeAttrs::new(),
                }
            }
        )
    }

    pub fn new_variable_definition(name : &str, value : Node) -> Self {
        NodeKind::Env(
            EnvNode::new_open(
                EnvNodeHeader::new(
                    "var",
                    HashMap::from([(name.to_string(), None)])
                ),
                VecDeque::from([value])
            )
        )
    }

}

impl EnvNode {

    /** Create new self closing tag. */
    pub fn new_self_closing(header : EnvNodeHeader) -> Self {
        Self { kind: EnvNodeKind::SelfClosing, header }
    }

    /** Create new open tag. */
    pub fn new_open(header : EnvNodeHeader, children: VecDeque<Node>) -> Self {
        Self { kind: EnvNodeKind::Open(children), header }
    }

    /** Create new module environment. */
    pub fn new_module(children: VecDeque<Node>) -> Self {
        Self { 
            kind: EnvNodeKind::Open(children), 
            header: EnvNodeHeader {
                kind: EnvNodeHeaderKind::Module,
                attrs: EnvNodeAttrs::new(),
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
            EnvNodeHeaderKind::Fragment => "",
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

    pub fn generate_attrs(pairs : Vec<(&str,Option<&str>)>) -> EnvNodeAttrs {

        let mut attrs = EnvNodeAttrs::new();

        for (key, value) in pairs {
            attrs.insert(
                key.to_string(), 
                value.map(|value| Node::new(
                        NodeKind::Leaf(LeafNode::Text(value.to_string())),
                    NodePosition::Inserted,
                    )
                )
            );
        }

        return attrs;

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

    pub fn new_variable_definition(name : &str, value : Node) -> Self {
        Node {
            kind: NodeKind::new_variable_definition(name, value),
            id: Node::generate_id(),
            position: NodePosition::Inserted
        }
    }
}

impl NodeKind {

    pub fn heading(level: u8, children:  VecDeque<Node>) -> Self {
        NodeKind::Env(
            EnvNode {
                kind: EnvNodeKind::Open(children),
                header: EnvNodeHeader {
                    kind: EnvNodeHeaderKind::Heading(level),
                    attrs: EnvNodeAttrs::new(),
                }
            }
        )
    }

}
