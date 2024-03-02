///
/// Visitor/transformer for evaluating variable expressions.
///

use std::collections::HashMap;

use crate::{
    document::{LeafNode, Node, NodeKind}, 
    visit::{Action, TransformResult, VisitError, Visitor}
};

pub struct Variables {
    values: HashMap<String, Node>
}

impl Variables {

    pub fn new() -> Self {
        Variables {
            values: HashMap::new()
        }
    }

}

impl Visitor for Variables {

    fn enter(&mut self, node : Node) -> TransformResult {
        
        match node.kind {
            NodeKind::Leaf(LeafNode::VariableExpression(expr)) => {
                
                let value = self.values.get(&expr).ok_or(
                    VisitError::Unknown(
                        format!("Cannot resolve variable \"{}\".", expr)
                    ),
                )?;

                Ok(Action::replace(value.clone()))
            },
            _ => Ok(Action::keep(node))
        }

    }

}
