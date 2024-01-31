
use vtx::document::CollectBytes;
use vtx::parse::*;

use std::io::Read;
use std::io::Write;

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

    let mut write = |bytes :&_| {
        std::io::stdout().write(bytes).unwrap();
    };

    document.collect_bytes(&mut write).unwrap();

}
