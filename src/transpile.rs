

use crate::document::*;
use crate::transform::*;
use crate::error::*;
use crate::parse::*;

pub fn transpile<'a>(
    src : &'a str,
    transformers : &Vec<Box<dyn Transformer>>,
    max_passes : u32
) -> Result<Node, Error<'a>> {

    let (document, _) = parse(src).map_err(
        |e| Error::parse(e, src)
    )?;

    transform(
        document,
        transformers,
        max_passes
    ).map_err(
        |e| Error::transform(e, src)
    )
}