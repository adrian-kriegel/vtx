
use std::collections::HashMap;

use crate::document::{
    EnvNodeAttrs, EnvNodeHeaderKind, EquationKind, LeafNode, NodeKind
};

/// determines how env children are parsed
#[derive(Debug)]
pub enum ContentParseMode {
    /// parse as nodes
    Vtx,
    /// parse as string until encountering un-escaped end tag
    Raw,
    /// same as Raw but requires end tag to be preceded by whitespace containing a line break
    RawStrict,
}

/// Dynamic parsing attributes for envs
#[derive(Debug)]
pub struct EnvParseAttrs {
    content: ContentParseMode,
}

const ENV_PARSE_ATTRS_DEFAULT : EnvParseAttrs = EnvParseAttrs {
    content: ContentParseMode::Vtx
};

pub struct DynamicParserState {
    /// keeps track of all EnvParseAttrs defined so far
    env_parse_attrs: HashMap<EnvNodeHeaderKind, EnvParseAttrs>,
}


#[derive(Debug)]
pub enum DynamicParsingError {
    InvalidContentParseMode,
}

impl ContentParseMode {

    pub fn from_attrs(attrs : &EnvNodeAttrs) -> Result<Self, DynamicParsingError> {
        match attrs.get("content") {
            Some(value) => match value {
                Some(node) => match &node.kind {
                    NodeKind::Leaf(LeafNode::Text(mode)) => {
                        match mode.as_str() {
                            "vtx" => Ok(ContentParseMode::Vtx),
                            "raw" => Ok(ContentParseMode::Vtx),
                            "raw-strict" => Ok(ContentParseMode::Vtx),
                            _ => Err(DynamicParsingError::InvalidContentParseMode)
                        }
                    },
                    _ => Err(DynamicParsingError::InvalidContentParseMode)
                },
                None => Ok(Self::Vtx),
            },
            None => Ok(Self::Vtx),
        }
    }

}

impl DynamicParserState {

    pub fn new() -> Self {
        Self {
            // TODO: define Eq/Code as components
            env_parse_attrs: HashMap::from([
                (EnvNodeHeaderKind::Eq(EquationKind::Block), EnvParseAttrs {
                    content: ContentParseMode::Raw
                }),
                (EnvNodeHeaderKind::Eq(EquationKind::Inline), EnvParseAttrs {
                    content: ContentParseMode::Raw
                }),
                (EnvNodeHeaderKind::Code, EnvParseAttrs {
                    content: ContentParseMode::Raw
                })
            ])
        }
    }

}

impl EnvParseAttrs {

    pub fn from_attrs(attrs : &EnvNodeAttrs) -> Result<Self, DynamicParsingError> {
        Ok(Self {
            content: ContentParseMode::from_attrs(attrs)?
        })
    }

    pub fn content(&self) -> &ContentParseMode {
        return &self.content;
    }

}

impl DynamicParserState {

    pub fn get_env_parse_attrs(&self, header_kind : &EnvNodeHeaderKind) -> &EnvParseAttrs {

        self.env_parse_attrs.get(header_kind).unwrap_or(&ENV_PARSE_ATTRS_DEFAULT)

    }
}
