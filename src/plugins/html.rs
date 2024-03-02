
use crate::document::*;
use crate::visit::Action;

pub struct HTMLPlugin;

pub fn element(
    name : &str, 
    children : Vec<Node>, 
    attrs : EnvNodeAttrs,
    position : &NodePosition
) -> Node {
    Node::new(
        NodeKind::Env(
            EnvNode::new_open(
                EnvNodeHeader::new(name, attrs), 
                children
            )
        ),
        position.clone()
    )
}

pub fn empty_element(name : &str, position : &NodePosition) -> Node {
    element(name, Vec::new(), EnvNodeAttrs::new(), position)
}

pub fn script(src : &str, position : &NodePosition, mut attrs : EnvNodeAttrs) -> Node {

    attrs.insert("src".to_string(), Some(src.to_string()));

    Node::new(
        NodeKind::Env(
            EnvNode::new_self_closing(
                EnvNodeHeader::new(
                    "script",
                    attrs
                )
            )
        ),
        position.clone()
    )
}

pub fn style_sheet(href : &str, position : &NodePosition) -> Node {
    Node::new(
        NodeKind::Env(
            EnvNode::new_self_closing(
                EnvNodeHeader::new(
                    "link",
                    EnvNodeAttrs::from([
                        ("href".to_string(), Some(href.to_string())),
                        ("rel".to_string(), Some("stylesheet".to_string()))
                    ])
                )
            )
        ),
        position.clone()
    )
}


impl crate::visit::Visitor for HTMLPlugin {

    fn enter(&mut self, node : Node) 
    -> crate::visit::TransformResult {
        
        match node {
            Node{ 
                kind: NodeKind::Env(
                    EnvNode{ 
                        // TODO: this will cause problems once imports are introduced
                        header: EnvNodeHeader{ 
                            kind: EnvNodeHeaderKind::Module, 
                            attrs, 
                            meta_attrs 
                        },
                        kind: EnvNodeKind::Open(children)
                    }
                ),
                position,
                id
            } => {

                let html_children = vec![
                    empty_element("head", &position),
                    element("body", children, EnvNodeAttrs::new(), &position),
                ];

                let html = Node::new(
                    NodeKind::Env(
                        EnvNode::new_open(
                            EnvNodeHeader::new_default("html"), 
                            html_children
                        )
                    ),
                    NodePosition::Inserted
                );

                let doctype =  Node::new(
                    NodeKind::Leaf(LeafNode::Text("<!DOCTYPE html>".to_string())),
                    NodePosition::Inserted
                );

                Ok(Action::replace(Node {
                    id,
                    kind: NodeKind::Env(
                        EnvNode{ 
                            // TODO: this will cause problems once imports are introduced
                            header: EnvNodeHeader{ 
                                kind: EnvNodeHeaderKind::Module, 
                                attrs, 
                                meta_attrs 
                            },
                            kind: EnvNodeKind::Open(
                                vec![doctype, html]
                            )
                        }
                    ),
                    position,
                }))
            },
            _ => Ok(Action::keep(node)),
        }

    }
}

#[cfg(test)]
mod test {
    use crate::{parse::parse, visit};
    use super::*;

    #[test]
    fn test() {

        let src = "test";

        let (doc, _) = parse(src);

        let _doc = visit::transform(
            doc, 
            &mut vec![Box::new(HTMLPlugin)],
            1
        ).unwrap();

        // TODO
    }

}
