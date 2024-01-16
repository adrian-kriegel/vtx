
use vtx::document::CollectBytes;
use vtx::parse::*;
use vtx::plugins::katex::KatexPlugin;
use vtx::plugins::html::HTMLPlugin;

use vtx::transform::*;

use std::io::Read;
use std::io::Write;

fn main() {

    let mut src : String = String::from("");

    std::io::stdin().read_to_string(&mut src).unwrap();

    let parse_result = parse(&src);

    match parse_result {
        Ok((document, _)) => {

            let transformed = transform(
                document,
                &mut vec![
                    Box::new(HTMLPlugin), 
                    Box::new(KatexPlugin::hosted())
                ],
                3
            ).unwrap();

            let mut write = |bytes :&_| {
                std::io::stdout().write(bytes).unwrap();
            };

            transformed.collect_bytes(&mut write).unwrap();

        },
        Err(error) => {

            eprintln!("{}", error.display(&src));

        }   
    }

}
