
use std::collections::HashSet;

use crate::document::*;

#[derive(Debug)]
pub enum VisitError {
    Unknown(String),
    RootRemoved,
    MaxIterationsReached,
}


pub enum ActionKind {
    Remove,
    Replace,
    Keep,
}

pub struct Action {
    kind: ActionKind,
    node: Node,
}

impl Action {
    pub fn keep(node: Node) -> Action {
        Action {
            kind: ActionKind::Keep,
            node,
        }
    }

    pub fn replace(node: Node) -> Action {
        Action {
            kind: ActionKind::Replace,
            node,
        }
    }

    pub fn remove(node: Node) -> Action {
        Action {
            kind: ActionKind::Remove,
            node,
        }
    }
}

pub type TransformResult = Result<Action, VisitError>;

pub trait Visitor {
    //
    // Called when entering a node, before entering the children.
    //
    fn enter(&mut self, node : Node, _parent_id : Option<NodeId>) -> TransformResult {
        Ok(Action::keep(node))
    }

    //
    // Called when leaving a node, after entering all children. 
    // The node passed to leave() is the transformed node, including its children.
    // The original_id is the id of the node that was initially entered. 
    //
    fn leave(&mut self, _node : &Node, _original_id : NodeId, _parent_id : Option<NodeId>) {
        
    }
}

pub struct TransformerOnce<T : Visitor> {

    transformer: T,

    visited: HashSet<NodeId>
}

impl<T: Visitor> Visitor for TransformerOnce<T> {

    fn enter(&mut self, node : Node, parent_id : Option<NodeId>) -> TransformResult {

        if self.visited.contains(&node.id) {
            Ok(Action::keep(node))
        } else {
            self.transformer.enter(node, parent_id)
        }
    }

    fn leave(&mut self, node : &Node, original_id : NodeId, parent_id : Option<NodeId>) {
        if !self.visited.contains(&original_id) {
            self.visited.insert(original_id);
            self.transformer.leave(node, original_id, parent_id)
        }
    }

}

impl<T : Visitor> TransformerOnce<T> {

    pub fn new(transformer : T) -> Self {
        Self {
            transformer,
            visited: HashSet::new()
        }
    }

}

impl Action {

    // TODO: add some sort of matching mechanism to avoid double-match
    pub fn append_children(node : Node, mut children : Vec<Node>) -> Action {

        match node {
            Node { 
                kind: NodeKind::Env(
                    EnvNode{ 
                        header, 
                        kind: EnvNodeKind::Open(mut old_children)
                    }
                ),
                ..
            } => {
                old_children.append(&mut children);

                Action::replace(
                    Node {
                        kind: {
                            NodeKind::Env(
                                EnvNode{ 
                                    header, 
                                    kind: EnvNodeKind::Open(old_children)
                                }
                            )
                        },
                        ..node
                    }
                )
            }
            _ => Action::keep(node)
        }
    }

}

fn transform_node_single_pass(
    node : Node,
    parent_id : Option<NodeId>,
    transformer : &mut Box<dyn Visitor>
) -> TransformResult {

    let original_id = node.id;

    let transform_action = transformer.enter(node, parent_id)?;

    match &transform_action.kind {
        ActionKind::Remove => return Ok(transform_action),
        _ => {}
    };

    let transform_action = match transform_action.node {
        // TODO: tidy up NodeKind: split into Leaf (no children) and NonLeaf (with children) to avoid this
        Node { 
            id,
            kind: NodeKind::Env(EnvNode{ header, kind: EnvNodeKind::Open(children) }), 
            position
        } => {
            
            let mut has_changed = false;

            let children = children
                .into_iter()
                .map(
                    |child| transform_node_single_pass(
                        child,
                        Some(id),
                        transformer
                    )
                )
                .collect::<Result<Vec<Action>, VisitError>>()?
                .into_iter()
                // remove children whose transform returned ActionKind::remove
                .filter(
                    |action| match &action.kind { 
                        ActionKind::Remove => { has_changed = true; false }, 
                        ActionKind::Replace => { has_changed = true; true }, 
                        ActionKind::Keep => { true }
                    }
                )
                .map(|action| action.node)
                .collect::<Vec<Node>>();

            let node = Node {
                id,
                kind: NodeKind::Env(EnvNode::new_open(header, children)),
                position
            };

            if has_changed { Action::replace(node) } else { Action::keep(node) }
        },
        _ => transform_action
    };

    transformer.leave(&transform_action.node, original_id, parent_id);

    Ok(transform_action)
}

///
/// Transforms the tree until all transformers return Action::keep
/// or max_passes is reached.
/// 
pub fn transform(
    node : Node,
    transformers : &mut Vec<Box<dyn Visitor>>,
    max_passes : u32
) -> Result<Node, VisitError> {

    let mut action = Action::replace(node);

    let mut iterations : u32 = 0;

    loop {
        for transformer in transformers.iter_mut() {
            
            action = match &action.kind {
                ActionKind::Keep | ActionKind::Replace => transform_node_single_pass(
                    action.node, 
                    None,
                    transformer
                )?,
                ActionKind::Remove => return Err(VisitError::RootRemoved),
            }

        }

        match &action.kind  {
            ActionKind::Keep => {
                return Ok(action.node)
            },
            _ => {
                iterations += 1;

                if iterations > max_passes {
                    return Err(VisitError::MaxIterationsReached)
                }
            }
        }
    }
}

pub struct DefaultTransformer;

// default transformer that is always active
impl Visitor for DefaultTransformer {

    fn enter(&mut self, node : Node, _parent_id : Option<NodeId>) -> TransformResult {

        match &node.kind {
            NodeKind::Leaf(LeafNode::Comment(_)) => Ok(Action::remove(node)),
            _ => Ok(Action::keep(node))
        }

    }

}

#[cfg(test)]
mod test {

    struct EquationTransformer;

    // puts any equations into a <pre> tag
    impl Visitor for EquationTransformer {

        fn enter(&mut self, node : Node, parent_id : Option<NodeId>) -> TransformResult {

            match &node.kind {
                // match equations
                NodeKind::Env(
                    EnvNode{
                        header: EnvNodeHeader{ 
                            kind: EnvNodeHeaderKind::Eq(equation_kind), 
                            attrs: _, 
                            meta_attrs: _
                        }, 
                        kind: EnvNodeKind::Open(children) 
                    }
                ) => {
                    // TODO: unwrap
                    // TODO: check that only one child exists
                    let child = children.get(0).unwrap();

                    if let NodeKind::Leaf(LeafNode::Text(text)) = &child.kind {
                        let raw_node = Node {
                            kind: NodeKind::Leaf(LeafNode::Text(
                                match equation_kind {
                                    EquationKind::Block => format!("<p><pre>{}</pre></p>", text),
                                    EquationKind::Inline => format!("<pre>{}</pre>", text),
                                },
                            )),
                            ..node
                        };

                        Ok(Action::replace(raw_node))
                    } else {
                        Ok(Action::remove(node))
                    }
                },
                _ => Ok(Action::keep(node))
            }

        }

    }


    use super::*;
    use crate::parse;

    #[test]
    fn transform_node() {

        let (document, _) = parse::parse(r#"
            Text with an equation $\nu Te \mathcal{X}$. 
            /** Comment */
            <Chapter>
                Contents
                <Eq>
                    \nu Te \mathcal{X}
                </Eq>
            </Chapter>
            <Eq>
                e = mc^2
            </Eq>
            "#
        );

        let document = transform(
            document, 
            &mut vec![Box::new(DefaultTransformer), Box::new(EquationTransformer)],
            3
        ).unwrap();

        // TODO: check that comments are removed

        // dbg!(&document);

        let mut collected_bytes = Vec::new();

        // Create a closure that appends bytes to the Vec<u8>
        let mut collect_closure = |bytes: &[u8]| {
            collected_bytes.extend_from_slice(bytes);
        };

        // document.collect_bytes(&mut collect_closure).unwrap();

        // println!("Result: \n{}", std::str::from_utf8(collected_bytes.as_slice()).unwrap());

    }

}
