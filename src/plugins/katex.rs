
use crate::{visit::{Visitor, Action}, document::*};

struct ResourcesHosted{
    script_src: String,
    style_src: String,
}

enum Resources {
    Hosted(ResourcesHosted)
}

pub struct RenderSettings {
    inline_class_name: String,
    block_class_name: String
}

pub struct KatexPlugin {
    resources: Resources,
    render_settings: RenderSettings
}


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


impl KatexPlugin {

    pub fn hosted() -> KatexPlugin {
        Self {
            resources: Resources::Hosted(
                ResourcesHosted {
                    script_src: "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js".to_string(),
                    style_src: "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css".to_string(),
                }
            ),
            render_settings: RenderSettings {
                block_class_name: String::from("eq-block"),
                inline_class_name: String::from("eq-inline"),
            }
        }
    }

}

impl ResourcesHosted {
    
    fn nodes(&self) -> Vec<Node> {

        let script_attrs = EnvNodeAttrs::from([
            // ("defer".to_string(), None),
            ("crossorigin".to_string(), Some("anonymous".to_string()))
        ]);

        vec![
            script(&self.script_src, &NodePosition::Inserted, script_attrs),
            style_sheet(&self.style_src, &NodePosition::Inserted),
        ]
    }

}

impl KatexPlugin {

    fn resource_nodes(&self) -> Vec<Node> {
        match &self.resources {
            Resources::Hosted(res) => res.nodes(),
        }
    }

    fn transform_equation(
        &self, 
        id : NodeId,
        math : &str,
        kind : &EquationKind
    ) -> Node {

        // TODO: replace with html node

        let element_id = format!("katex-equation-{}", id);
        let text = format!(
            "<span class=\"{}\" id=\"{element_id}\" /><script>katex.render({}, document.getElementById(\"{element_id}\"));</script>",
            match kind { 
                EquationKind::Inline => &self.render_settings.inline_class_name,
                EquationKind::Block => &self.render_settings.block_class_name 
            },
            serde_json::to_string(math).unwrap()
        );

        Node {
            id,
            kind: NodeKind::Leaf(LeafNode::Text(text)),
            position: NodePosition::Inserted
        }
    }

}

impl Visitor for KatexPlugin {

    fn enter(&mut self, node : Node) 
    -> crate::visit::TransformResult {

        let action = match &node.kind {
            NodeKind::Env(
                EnvNode{ 
                    header: EnvNodeHeader{
                        attrs: _,
                        meta_attrs: _ ,
                        kind: EnvNodeHeaderKind::Other(name),
                    }, 
                    kind: _,
                }
            ) if name == "head" => Action::append_children(
                node, 
                self.resource_nodes(),
            ),
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
                    Action::replace(
                        self.transform_equation(node.id, &text, equation_kind)
                    )
                } else {
                    Action::remove(node)
                }
            }
            _ => Action::replace(node),
        };

        Ok(action)
    }

}
