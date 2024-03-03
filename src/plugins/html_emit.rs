
use crate::document::*;
use crate::visit::{Action, VisitError, TransformResult, Visitor};

pub struct HTMLEmitter {
    /// 
    /// Called for every sub-string in the emitted HTML.
    /// Can be used to concatenate into a string or stream to a file or socket.
    /// 
    pub collector: fn (&str)
}

fn collect_env_attrs(attrs : &EnvNodeAttrs, f: &fn(&str)) {

    for (key, value) in attrs {

        f(key);

        if let Some(value) = value  {
            f("=\"");
            
            match &value.kind {
                NodeKind::Leaf(LeafNode::Text(text)) => f(&text),
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
            NodeKind::Env(node) => 
                collect_env_header(&node.header, &self.collector),

            NodeKind::Leaf(LeafNode::Text(text)) => (self.collector)(&text),
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
            NodeKind::Env(node) => 
                (self.collector)(&node.header.kind.get_closing_string()),
            _ => {}
        }
    }

}


