
use crate::document::*;

pub enum TransformError {
    Unknown(String),
    RootRemoved,
    MaxIterationsReached,
}

pub enum TransformAction {
    Keep(Node),
    Replace(Node),
    Remove,
}

pub type TransformResult = Result<TransformAction, TransformError>;

pub trait Transformer {
    fn transform_node(&self, node : Node) -> TransformResult;
}

fn transform_node_single_pass(
    node : Node,
    transformers : &Vec<impl Transformer>
) -> TransformResult {

    let mut transform_action = TransformAction::Keep(node);

    for visitor in transformers {
        match transform_action {
            TransformAction::Keep(node) | TransformAction::Replace(node) => {
                transform_action = visitor.transform_node(node)?
            },
            TransformAction::Remove => break
        }
    }

    match transform_action {
        // TODO: tidy up NodeKind: split into Leaf (no children) and NonLeaf (with children) to avoid this
        // TODO: also split up actions into WithNode and WithoutNode or similar
        TransformAction::Keep(Node { kind: NodeKind::Env(EnvNode::Open(env_node)), position }) | 
        TransformAction::Replace(Node { kind: NodeKind::Env(EnvNode::Open(env_node)), position }) => {
            
            let header = env_node.header;

            let mut has_changed = false;

            let children = env_node.children
                .into_iter()
                .map(|child| transform_node_single_pass(child, transformers))
                .collect::<Result<Vec<TransformAction>, TransformError>>()?
                .into_iter()
                .filter(
                    |action| match action { 
                        TransformAction::Remove => { has_changed = true; false }, 
                        TransformAction::Replace(_) => { has_changed = true; true }, 
                        TransformAction::Keep(_) => { true }
                    }
                )
                .map(|action| match action {
                    TransformAction::Replace(node) | TransformAction::Keep(node) => { node },
                    _ => unreachable!()
                })
                .collect::<Vec<Node>>();

            let node = Node {
                kind: NodeKind::Env(
                    EnvNode::Open(EnvNodeOpen { header, children })
                ),
                position
            };
        

            Ok(if has_changed { TransformAction::Replace(node) } else { TransformAction::Keep(node) })
        },
        _ => Ok(transform_action)
    }
}

pub fn transform_node(
    node : Node,
    transformers : &Vec<impl Transformer>,
    max_passes : u32
) -> Result<Node, TransformError> {

    let mut action = TransformAction::Replace(node);

    let mut iterations = 0;

    loop {
        action = match action {
            TransformAction::Replace(node) => transform_node_single_pass(node, transformers)?,
            TransformAction::Keep(node) => return Ok(node),
            TransformAction::Remove => return Err(TransformError::RootRemoved),
        };

        iterations += 1;

        if iterations > max_passes {
            return Err(TransformError::MaxIterationsReached);
        }
    }
}
