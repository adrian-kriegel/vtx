
use tnx;

fn main() {
    let src = "<Document>hello $<div>hi<$/div>/** </** */> */ */<br/></Document>";

    let parsed = tnx::parse::parse(src);

    if let Ok((document, tokens)) = parsed {
        dbg!(&document);
        dbg!(&tokens);
    } else {
        dbg!(&parsed);
    }
}
