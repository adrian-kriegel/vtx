
use crate::{transform::{Transformer, Action}, document::*};

use super::html;

struct ResourcesHosted{
    script_src: String,
    style_src: String,
}

enum Resources {
    Hosted(ResourcesHosted)
}

pub struct KatexPlugin {
    resources: Resources
}

impl KatexPlugin {

    pub fn hosted() -> KatexPlugin {
        Self {
            resources: Resources::Hosted(
                ResourcesHosted {
                    script_src: "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js".to_string(),
                    style_src: "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css".to_string(),
                }
            )
        }
    }

}

impl ResourcesHosted {
    
    fn nodes(&self) -> Vec<Node> {

        let script_attrs = EnvNodeAttrs::from([
            ("defer".to_string(), None),
            ("crossorigin".to_string(), Some("anonymous".to_string()))
        ]);

        vec![
            html::script(&self.script_src, &NodePosition::Inserted, script_attrs),
            html::style_sheet(&self.style_src, &NodePosition::Inserted),
        ]
    }

}

impl KatexPlugin {

    fn resource_nodes(&self) -> Vec<Node> {
        match &self.resources {
            Resources::Hosted(res) => res.nodes(),
        }
    }

}

impl Transformer for KatexPlugin {

    fn transform(&self, node : Node) 
    -> crate::transform::TransformResult {

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
            _ => Action::Keep(node),
        };

        Ok(action)
    }

}
