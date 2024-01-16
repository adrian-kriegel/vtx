
use std::fmt::Display;

use crate::{transform::TransformError, document::EmitError, parse_error::ParseError};

pub enum ErrorKind<'a> {
    Parse(ParseError),
    Transform(TransformError),
    Emit(EmitError<'a>)
}

pub struct Error<'a> {
    src: &'a str,
    kind: ErrorKind<'a>
}

impl<'a> Error<'a> {

    pub fn parse(e : ParseError, src : &'a str) -> Error<'a> {
        Error {
            kind: ErrorKind::Parse(e),
            src,
        }
    }

    pub fn transform(e : TransformError, src : &'a str) -> Error<'a> {
        Error {
            kind: ErrorKind::Transform(e),
            src,
        }
    }

    pub fn emit(e : EmitError<'a>, src : &'a str) -> Error<'a> {
        Error {
            kind: ErrorKind::Emit(e),
            src,
        }
    }

}

impl <'a> Display for Error<'a> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match &self.kind {
            ErrorKind::Parse(err) => write!(f, "{}", err.display(self.src)),
            ErrorKind::Transform(_) => todo!(),
            ErrorKind::Emit(_) => todo!(),
        }   

    }
}
