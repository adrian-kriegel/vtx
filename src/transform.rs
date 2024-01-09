
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
    fn transform_node(&self, node : Node) -> TransformResult;
}

fn transform_node_single_pass(
    node : Node,
    transformers : &Vec<Box<dyn Transformer>>
) -> TransformResult {

    let mut transform_action = Action::Keep(node);

    for visitor in transformers {
        match transform_action {
            Action::Keep(node) | Action::Replace(node) => {
                transform_action = visitor.transform_node(node)?
            },
            Action::Remove => break
        }
    }

    match transform_action {
        // TODO: tidy up NodeKind: split into Leaf (no children) and NonLeaf (with children) to avoid this
        // TODO: also split up actions into WithNode and WithoutNode or similar
        Action::Keep(
            Node { 
                kind: NodeKind::Env(EnvNode{ header, kind: EnvNodeKind::Open(children) }), 
                position
            }
        ) | 
        Action::Replace(
            Node { 
                kind: NodeKind::Env(EnvNode{ header, kind: EnvNodeKind::Open(children) }), 
                position
            }
        ) => {
            
            let mut has_changed = false;

            let children = children
                .into_iter()
                .map(|child| transform_node_single_pass(child, transformers))
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
                kind: NodeKind::Env(EnvNode::new_open(header, children)),
                position
            };
        

            Ok(if has_changed { Action::Replace(node) } else { Action::Keep(node) })
        },
        _ => Ok(transform_action)
    }
}

pub fn transform(
    node : Node,
    transformers : &Vec<Box<dyn Transformer>>,
    max_passes : u32
) -> Result<Node, TransformError> {

    let mut action = Action::Replace(node);

    let mut iterations = 0;

    loop {
        action = match action {
            Action::Replace(node) => transform_node_single_pass(node, transformers)?,
            Action::Keep(node) => return Ok(node),
            Action::Remove => return Err(TransformError::RootRemoved),
        };

        iterations += 1;

        if iterations > max_passes {
            return Err(TransformError::MaxIterationsReached);
        }
    }
}

pub struct DefaultTransformer;

// default transformer that is always active
impl Transformer for DefaultTransformer {

    fn transform_node(&self, node : Node) -> TransformResult {

        match &node.kind {
            NodeKind::Leaf(LeafNode::Comment(_)) => Ok(Action::Remove),
            _ => Ok(Action::Keep(node))
        }

    }

}

struct EquationTransformer;

// puts any equations into a <pre> tag
impl Transformer for EquationTransformer {

    fn transform_node(&self, node : Node) -> TransformResult {

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
        ).unwrap();

        let document = transform(
            document, 
            &vec![Box::new(DefaultTransformer), Box::new(EquationTransformer)], 
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
