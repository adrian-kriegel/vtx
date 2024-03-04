use std::collections::VecDeque;

///
/// Components works by simply transforming the <Component> tag 
/// into a variable definition.
/// Usage of the component is then transformed from 
/// <MyComponent foo="bar">Contents</MyComponent>
/// <> <var foo="bar"/><var children>Contents</var> ${MyComponent} </>
/// 
/// 

use crate::{
    document::{
        EnvNode,
        EnvNodeHeader,
        EnvNodeHeaderKind,
        EnvNodeKind,
        LeafNode,
        Node,
        NodeId,
        NodeKind,
        NodePosition
    }, dynamic_parse::component_name_definition_attrs, visit::{Action, TransformResult, VisitError, Visitor}
};


pub struct ComponentRegister;
pub struct ComponentInsert;

impl Visitor for ComponentRegister {

    fn enter(&mut self, node : Node, _parent_id : Option<NodeId>) -> TransformResult {
        match node.kind {
            // a component is being defined
            NodeKind::Env(
                EnvNode { 
                    header: EnvNodeHeader { 
                        attrs, 
                        kind: EnvNodeHeaderKind::ComponentDefinition,
                        ..
                    },
                    kind: EnvNodeKind::Open(children),
                    ..
                }
            ) => {

                let name = component_name_definition_attrs(&attrs).ok_or(
                    VisitError::Unknown("Component must have a name.".to_string())
                )?;
                
                let children_container = Node {
                    kind: NodeKind::new_fragment(children),
                    ..node
                };

                let node = Node {
                    kind: NodeKind::new_variable_definition(name, children_container),
                    id: Node::generate_id(),
                    position: NodePosition::Inserted
                };

                Ok(Action::replace(node))
            },
            _ => Ok(Action::keep(node))
        }
    }

}

impl Visitor for ComponentInsert {
    fn enter(&mut self, node : Node, _parent_id : Option<NodeId>) -> TransformResult {
        match node.kind {
            NodeKind::Env(
                EnvNode { 
                    header: EnvNodeHeader { 
                        attrs, 
                        kind: EnvNodeHeaderKind::Other(name),
                        ..
                    },
                    kind,
                    ..
                    // TODO: should "var" be an internal type? 
                }
            ) if name.chars().next().map_or(false, |c| c.is_uppercase()) => {
                // capacity of the children container of <></>
                // list of variable definitions and
                // variable insertion of the actual component (+1)
                let mut capacity = attrs.len() + 1;

                let component_children = match kind {
                    EnvNodeKind::Open(children) => Some(children),
                    EnvNodeKind::SelfClosing => None,
                };

                if component_children.is_some() {
                    capacity += 1;
                }

                let mut children = VecDeque::with_capacity(capacity);

                // define all variables from attrs
                for (key, value) in attrs {

                    let value = value.ok_or(
                        VisitError::Unknown(
                            "Component parameters must not be None.".to_string()
                        )
                    )?;

                    children.push_back(Node {
                        kind: NodeKind::new_variable_definition(&key, value),
                        id: Node::generate_id(),
                        position: NodePosition::Inserted
                    });
                }

                match component_children {
                    Some(component_children) => children.push_back(
                        Node::new_variable_definition(
                            "children",
                            Node {
                                kind: NodeKind::new_fragment(component_children),
                                // re-use properties from node 
                                ..node
                            }
                        ),
                    ),
                    None => {}
                };

                // insert the component
                children.push_back(Node {
                    kind: NodeKind::Leaf(
                        LeafNode::VariableExpression(name)
                    ),
                    position: NodePosition::Inserted,
                    id: Node::generate_id(),
                });

                Ok(Action::replace(Node {
                    kind: NodeKind::new_fragment(children),
                    id: Node::generate_id(),
                    position: NodePosition::Inserted,
                }))
            },
            _ => Ok(Action::keep(node)),
        }
    }
}

// TODO: test

