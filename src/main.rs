
use vtx::parse::*;
use vtx::plugins::html_emit::HTMLEmitter;
use vtx::visit::transform;
use vtx::visit::TransformerOnce;

use std::io::Read;

fn stdout_collector(s : &str) {
    print!("{}", s);
}

fn main() {

    let mut src : String = String::from("");

    std::io::stdin().read_to_string(&mut src).unwrap();

    let (document, _) = parse(&src);

    /*
    let document = transform(
        document,
        &mut vec![
            Box::new(TransformerOnce::new(HTMLPlugin)), 
            Box::new(TransformerOnce::new(KatexPlugin::hosted()))
        ],
        3
    ).unwrap();
     */

    let _ = transform(document, &mut vec![
        Box::new(TransformerOnce::new(HTMLEmitter{ collector: stdout_collector })),
    ], 1);

}
