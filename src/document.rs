
use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

use crate::parse::{ParserPosition, Token};


#[derive(Debug)]
pub enum EnvNodeHeaderKind {
    Eq,
    Code,
    Module,
    Other(String)
}

#[derive(Debug)]
pub struct EnvNodeMetaAttrs {
    /** Indicates that anything inside this environment will be parsed as text. */
    pub raw : bool
}

pub type EnvNodeAttrs = HashMap<String, Option<String>>;

#[derive(Debug)]
pub struct EnvNodeHeader {
    pub kind: EnvNodeHeaderKind,
    pub attrs: EnvNodeAttrs,
    /** Attributes about the node, relevant at parse time. */
    pub meta_attrs: EnvNodeMetaAttrs,
}

#[derive(Debug)]
pub enum EnvNodeKind {
    Open(Vec<Node>),
    SelfClosing,
}

#[derive(Debug)]
pub struct EnvNode {
    pub kind: EnvNodeKind,
    pub header: EnvNodeHeader,
}

#[derive(Debug)]
pub enum LeafNode {
    InlineEquation(String),
    Text(String),
    Comment(String),
    RawBytes(Vec<u8>)
}

#[derive(Debug)]
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


#[derive(Debug)]
pub enum EmitError<'a> {
    NodeNotTransformed(&'a Node)
}

pub trait CollectBytes {

    fn collect_bytes<F>(&self, f : &mut F) 
    -> Result<(), EmitError> 
    where F : FnMut(&[u8]);

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

impl CollectBytes for EnvNode {

    fn collect_bytes<F>(&self, f : &mut F) -> Result<(), EmitError>
    where F : FnMut(&[u8]) {

        self.header.collect_bytes(f)?;

        if let EnvNodeKind::Open(children) = &self.kind {
            for child in children {
                child.collect_bytes::<F>(f)?;
            }
        }

        f(self.header.kind.get_closing_string().as_bytes());

        Ok(())
    }
}

impl CollectBytes for EnvNodeAttrs {

    fn collect_bytes<F>(&self, f : &mut F) -> Result<(), EmitError>
    where F : FnMut(&[u8]) {

        for (key, value) in self {

            f(key.as_bytes());

            if let Some(value) = value  {
                f(&[b'=', b'"']);
                f(value.as_bytes());
                f(&[b'"', b' ']);
            } else {
                f(&[b' ']);
            }

        }

        Ok(())

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

    pub fn get_name(&self) -> &str {
        match self {
            EnvNodeHeaderKind::Eq => "Eq",
            EnvNodeHeaderKind::Code => "Code",
            EnvNodeHeaderKind::Module => "",
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

    /** Create new empty header with the specified nae */
    pub fn new(parsed_name : &str, attrs : EnvNodeAttrs) -> Self {

        let kind = EnvNodeHeaderKind::new(parsed_name);

        Self { 
            meta_attrs: EnvNodeMetaAttrs::new(&kind),
            kind, 
            attrs,
        }
    }

    pub fn new_empty(parsed_name : &str) -> Self {

        Self::new(parsed_name, EnvNodeAttrs::new())
    }
}

impl CollectBytes for EnvNodeHeader {

    fn collect_bytes<F>(&self, f : &mut F) 
    -> Result<(), EmitError> 
    where F : FnMut(&[u8]) {
        
        match self.kind {
            EnvNodeHeaderKind::Module => {},
            _ => {
                f(&[b'<']);
                f(self.kind.get_name().as_bytes());

                if !self.attrs.is_empty() {

                    f(&[b' ']);

                    self.attrs.collect_bytes(f)?;

                }

                f(&[b'>']);
            }
        }
        
        Ok(())
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

impl CollectBytes for Node {

    fn collect_bytes<F>(&self, f : &mut F) -> Result<(), EmitError>
    where F : FnMut(&[u8]) {

        match &self.kind {
            NodeKind::Leaf(LeafNode::RawBytes(bytes)) => f(bytes),

            NodeKind::Leaf(
                LeafNode::Text(text) | 
                LeafNode::InlineEquation(text)
            ) => f(text.as_bytes()),

            NodeKind::Env(env_node) => env_node.collect_bytes(f)?,

            _ => Err(EmitError::NodeNotTransformed(self))?,
        }

        Ok(())
    }

}
