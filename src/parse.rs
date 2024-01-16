
use std::str::Chars;
use std::vec;

use crate::document::*;
use crate::parse_error::*;

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct ParserPosition {
    // index of the current line in the module 
    line: usize,
    // index of the current character in the current line
    col: usize,
    // offset of the current char (from the source start) in bytes
    byte_idx: usize,
}

impl Clone for ParserPosition {
    fn clone(&self) -> ParserPosition {
        Self{ line: self.line, col: self.col, byte_idx: self.byte_idx }
    }
}

impl Ord for ParserPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.byte_idx.cmp(&other.byte_idx)
    }
}

pub struct TokenStorage<'a> {
    tokens: Vec<Token<'a>>
}

pub struct Parser<'a>{
    /** Remaining source string. */
    remaining : &'a str,
    /** Char iterator over the source string. */
    iter : Chars<'a>,
    /** Current position in the source string. */
    position: ParserPosition,
    /** Tokens parsed so far (until position). */
    parsed_tokens: TokenStorage<'a>
}

pub struct TokenHandle(usize);

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind{
    EnvOpen,
    EnvClose(String),
    EnvSelfClose,
    RightAngle,
    CommentOpen,
    CommentClose,
    Whitespace,
    EndOfLine,
    EndOfModule,
    Dollar,
    Equals,
    Quote,
    // TODO: these are non-matchable tokens that are only parsed when capturing
    //       separate matchable from non-matchable tokens
    Text,
    CommentText,
    Math,
    EnvName,
    AttrName,
    StringLiteral,
}


#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub value: &'a str,
    pub kind: TokenKind,
    pub position: ParserPosition
}

impl ParserPosition {

    pub fn zero() -> Self {
        Self { line: 0, col: 0, byte_idx: 0 }
    }

    pub fn new(line : usize, col : usize, abs: usize) -> Self {
        Self { line, col, byte_idx: abs }
    }

    //
    // Advances the position by the size of the char.
    // Returns bytes advanved.
    //
    pub fn advance(&mut self, c : &char) -> usize {
        
        if *c == '\n' {
            self.line += 1;
            self.col = 0
        } else {
            self.col += 1;
        }

        let delta_bytes = c.len_utf8();

        self.byte_idx += delta_bytes;

        delta_bytes
    }

    pub fn line(&self) -> &usize { &self.line }
    pub fn col(&self) -> &usize { &self.col }
    pub fn bytes(&self) -> &usize { &self.line }

}

impl TokenKind {

    fn new_env_close(header_kind : &EnvNodeHeaderKind) -> Self {
        TokenKind::EnvClose(header_kind.get_closing_string())
    }
}

impl<'a> Token<'a> {

    pub fn len(&self) -> usize {
        self.value.len()
    } 

}

impl<'a> TokenStorage<'a> {
    
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }
    
    fn get(&self, handle : TokenHandle) -> &Token<'a> {
        self.tokens.get(handle.0).unwrap()
    }

    fn get_option(&self, handle : Option<TokenHandle>) -> Option<&Token<'a>> {
        handle.map(|handle| self.get(handle))
    }

    //
    // Pushes token into the storage and returns a TokenHandle.
    //
    fn push(&mut self, token : Token<'a>) -> TokenHandle {
        self.tokens.push(token);
        TokenHandle(self.tokens.len() - 1)
    }

}

impl<'a> Parser<'a> {

    ///
    /// Create a new parser from a source slice. 
    /// 
    pub fn new(src : & 'a str) -> Self {
        Parser {
            iter: src.chars(), 
            remaining: src, 
            position: ParserPosition::zero(),
            parsed_tokens: TokenStorage::new(),
        }
    }

    ///
    /// Returns next char in the source.
    /// Advances the parser position.
    /// 
    fn next_char(&mut self) -> Option<char> {

        let c = self.iter.next()?;
        
        let delta_bytes = self.position.advance(&c);

        self.remaining = &self.remaining[delta_bytes..];

        Some(c)   
    }

    ///
    /// Moves the current position by n chars
    /// 
    fn skip(&mut self, n : usize) {
        
        for _ in 0..n {
            self.next_char();
        }
    }

    ///
    ///  Moves the iterator to the next unescaped char.
    /// 
    fn next_unescaped_char(&mut self) -> Option<char> {

        let c = self.next_char()?;

        if c == '\\' { 
            self.next_char() 
        } else { 
            Some(c) 
        }
    }

    //
    // Attempts to match the provided token kind to the start of the remaining source.
    // Returns the part of the string that matched the token kind.
    // Advances the position by the length of the matched string.
    //
    fn try_parse_token(&mut self, token: &TokenKind) -> Option<&'a str> {
        
        let bytes = self.remaining.as_bytes();

        let value = match token {

            TokenKind::EnvOpen => (
                bytes[0] == b'<' && 
                bytes.len() > 1 && (
                    (bytes[1] >= b'a' && bytes[1] <= b'z') ||
                    (bytes[1] >= b'A' && bytes[1] <= b'Z')
                )
            ).then(||&self.remaining[..1]),

            TokenKind::Whitespace => {
                let whitespace_len = self.remaining
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count();

                (whitespace_len > 0).then(|| &self.remaining[..whitespace_len])
            },

            TokenKind::EndOfModule => (self.remaining.len() == 1)
                .then(|| ""),

            TokenKind::Dollar => (bytes[0] == b'$' )
                .then(|| &self.remaining[..1]),

            TokenKind::Equals => (bytes[0] == b'=' )
                .then(|| &self.remaining[..1]),

            TokenKind::Quote => (bytes[0] == b'"' )
                .then(|| &self.remaining[..1]),

            TokenKind::EndOfLine => (bytes[0] == b'\n')
                .then(|| &self.remaining[..1]),

            TokenKind::CommentOpen => self.remaining.starts_with("/**")
                .then(|| "/**"),

            TokenKind::CommentClose => self.remaining.starts_with("*/")
                .then(|| "*/"),

            TokenKind::EnvSelfClose => self.remaining.starts_with("/>")
                .then(|| "/>"),

            TokenKind::RightAngle => self.remaining.starts_with(">")
                .then(|| ">"),

            TokenKind::EnvClose(closer) => self.remaining.starts_with(closer).then( 
                || &self.remaining[..closer.len()]
            ),

            // These can never be used for matching 
            // as they would match anything
            // TODO: split TokenKind into matchable and non-matchable
            TokenKind::Text | 
            TokenKind::Math | 
            TokenKind::EnvName | 
            TokenKind::AttrName | 
            TokenKind::CommentText |
            TokenKind::StringLiteral => unreachable!(
                "Cannot use non-matchable token for matching."
            ),
        };

        match value {
            Some(s) => {
                // TODO: this will skip n bytes, not n chars as str.len() is in bytes
                self.skip(s.len());
            },
            _ => {}
        }

        value

    }

    ///
    /// Returns handles to the captured tokens.
    /// Same as seek_to but also captures all skipped chars 
    /// in token with captured_kind.
    /// 
    fn seek_to_and_capture(
        &mut self, 
        captured_kind : TokenKind,
        end_kinds : &[TokenKind],
    ) -> (Option<TokenHandle>, Option<TokenHandle>) {

        let prev_position = self.position.clone();

        let prev_remaining = self.remaining;

        let end_token = self.seek_to(end_kinds);
        
        let end_position = end_token.as_ref().map(
            |token| token.position.byte_idx
        ).unwrap_or(self.position.byte_idx);

        let captured_length = end_position - prev_position.byte_idx;

        let captured_handle = (captured_length > 0).then(
            || self.parsed_tokens.push(Token { 
                value: &prev_remaining[..captured_length], 
                position: prev_position, 
                kind: captured_kind
            })
        );

        let end_handle = end_token.map(
            |token| self.parsed_tokens.push(token)
        );

        (captured_handle, end_handle)
    }   

    /**
     * Moves the iterator right behind the first matching token.
     */
    fn seek_to(&mut self, tokens : &[TokenKind])  -> Option<Token<'a>> {
        
        while self.remaining.len() > 0 {

            for kind in tokens {
                
                let position = self.position.clone();

                if let Some(value) = self.try_parse_token(kind) {
                    return Some(
                        Token {
                            value,
                            kind: kind.clone(),
                            position
                        }
                    );
                }
            }

            self.next_unescaped_char();
        }

        // TODO: this is a shim because the loop above stops before EndOfModule could possibly match
        tokens.contains(&TokenKind::EndOfModule).then(
            || Token {
                value: "",
                kind: TokenKind::EndOfModule,
                position: self.position.clone()
            }
        )

    }

    pub fn parse_comment(&mut self) -> Result<&'a str, ParseError> {

        // TODO: allow nested comments

        let position = self.position.clone();

        let (text, end_token) = self.seek_to_and_capture(
            TokenKind::CommentText,
            &[TokenKind::CommentClose]
        );

        end_token.ok_or(
            ParseError::env_not_closed(
                &TokenKind::CommentClose, 
                &position
            )
        )?;

        Ok(text.map(
            |t| self.parsed_tokens.get(t).value
        ).unwrap_or(""))
    }

    /// 
    /// Parse children of env node terminated by closing_tag.
    /// 
    pub fn parse_children(
        &mut self,
        closing_tag : TokenKind
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

            let stop_token = self.parsed_tokens.get(stop_token.ok_or(
                ParseError::env_not_closed(&closing_tag, &start_position)
            )?);
            
            self.parsed_tokens.get_option(text).map(
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
                        Some(handle) => self.parsed_tokens.get(handle).value,
                        None => "",
                    };

                    dollar.map(
                        |_| NodeKind::Leaf(
                            LeafNode::InlineEquation(String::from(math))
                        )
                    ).ok_or(
                        ParseError::env_not_closed(
                            &TokenKind::Dollar,
                            &stop_position
                        ),
                    )?
                },

                TokenKind::CommentOpen => NodeKind::Leaf(
                    LeafNode::Comment(self.parse_comment()?.to_string())
                ),

                _ => unreachable!(),
            };
            
            children.push(Node::new(kind, NodePosition::Source(stop_position)));
        }
        
        Ok(children)
    }

    ///
    /// Parse env header attributes after the env name
    /// 
    pub fn parse_env_header_attrs(&mut self) -> Result<(EnvNodeAttrs, TokenKind), ParseError> {

        let start_position = self.position.clone();

        let mut attrs = EnvNodeAttrs::new();

        loop {

            let (key, end_token) = self.seek_to_and_capture(
                TokenKind::AttrName,
                &[
                    TokenKind::Equals,
                    TokenKind::Whitespace,
                    TokenKind::EnvSelfClose,
                    TokenKind::RightAngle,
                ]
            );

            let end_token = self.parsed_tokens.get(
                end_token.ok_or(
                    ParseError::env_header_not_closed(&start_position)
                )?
            );

            let end_kind = match end_token.kind {

                TokenKind::Equals => {
                    let position = self.position.clone();

                    let (_, start_quote) = self.seek_to_and_capture(
                        TokenKind::Text,
                        &[TokenKind::Quote]
                    );

                    let start_quote_position = self.parsed_tokens.get(
                        start_quote.ok_or(ParseError::missing_attr_value(&position))?
                    ).position.clone();

                    let (captured, end_quote) = self.seek_to_and_capture(
                        TokenKind::StringLiteral,
                        &[TokenKind::Quote]
                    );

                    end_quote.ok_or(ParseError::quote_not_closed(&start_quote_position))?;

                    let value = captured.map(
                        |t| self.parsed_tokens.get(t).value
                    ).unwrap_or("");

                    let key = key.map(
                        |t| self.parsed_tokens.get(t).value
                    ).unwrap_or("");

                    attrs.insert(key.to_string(), Some(value.to_string()));

                    // skip any whitespace after the value
                    self.try_parse_token(&TokenKind::Whitespace);

                    None
                },

                TokenKind::Whitespace => {
                    let (_, end_token) = self.seek_to_and_capture(
                        TokenKind::AttrName,
                        &[
                            TokenKind::EnvSelfClose,
                            TokenKind::RightAngle,
                        ]
                    );

                    let end_token = self.parsed_tokens.get(
                        end_token.ok_or(
                            ParseError::env_header_not_closed(&start_position)
                        )?
                    );

                    Some(end_token.kind.clone())
                },

                TokenKind::EnvSelfClose | TokenKind::RightAngle => Some(end_token.kind.clone()),

                _ => unreachable!()
            };

            if let Some(end_kind) = end_kind {
                return Ok((attrs, end_kind))
            }
        };

    }

    ///
    /// Parse an env node header starting from the name. 
    /// Example input: "Eq>", "Eq label='eq:my_equation'>"
    /// 
    pub fn parse_env_header_from_name(&mut self) -> Result<(EnvNodeHeader, TokenKind), ParseError> {

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
        let name = self.parsed_tokens.get(name.unwrap()).value;

        let mut header = EnvNodeHeader::new_empty(name);

        let stop_token = self.parsed_tokens.get(
            stop_token.ok_or(
                ParseError::env_header_not_closed(&start_position)
            )?
        );

        let stop_kind = match stop_token.kind {
            TokenKind::Whitespace => {
                
                let (attrs, stop_kind) = self.parse_env_header_attrs()?;

                header.attrs = attrs;

                stop_kind
            },
            _ => stop_token.kind.clone(),
        };

        Ok((header, stop_kind))

    }

    ///
    /// Begins parsing an environment node right after the '<'
    /// Example input: "Document></Document>"
    /// 
    pub fn parse_env_from_name(&mut self) -> Result<EnvNode, ParseError> {

        let (header, stop_token) = self.parse_env_header_from_name()?;
        
        let body_position = self.position.clone();

        Ok(match stop_token {

            TokenKind::EnvSelfClose => EnvNode::new_self_closing(header),

            TokenKind::RightAngle => {
                let children = if header.meta_attrs.raw {
                        
                    let closing_tag = TokenKind::new_env_close(&header.kind);
                    
                    let (text, stop_token) = self.seek_to_and_capture(
                        TokenKind::Text,
                        &[closing_tag.clone()],
                    );

                    stop_token.ok_or(ParseError::env_not_closed(&closing_tag, &body_position))?;

                    if let Some(text) = text {
                        vec![Node::new_text(self.parsed_tokens.get(text))]
                    } else {
                        vec![]
                    }
                } else {
                    self.parse_children(TokenKind::new_env_close(&header.kind))?
                };

                EnvNode::new_open(header, children)
            },

            // kind can only be one of the variants passed to seek_to_and_capture
            _ => unreachable!()
        })

    }

    ///
    /// Returns document node.
    /// Parses entire document.
    /// 
    fn parse_document(&mut self) -> Result<Node, ParseError> {

        let children = self.parse_children(
            TokenKind::EndOfModule
        )?;

        Ok(
            Node::new(
                NodeKind::Env(EnvNode::new_module(children)),
                NodePosition::Source(ParserPosition::zero())
            )
        )
    }
    
}

pub fn parse(src : &str) -> Result<(Node, TokenStorage), ParseError> {
    
    let mut parser = Parser::new(src);

    let document = parser.parse_document()?;

    Ok((document, parser.parsed_tokens))    
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn seek_to_and_capture() {
        
        let end_document = TokenKind::new_env_close(&EnvNodeHeaderKind::Other("Document".to_string()));

        let cases = vec![
            (
                "</Document>",
                TokenKind::Text,
                [end_document.clone()],
                // expected tokens
                (
                    Option::<Token>::None, 
                    Some(
                        Token {
                            position: ParserPosition::zero(),
                            value: "</Document>",
                            kind: end_document.clone()
                        }
                    )
                )
            ),
            (
                "some text abc! <1 </Document>",
                TokenKind::Text,
                [end_document.clone()],
                // expected tokens
                (
                    Some(
                        Token {
                            position: ParserPosition::zero(),
                            value: "some text abc! <1 ",
                            kind: TokenKind::Text
                        }
                    ),
                    Some(
                        Token {
                            position: ParserPosition::new(0, 18, 18),
                            value: "</Document>",
                            kind: end_document.clone()
                        }
                    )
                )
            ),
            (
                "some text\n abc! <1 \\</Document>",
                TokenKind::Text,
                [end_document.clone()],
                // expected tokens
                (
                    Some(
                        Token {
                            position: ParserPosition::zero(),
                            value: "some text\n abc! <1 \\</Document>",
                            kind: TokenKind::Text
                        }
                    ),
                    None
                )
            )
        ];

        for (src, captured_kind, end_kinds, expected) in cases {
            
            let mut parser = Parser::new(src);

            let (captured, end) = parser.seek_to_and_capture(captured_kind, &end_kinds);

            let lines = src.lines();
            
            assert_eq!(parser.position.byte_idx, src.len());
            // TODO: also check parser.position.lines
            assert_eq!(parser.position.col, lines.last().unwrap().len());

            assert_eq!(parser.remaining.len(), 0);

            assert_eq!(
                parser.parsed_tokens.get_option(captured),
                expected.0.as_ref(),
            );

            assert_eq!(
                parser.parsed_tokens.get_option(end),
                expected.1.as_ref(),
            );
        }
    }


    #[test]
    fn parse_env_header_attrs() {

        // attrs cannot start with whitespace but may be emtpy
        let cases = vec![
            (
                "/>",
                EnvNodeAttrs::new(),
                TokenKind::EnvSelfClose,
            ),
            (
                "label=\"foo\"/>",
                EnvNodeAttrs::from([
                    ("label".to_string(), Some("foo".to_string())),
                ]),
                TokenKind::EnvSelfClose,
            ),
            (
                "label=\"foo\">",
                EnvNodeAttrs::from([
                    ("label".to_string(), Some("foo".to_string())),
                ]),
                TokenKind::RightAngle,
            ),
            (
                "label=\"foo\"  bar=\"1\" >",
                EnvNodeAttrs::from([
                    ("label".to_string(),Some("foo".to_string())),
                    ("bar".to_string(), Some("1".to_string())),
                ]),
                TokenKind::RightAngle,
            ),
            (
                "label=\"foo\" bar=\"1\">",
                EnvNodeAttrs::from([
                    ("label".to_string(),Some("foo".to_string())),
                    ("bar".to_string(), Some("1".to_string())),
                ]),
                TokenKind::RightAngle,
            ),
            (
                "label=\"foo\"\n\tbar=\"1\"\n />",
                EnvNodeAttrs::from([
                    ("label".to_string(), Some("foo".to_string())),
                    ("bar".to_string(), Some("1".to_string())),
                ]),
                TokenKind::EnvSelfClose,
            ),
        ];

        for (src, expected_attrs, expected_end) in cases {

            let mut parser = Parser::new(src);

            let (attrs, end_token) = parser.parse_env_header_attrs().unwrap();

            assert_eq!(end_token, expected_end);
            
            assert_eq!(attrs, expected_attrs);
        }
            
    }

    #[test]
    pub fn parse_document() {

        let cases = vec![
            (
                r#"
                This is what a document may look like.

                <Eq label="eq_some_label">
                    e = mc^2
                </Eq>

                \<Eq> and \<Code> environents are parsed as "raw". This means they cannot contain other environments.
                
                <Code>
                    Code can be vtx code or anything else and will not throw syntax errors.
                    The parser does not care about bad syntax in ther: <Eq> <Figure> $ /**
                </Code>
                
                <Image
                    src="https://example.com"
                />
                <Chapter>
                    <Section>
                        Equation <ref eq_some_label/> may be included in the test using the \$ sign like this: $e=mc^2$. 
                    </Section>
                </Chapter>

                /** A comment */"#,
                ()
            )
        ];

        for (src, _) in cases {

            // TODO check the resulting document tree
            let (_document, _tokens) = parse(src).unwrap();

            // dbg!(&document);
        }

    } 


}