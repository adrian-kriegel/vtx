///
/// Visitor/transformer for evaluating variable expressions.
///

use std::collections::HashMap;

use crate::document::{
    EnvNode, 
    EnvNodeHeader, 
    EnvNodeHeaderKind, 
    EnvNodeKind, 
    LeafNode, 
    Node, 
    NodeId, 
    NodeKind,
    visit::{Action, TransformResult, VisitError, Visitor}
};

struct Scope {
    /// The Node this stack belongs to
    node_id: NodeId,
    /// Values in the Scope
    values: HashMap<String, Node>,
}

pub struct Variables {
    ///
    /// This changes as part of the visitor state.
    /// Represents a stack of scopes that grows with every set of variables introduced in a Node.
    /// The stack does not grow if a node does not define any variables.
    /// The stack is popped when leaving the node.
    /// 
    scopes: Vec<Scope>
}

impl Variables {

    pub fn new() -> Self {
        Variables {
            scopes: Vec::new()
        }
    }

    pub fn resolve(&self, name : &String) -> Option<&Node> {
        for scope in self.scopes.iter().rev() {
            let value = scope.values.get(name);

            match &value {
                Some(_) => return value,
                _ => continue
            }
        }

        return None
    }

    pub fn define(&mut self, node_id: NodeId, name : String, value : Node) {

        // find the target scope
        let scope = self.scopes
            .last_mut()
            .filter(|s| s.node_id == node_id);

        match scope {
            // target scope exists: define the variable
            Some(scope) => { scope.values.insert(name, value); },
            // target scope does not exist: grow the stack
            None => self.scopes.push(Scope {
                node_id,
                values: HashMap::from([(name, value)])
            })
        }
    }

}

impl Visitor for Variables {

    fn enter(&mut self, node : Node, parent_id : Option<NodeId>) -> TransformResult {
        match &node.kind {
            // a variable is being used
            NodeKind::Leaf(LeafNode::VariableExpression(expr)) => {
                
                let value = self.resolve(&expr).ok_or(
                    VisitError::Unknown(
                        format!("Cannot resolve variable \"{}\".", expr)
                    ),
                )?;

                Ok(Action::replace(Node {
                    id: Node::generate_id(),
                    ..value.clone()
                }))
            },
            // a variable is being defined
            NodeKind::Env(
                EnvNode { 
                    header: EnvNodeHeader { 
                        attrs, 
                        kind: EnvNodeHeaderKind::Other(name),
                        ..
                    },
                    kind: env_node_kind,
                    ..
                    // TODO: should "var" be an internal type? 
                }
            ) if name == "var" => {
                
                // this is OK because var cannot be the root node of a document
                let parent_id = parent_id.unwrap();

                let (key, value) = attrs.iter().next().ok_or(
                    VisitError::Unknown("Variable definition empty.".to_string())
                )?;

                let value = match &env_node_kind {
                    // <var name>value</var>
                    EnvNodeKind::Open(children) => {
                        if children.len() == 1 {
                            children.front()
                        } else {
                            dbg!(children);
                            todo!("Variable definitions must have exactly one child.");
                        }
                    },
                    // <var name="value" />
                    EnvNodeKind::SelfClosing => value.as_ref(),
                };

                let value = value.ok_or(
                    VisitError::Unknown(
                        String::from(
                            format!("Empty variable definition for {}", key)
                        )
                    )
                )?;

                self.define(parent_id, key.to_string(), value.clone());

                Ok(Action::remove(node))
            }
            _ => Ok(Action::keep(node))
        }

    }

    fn leave(&mut self, _ : &Node, node_id : NodeId, _ : Option<NodeId>) {
        match self.scopes.last() {
            Some(scope) 
                if scope.node_id == node_id => { self.scopes.pop(); },
            _ => {}
        }
    }

}

// TODO: test
