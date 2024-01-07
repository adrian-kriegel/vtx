
use crate::parse::{ParserPosition, TokenKind};

#[derive(Debug)]
pub enum ParseErrorKind {
    EnvHeaderNotClosed,
    EnvNotClosed,
    MissingAttrValue,
    QuoteNotClosed,
    Unknown,
    ToDo
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    position: ParserPosition,
    message: String,
}

impl ParseError {
    
    pub fn env_not_closed(closing_tag : &TokenKind, position : &ParserPosition) -> Self {
        
        let closing_tag = match closing_tag {
            // TODO create closable variant for TokenKind
            TokenKind::EnvClose(s) => String::from(s),
            TokenKind::EndOfFile => String::from("EOF"),
            TokenKind::Dollar => String::from("$"),
            TokenKind::CommentClose => String::from("**/"),
            _ => unreachable!("Consturctor env_not_closed can only be used with TokenKind::EnvNotClosed."),
        };

        ParseError {
            kind: ParseErrorKind::EnvNotClosed,
            position: position.clone(),
            message: format!("Environment never closed. Expected {closing_tag}."),
        }
    }

    pub fn env_header_not_closed(position : &ParserPosition) -> Self {     
        ParseError {
            kind: ParseErrorKind::EnvHeaderNotClosed,
            position: position.clone(),
            message: format!("Expected '>', '/>', or attribute list."),
        }
    }

    pub fn todo(message : &str, position : &ParserPosition) -> Self {
        ParseError{
            kind: ParseErrorKind::ToDo,
            position: position.clone(),
            message: String::from(message),
        }
    }

    pub fn missing_attr_value(position : &ParserPosition) -> Self{
        ParseError{
            kind: ParseErrorKind::MissingAttrValue,
            position: position.clone(),
            message: String::from("Expected attribute value after '='."),
        }
    }

    pub fn quote_not_closed(position : &ParserPosition) -> Self{
        ParseError{
            kind: ParseErrorKind::QuoteNotClosed,
            position: position.clone(),
            message: String::from("Quote '\"' not closed."),
        }
    }

}
