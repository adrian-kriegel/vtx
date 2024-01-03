
use std::str::Chars;

use crate::document::*;
use crate::parse_error::*;

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct ParserPosition {
    line: usize,
    col: usize,
}

impl Clone for ParserPosition {
    fn clone(&self) -> ParserPosition {
        Self{ line: self.line, col: self.col }
    }
}

impl Ord for ParserPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            core::cmp::Ordering::Equal => self.col.cmp(&other.col),
            ord => return ord,
        }   
    }
}

pub struct Parser<'a>{
    /** Remaining source string. */
    remaining : &'a str,
    /** Char iterator over the source string. */
    iter : Chars<'a>,
    /** Current position in the source string. */
    position: ParserPosition,
    /** Tokens parsed so far (until position). */
    parsed_tokens: Vec<Token<'a>>
}

pub struct TokenHandle(usize);

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind{
    EnvOpen,
    EnvClose(String),
    EnvSelfClose,
    RightAngle,
    StringToken(String),
    CommentOpen,
    CommentClose,
    Whitespace,
    EndOfLine,
    EndOfFile,
    Dollar,
    // TODO: these are non-matchable tokens that are only parsed when capturing
    //       separate matchable from non-matchable tokens
    Text,
    Math,
    EnvName
}


#[derive(Debug)]
pub struct Token<'a> {
    pub value: &'a str,
    pub kind: TokenKind,
    pub position: ParserPosition
}

impl TokenKind {

    fn parse<'a>(&self, parser : &mut Parser<'a>) -> Option<&'a str> {
        
        let bytes = parser.remaining.as_bytes();

        let value = match self {

            Self::StringToken(s) if parser.remaining.starts_with(s) 
            => Some(&parser.remaining[..s.len()]),

            Self::StringToken(_) => None,

            Self::EnvOpen => (
                bytes[0] == b'<' && 
                bytes.len() > 1 && (
                    (bytes[1] >= b'a' && bytes[1] <= b'z') ||
                    (bytes[1] >= b'A' && bytes[1] <= b'Z')
                )
            ).then(||&parser.remaining[..1]),

            Self::Whitespace => {
                let whitespace_len = parser.remaining
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count();

                (whitespace_len > 0).then(|| &parser.remaining[..whitespace_len])
            },

            Self::EndOfFile => (parser.remaining.len() == 0)
                .then(|| ""),

            Self::Dollar => (bytes[0] == b'$' )
                .then(|| &parser.remaining[..1]),

            Self::EndOfLine => (bytes[0] == b'\n')
                .then(|| &parser.remaining[..1]),

            Self::CommentOpen => parser.remaining.starts_with("/**")
                .then(|| "/**"),

            Self::CommentClose => parser.remaining.starts_with("*/")
                .then(|| "*/"),

            Self::EnvSelfClose => parser.remaining.starts_with("/>")
                .then(|| "/>"),

            Self::RightAngle => parser.remaining.starts_with(">")
                .then(|| ">"),

            Self::EnvClose(closer) => parser.remaining.starts_with(closer).then( ||&parser.remaining[..closer.len()]),

            // Text and Math kind can never be used for matching 
            // as it would match anything
            TokenKind::Text | TokenKind::Math | TokenKind::EnvName  => panic!(
                "Cannot use non-matchable token for matching."
            ),
        };

        match value {
            Some(s) => {
                parser.skip(s.len());
            },
            _ => {}
        }

        value

    }
}

impl<'a> Token<'a> {

    pub fn len(&self) -> usize {
        self.value.len()
    } 

}

impl<'a> Parser<'a> {

    /** Create a new parser from a source slice. */
    pub fn new(src : & 'a str) -> Self {
        Parser { 
            iter: src.chars(), 
            remaining: src, 
            position: ParserPosition { line: 0, col: 0 },
            parsed_tokens: Vec::new(),
        }
    }

    /**
     * Retrieves the next char from the iterator.
     */
    fn next_char(&mut self) -> Option<char> {

        let c = self.iter.next()?;
        
        if c == '\n' {
            self.position.line += 1;
            self.position.col = 0
        } else {
            self.position.col += 1;
        }

        self.remaining = &self.remaining[1..];

        Some(c)   
    }

    /**
     * Moves the iterator by n chars.
     */
    fn skip(&mut self, n : usize) {
        
        // TODO
        for _ in 0..n {
            self.next_char();
        }
    }

    /**
     * Moves the iterator to the next unescaped char.
     */
    fn next_unescaped_char(&mut self) -> Option<char> {

        let c = self.next_char()?;

        if c == '\\' { 
            self.next_char() 
        } else { 
            Some(c) 
        }
    }

    fn get_token(&self, handle : TokenHandle) -> &Token<'a> {
        self.parsed_tokens.get(handle.0).unwrap()
    }

    fn get_token_option(&self, handle : Option<TokenHandle>) -> Option<&Token<'a>> {
        handle.map(|handle| self.get_token(handle))
    }

    /**
     * Same as seek_to but also captures all skipped chars 
     * in token with captured_kind.
     * Returns indices to the captured tokens.
     */
    fn seek_to_and_capture(
        &mut self, 
        captured_kind : TokenKind,
        tokens : &[TokenKind],
    ) -> (Option<TokenHandle>, Option<TokenHandle>) {

        let prev_position = self.position.clone();

        let prev_remaining = self.remaining;

        let (idx, end_token) = self.seek_to(tokens);
        
        let end_token = end_token.map(
            |(kind, value)| self.push_token(Token { 
                value, 
                kind, 
                position: self.position.clone()
            })
        );

        if idx > 0 {
            let captured_kind = self.push_token(Token { 
                position: prev_position, 
                value: &prev_remaining[..idx], 
                kind: captured_kind
            });
    
            (Some(captured_kind), end_token)
        } else {
            (None, end_token)
        }
    }   

    /**
     * Pushes token into parsed_tokens and returns a TokenHandle.
     */
    fn push_token(&mut self, token : Token<'a>) -> TokenHandle {
        self.parsed_tokens.push(token);
        TokenHandle(self.parsed_tokens.len() - 1)
    }

    /**
     * Moves the iterator right behind the first matching token.
     */
    fn seek_to(&mut self, tokens : &[TokenKind])  -> (usize, Option<(TokenKind, & 'a str)>) {
        
        let mut idx : usize = 0;
        
        loop {

            for kind in tokens {
                
                if let Some(value) = kind.parse(self) {
                    return (
                        idx, 
                        Some((kind.clone(), value))
                    )
                }
            }

            self.next_unescaped_char();
            
            idx += 1;

            if self.remaining.len() == 0 {
                break
            }
        };

        (idx, None)
    }


    /** Parse children of env node terminated by closing_tag. */
    pub fn parse_children(
        &mut self,
        closing_tag : TokenKind,
    ) -> Result<Vec<Node>, ParseError> {

        let mut children = Vec::new();
        
        let start_position = self.position.clone();

        loop {

            let (text, stop_token) = self.seek_to_and_capture(
                    TokenKind::Text,
                    &[
                        closing_tag.clone(), 
                        TokenKind::EnvOpen, 
                        TokenKind::Dollar,
                        TokenKind::CommentOpen
                    ],
                );

            let stop_token = self.get_token(stop_token.ok_or(
                ParseError::env_not_closed(&closing_tag, &start_position)
            )?);
            
            self.get_token_option(text).map(
                |token| children.push(Node::new_text(token))
            );

            let stop_kind = stop_token.kind.clone();
            let stop_position = stop_token.position.clone();

            let kind = match stop_kind {

                _ if stop_kind == closing_tag => break,
                
                TokenKind::EnvOpen => NodeKind::Env(self.parse_env_from_name()?),

                TokenKind::Dollar => {

                    let (math, dollar) = self.
                        seek_to_and_capture(
                            TokenKind::Math,
                            &[TokenKind::Dollar]
                        );
                    
                    // math may be empty
                    let math = match math {
                        Some(handle) => self.get_token(handle).value,
                        None => "",
                    };

                    dollar.map(
                        |_| NodeKind::InlineEquation(String::from(math))
                    ).ok_or(
                        ParseError::env_not_closed(
                            &TokenKind::Dollar,
                            &stop_position
                        ),
                    )?
                },

                TokenKind::CommentOpen => NodeKind::Comment(
                    self.parse_children(TokenKind::CommentClose)?
                ),

                _ => unreachable!(),
            };
            
            children.push(
                Node{ 
                    position: stop_position, 
                    kind
                }
            );
        }
        
        Ok(children)
    }

    /**
     * Begins parsing an env tag right after the '<'
     * 
     * Example input: "Document></Document>"
     * 
     */
    pub fn parse_env_from_name(&mut self) -> Result<EnvNode, ParseError> {

        let start_position = self.position.clone();

        let (name, stop_token) = self
            .seek_to_and_capture(
                TokenKind::EnvName,
                &[
                    TokenKind::Whitespace,
                    TokenKind::EnvSelfClose, 
                    TokenKind::RightAngle, 
                ]
            );

        // name can be unwrapped: 
        // EnvOpen only matches if followed by a letter
        let name = String::from(self.get_token(name.unwrap()).value);

        let header = EnvNodeHeader::new_empty(name);

        let stop_token = self.get_token(
            stop_token.ok_or(
                ParseError::env_header_not_closed(&start_position)
            )?
        );

        Ok(match stop_token.kind {

            TokenKind::Whitespace 
                => Err(ParseError::todo("parse attrs", &self.position))?,

            TokenKind::EnvSelfClose => EnvNode::new_self_closing(header),

            TokenKind::RightAngle => EnvNode::Open(
                EnvNodeOpen{ 
                    children: self.parse_children(
                        TokenKind::EnvClose(
                            format!("</{}>", header.name)
                        )
                    )?,
                    header,
                }
            ),

            // kind can only be one of the variants passed to seek_to_and_capture
            _ => unreachable!()
        })

    }

    fn parse_document(&mut self) -> Result<Node, ParseError> {
        
        let children = self.parse_children(
            TokenKind::EndOfFile
        )?;

        Ok(Node {
            kind: NodeKind::Root(children),
            position: ParserPosition { line: 0, col: 0 }
        })
    }
    
}

pub fn parse(src : &str) -> Result<(Node, Vec<Token>), ParseError> {
    
    let mut parser = Parser::new(src);

    let document = parser.parse_document()?;

    let mut tokens = parser.parsed_tokens;

    tokens.sort_by(|a, b| a.position.cmp(&b.position));

    Ok((document, tokens))    
}