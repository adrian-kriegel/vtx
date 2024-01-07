
use core::fmt;

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

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    position: ParserPosition,
    message: String,
}

impl ParseError {
    
    pub fn env_not_closed(closing_tag : &TokenKind, position : &ParserPosition) -> Self {
        
        let closing_tag_desc = match closing_tag {
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
            message: format!("Environment never closed. Expected {closing_tag_desc}."),
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

    pub fn display(&self, src: &str) -> String {

        let line = src
            .lines()
            .enumerate()
            .nth(*self.position.line());

        if let Some((_, line)) = line {

            let line_indicator = format!("{} | ", *self.position.line() + 1);

            let space = *self.position.col() + line_indicator.len();

            let spacer = std::iter::repeat(" ")
                .take(space)
                .collect::<String>();

            let desc = self.kind.to_string();

            format!("ParseError ({desc}):\n\n{line_indicator}{line}\n{spacer}↑\n{spacer}{}", self.message)

        } else {
            "TODO".to_string()
        }

    }

}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_display() {

        let src = "line0\nline1\nline2";

        let display = ParseError {
            position: ParserPosition::new(1, 2, 0),
            kind: ParseErrorKind::Unknown,
            message: "Some Message.".to_string(),
        }.display(src);

        assert_eq!(
            display,
            "ParseError (Unknown):\n\n2 | line1\n      ↑\n      Some Message."
        );

    }
}