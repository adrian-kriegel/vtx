
use crate::{visit::VisitError, parse_error::ParseError};

pub enum ErrorKind {
    Parse(ParseError),
    Transform(VisitError)
}

pub struct Error<'a> {
    src: &'a str,
    kind: ErrorKind
}

impl<'a> Error<'a> {

    pub fn parse(e : ParseError, src : &'a str) -> Error<'a> {
        Error {
            kind: ErrorKind::Parse(e),
            src,
        }
    }

    pub fn transform(e : VisitError, src : &'a str) -> Error<'a> {
        Error {
            kind: ErrorKind::Transform(e),
            src,
        }
    }

}

