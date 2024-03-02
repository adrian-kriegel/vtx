

use crate::document::*;
use crate::visitor::*;
use crate::error::*;
use crate::parse::*;

pub fn transpile<'a>(
    src : &'a str,
    transformers : &mut Vec<Box<dyn Visitor>>
) -> Result<Node, Error<'a>> {

    let (document, _) = parse(src);

    transform(
        document,
        transformers,
        1
    ).map_err(
        |e| Error::transform(e, src)
    )
}