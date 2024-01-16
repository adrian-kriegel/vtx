
use std::collections::HashSet;

use crate::document::*;

#[derive(Debug)]
pub enum TransformError {
    Unknown(String),
    RootRemoved,
    MaxIterationsReached,
}

pub enum Action {
    Keep(Node),
    Replace(Node),
    Remove,
}

pub type TransformResult = Result<Action, TransformError>;

pub trait Transformer {
    
    fn transform(&mut self, node : Node) -> TransformResult;
    
}

pub struct TransformerOnce<T : Transformer> {

    transformer: T,

    visited: HashSet<NodeId>
}

impl<T: Transformer> Transformer for TransformerOnce<T> {

    fn transform(&mut self, node : Node) -> TransformResult {

        if self.visited.contains(&node.id) {
            Ok(Action::Keep(node))
        } else {
            self.visited.insert(node.id);
            self.transformer.transform(node)
        }
    }

}

impl<T : Transformer> TransformerOnce<T> {

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

                Action::Replace(
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
            _ => Action::Keep(node)
        }
    }

}

fn transform_node_single_pass(
    node : Node,
    transformer : &mut Box<dyn Transformer>
) -> TransformResult {

    let transform_action = transformer.transform(node)?;

    match transform_action {
        // TODO: tidy up NodeKind: split into Leaf (no children) and NonLeaf (with children) to avoid this
        // TODO: also split up actions into WithNode and WithoutNode or similar
        Action::Keep(
            Node { 
                id,
                kind: NodeKind::Env(EnvNode{ header, kind: EnvNodeKind::Open(children) }), 
                position
            }
        ) | 
        Action::Replace(
            Node { 
                id,
                kind: NodeKind::Env(EnvNode{ header, kind: EnvNodeKind::Open(children) }), 
                position
            }
        ) => {
            
            let mut has_changed = false;

            let children = children
                .into_iter()
                .map(|child| transform_node_single_pass(child, transformer))
                .collect::<Result<Vec<Action>, TransformError>>()?
                .into_iter()
                .filter(
                    |action| match action { 
                        Action::Remove => { has_changed = true; false }, 
                        Action::Replace(_) => { has_changed = true; true }, 
                        Action::Keep(_) => { true }
                    }
                )
                .map(|action| match action {
                    Action::Replace(node) | Action::Keep(node) => { node },
                    _ => unreachable!()
                })
                .collect::<Vec<Node>>();

            let node = Node {
                id,
                kind: NodeKind::Env(EnvNode::new_open(header, children)),
                position
            };
        
            Ok(if has_changed { Action::Replace(node) } else { Action::Keep(node) })
        },
        _ => Ok(transform_action)
    }
}

///
/// Transforms the tree until all transformers return Action::Keep
/// or max_passes is reached.
/// 
pub fn transform(
    node : Node,
    transformers : &mut Vec<Box<dyn Transformer>>,
    max_passes : u32
) -> Result<Node, TransformError> {

    let mut action = Action::Replace(node);

    let mut iterations : u32 = 0;

    loop {
        for transformer in transformers.iter_mut() {
            
            action = match action {
                Action::Keep(node) | Action::Replace(node) => transform_node_single_pass(
                    node, 
                    transformer
                )?,
                Action::Remove => return Err(TransformError::RootRemoved),
            }

        }

        match action  {
            Action::Keep(node) => {
                return Ok(node)
            },
            _ => {
                iterations += 1;

                if iterations > max_passes {
                    return Err(TransformError::MaxIterationsReached)
                }
            }
        }
    }
}

pub struct DefaultTransformer;

// default transformer that is always active
impl Transformer for DefaultTransformer {

    fn transform(&mut self, node : Node) -> TransformResult {

        match &node.kind {
            NodeKind::Leaf(LeafNode::Comment(_)) => Ok(Action::Remove),
            _ => Ok(Action::Keep(node))
        }

    }

}

struct EquationTransformer;

// puts any equations into a <pre> tag
impl Transformer for EquationTransformer {

    fn transform(&mut self, node : Node) -> TransformResult {

        match &node.kind {
            // match inline equations
            NodeKind::Leaf(LeafNode::InlineEquation(text)) => Ok(
                Action::Replace(
                    Node {
                        kind: NodeKind::Leaf(LeafNode::Text(format!("<pre>{}</pre>", text))),
                        ..node
                    }
                )
            ),
            // match block equations
            NodeKind::Env(
                EnvNode{
                    header: EnvNodeHeader{ 
                        kind: EnvNodeHeaderKind::Eq, 
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
                        kind: NodeKind::Leaf(LeafNode::Text(format!("<p><pre>{}</pre></p>", text))),
                        ..node
                    };

                    Ok(Action::Replace(raw_node))
                } else {
                    Ok(Action::Remove)
                }
            },
            _ => Ok(Action::Keep(node))
        }

    }

}

#[cfg(test)]
mod test {

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

        document.collect_bytes(&mut collect_closure).unwrap();

        println!("Result: \n{}", std::str::from_utf8(collected_bytes.as_slice()).unwrap());

    }

}
