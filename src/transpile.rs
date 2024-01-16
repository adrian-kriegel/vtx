

use crate::document::*;
use crate::transform::*;
use crate::error::*;
use crate::parse::*;

pub fn transpile<'a>(
    src : &'a str,
    transformers : &mut Vec<Box<dyn Transformer>>
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