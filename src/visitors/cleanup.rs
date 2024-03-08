///
/// Cleans up text and removes nodes that do not contribute to the contents of the document.
/// These include empty lines at the start or end of env bodies.
///

use crate::document::{
    EnvNode,
    EnvNodeKind,
    LeafNode,
    Node, 
    NodeId,
    NodeKind, 
    visit::{Action, TransformResult, Visitor}
};

pub struct Cleanup;

fn is_empty_text(node : &Node) -> bool {

    match &node.kind {
        NodeKind::Leaf(LeafNode::Text(text)) => text
            .chars()
            .find(|c|!c.is_whitespace())
            .is_none(),
        _ => false,
    }

}

impl Visitor for Cleanup {

    fn enter(&mut self, node : Node, _parent_id : Option<NodeId>) -> TransformResult {
        match node.kind {
            NodeKind::Env(
                EnvNode { 
                    kind: EnvNodeKind::Open(mut children), 
                    header,
                }
            ) => {
                let front_is_empty = children.front().map_or(
                    false,
                    is_empty_text
                );

                let back_is_empty = children.back().map_or(
                    false,
                    is_empty_text
                );

                if front_is_empty {
                    children.pop_front();
                }

                if back_is_empty {
                    children.pop_back();
                }

                let node = Node {
                    kind: NodeKind::Env(
                        EnvNode  {
                            kind: EnvNodeKind::Open(children),
                            header: header.clone(),
                        }
                    ),
                    ..node
                };

                if !back_is_empty && !front_is_empty {

                    Ok(Action::keep(node))

                } else {

                    Ok(Action::replace(node))
                }
            },
            _ => Ok(Action::keep(node))
        }

    }

}
