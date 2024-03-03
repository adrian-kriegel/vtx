
use crate::document::*;
use crate::visit::{Action, VisitError, TransformResult, Visitor};
use html_escape::encode_safe;

pub struct HTMLEmitter {
    /// 
    /// Called for every sub-string in the emitted HTML.
    /// Can be used to concatenate into a string or stream to a file or socket.
    /// 
    pub collector: fn (&str),
    pub debug: bool,
}

// there must be a library for this... 
// TODO: tidy this up...
fn encode(text: &str) -> String {
    encode_safe(text)
        .replace("ä", "&auml;")
        .replace("ö", "&ouml;")
        .replace("ü", "&uuml;")
        .replace("Ä", "&Auml;")
        .replace("Ö", "&Ouml;")
        .replace("Ü", "&Uuml;")
        .replace("ß", "&szlig;")
        .replace("á", "&aacute;")
        .replace("é", "&eacute;")
        .replace("í", "&iacute;")
        .replace("ó", "&oacute;")
        .replace("ú", "&uacute;")
        .replace("Á", "&Aacute;")
        .replace("É", "&Eacute;")
        .replace("Í", "&Iacute;")
        .replace("Ó", "&Oacute;")
        .replace("Ú", "&Uacute;")
        .replace("à", "&agrave;")
        .replace("è", "&egrave;")
        .replace("ì", "&igrave;")
        .replace("ò", "&ograve;")
        .replace("ù", "&ugrave;")
        .replace("À", "&Agrave;")
        .replace("È", "&Egrave;")
        .replace("Ì", "&Igrave;")
        .replace("Ò", "&Ograve;")
        .replace("Ù", "&Ugrave;")
        .replace("â", "&acirc;")
        .replace("ê", "&ecirc;")
        .replace("î", "&icirc;")
        .replace("ô", "&ocirc;")
        .replace("û", "&ucirc;")
        .replace("Â", "&Acirc;")
        .replace("Ê", "&Ecirc;")
        .replace("Î", "&Icirc;")
        .replace("Ô", "&Ocirc;")
        .replace("Û", "&Ucirc;")
        .replace("ã", "&atilde;")
        .replace("ñ", "&ntilde;")
        .replace("õ", "&otilde;")
        .replace("Ã", "&Atilde;")
        .replace("Ñ", "&Ntilde;")
        .replace("Õ", "&Otilde;")
        .replace("å", "&aring;")
        .replace("Å", "&Aring;")
        .replace("ç", "&ccedil;")
        .replace("Ç", "&Ccedil;")
        .replace("ë", "&euml;")
        .replace("ï", "&iuml;")
        .replace("Ö", "&Ouml;")
        .replace("ÿ", "&yuml;")
}

fn collect_env_attrs(attrs : &EnvNodeAttrs, f: &fn(&str)) {

    for (key, value) in attrs {

        f(key);

        if let Some(value) = value  {
            f("=\"");
            
            match &value.kind {
                NodeKind::Leaf(LeafNode::Text(text)) => f(&encode(text)),
                _ =>  todo!("Attr values must be text nodes.")
            }

            f("\" ");
        } else {
            f(" ");
        }

    }
}

fn collect_env_header(header : &EnvNodeHeader, f: &fn(&str)) {

    match header.kind {
        EnvNodeHeaderKind::Module => {},
        _ => {
            f("<");
            f(header.kind.get_name());

            if !header.attrs.is_empty() {
                f(" ");
                collect_env_attrs(&header.attrs, f)
            }

            f(">");
        }
    }
}

impl Visitor for HTMLEmitter {

    fn enter(&mut self, node : Node, _parent_id : Option<NodeId>) -> TransformResult {

        match &node.kind {
            NodeKind::Env(node) => match &node.header.kind {
                EnvNodeHeaderKind::Fragment => { },
                _ => collect_env_header(&node.header, &self.collector)
            }

            NodeKind::Leaf(LeafNode::Text(text)) => (self.collector)(&encode(text)),
            kind if self.debug => {
                dbg!(kind);
            },
            _ => return Err(
                VisitError::Unknown(
                    "Encountered a node which cannot be emitted as HTML.".to_string()
                )
            )
        }

        Ok(Action::keep(node))

    }

    fn leave(&mut self, node : &Node, _original_id : NodeId, _parent_id : Option<NodeId>) {

        match &node.kind {
            NodeKind::Env(node) => match &node.header.kind {
                EnvNodeHeaderKind::Fragment => { },
                _ => (self.collector)(&node.header.kind.get_closing_string())
            },
            _ => {}
        }
    }

}


