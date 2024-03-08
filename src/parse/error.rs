
use core::fmt;

use crate::parse::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    EnvHeaderNotClosed,
    EnvNotClosed,
    MissingAttrName,
    MissingAttrValue,
    QuoteNotClosed,
    Unknown,
    ToDo
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
}

impl ParseError {
    
    pub fn unexpected_eof(_end_kinds : &[TokenKind],) -> Self {
        
        ParseError {
            kind: ParseErrorKind::EnvNotClosed,
            message: format!("Environment never closed. Expected TODO: print end_kinds."),
        }
    }

    pub fn env_header_not_closed() -> Self {     
        ParseError {
            kind: ParseErrorKind::EnvHeaderNotClosed,
            message: format!("Expected '>', '/>', or attribute list."),
        }
    }

    pub fn todo(message : &str, ) -> Self {
        ParseError{
            kind: ParseErrorKind::ToDo,
            message: String::from(message),
        }
    }

    pub fn missing_attr_value() -> Self{
        ParseError{
            kind: ParseErrorKind::MissingAttrValue,
            message: String::from("Expected attribute value after '='."),
        }
    }

    pub fn missing_attr_name() -> Self{
        ParseError{
            kind: ParseErrorKind::MissingAttrName,
            message: String::from("Expected attribute name before '='."),
        }
    }

    pub fn invalid_attr_value(name : &str) -> Self{
        ParseError{
            kind: ParseErrorKind::MissingAttrName,
            message: format!("Invalid value for attribute \"{}\".", name),
        }
    }

    pub fn quote_not_closed() -> Self{
        ParseError{
            kind: ParseErrorKind::QuoteNotClosed,
            message: String::from("Quote '\"' not closed."),
        }
    }

    pub fn kind(&self) -> &ParseErrorKind {
        return &self.kind
    }

}
