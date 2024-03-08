
use vtx::parse::*;
use vtx::visitors::components::ComponentInsert;
use vtx::visitors::components::ComponentRegister;
use vtx::visitors::html_emit::HTMLEmitter;
use vtx::visitors::cleanup::Cleanup;
use vtx::visitors::variables::Variables;
use vtx::document::visit::transform;
use vtx::document::visit::TransformerOnce;

use std::io::Read;

fn stdout_collector(s : &str) {
    print!("{}", s);
}

fn main() {

    let mut src : String = String::from("");

    std::io::stdin().read_to_string(&mut src).unwrap();

    let (document, _) = parse(&src);

    let document = transform(
        document,
        &mut vec![
            Box::new(TransformerOnce::new(Cleanup)),
            Box::new(TransformerOnce::new(ComponentRegister)),
            Box::new(TransformerOnce::new(ComponentInsert)),
            Box::new(TransformerOnce::new(Variables::new()))
        ],
        1
    ).unwrap();
    
    let _ = transform(document, &mut vec![
        Box::new(TransformerOnce::new(HTMLEmitter{ collector: stdout_collector, debug: true })),
    ], 1);

}
