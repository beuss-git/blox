use std::collections::HashMap;
#[derive(Clone)]
pub struct Lexer {
    pub source: String,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    keywords: HashMap<&'static str, TokenKind>,
}
// TODO: move this
fn is_alpha(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_digit(c: char) -> bool {
    return c.is_digit(10);
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            source: String::from(""),
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from([
                ("and", TokenKind::And),
                ("or", TokenKind::Or),
                ("class", TokenKind::Class),
                ("super", TokenKind::Super),
                ("this", TokenKind::This),
                ("if", TokenKind::If),
                ("else", TokenKind::Else),
                ("true", TokenKind::True),
                ("false", TokenKind::False),
                ("fun", TokenKind::Fun),
                ("return", TokenKind::Return),
                ("for", TokenKind::For),
                ("while", TokenKind::While),
                ("break", TokenKind::Break),
                ("continue", TokenKind::Continue),
                ("var", TokenKind::Var),
                ("nil", TokenKind::Nil),
                ("print", TokenKind::Print),
                ("sleep", TokenKind::Sleep),
            ]),
        }
    }

    pub fn set_source(&mut self, source: String) {
        self.source = source;
    }

    pub fn get_lexeme(&self, token: &Token) -> &str {
        return &self.source[token.start..token.start + token.length];
    }

    pub fn scan_token(&mut self) -> Result<Token, LexerError> {
        // Advance to the next valid character
        self.skip_whitespace();

        self.start = self.current;

        // Check if EOF
        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        let c = self.advance();

        let kind_res = self.match_token(c);
        match kind_res {
            Ok(kind) => Ok(match kind {
                TokenKind::String => match self.string() {
                    Ok(t) => t,
                    Err(e) => return Err(e),
                },
                TokenKind::Identifier => self.identifier(),
                TokenKind::Number => self.number(),
                _ => self.make_token(kind),
            }),
            Err(err) => Err(err),
        }
    }

    fn match_token(&mut self, c: char) -> Result<TokenKind, LexerError> {
        match c {
            '(' => Ok(TokenKind::LeftParen),
            ')' => Ok(TokenKind::RightParen),
            '{' => Ok(TokenKind::LeftBrace),
            '}' => Ok(TokenKind::RightBrace),
            ';' => Ok(TokenKind::Semicolon),
            ',' => Ok(TokenKind::Comma),
            '.' => Ok(TokenKind::Dot),
            '%' => Ok(TokenKind::Percent),
            '-' => Ok(TokenKind::Minus),
            '+' => Ok(TokenKind::Plus),
            '/' => Ok(TokenKind::Slash),
            '*' => Ok(TokenKind::Star),

            '!' => Ok(self.match_either('=', TokenKind::BangEqual, TokenKind::Bang)),
            '=' => Ok(self.match_either('=', TokenKind::EqualEqual, TokenKind::Equal)),
            '<' => Ok(self.match_either('=', TokenKind::LessEqual, TokenKind::Less)),
            '>' => Ok(self.match_either('=', TokenKind::GreaterEqual, TokenKind::Greater)),

            '"' => Ok(TokenKind::String),

            ch if is_digit(ch) => Ok(TokenKind::Number),
            ch if is_alpha(ch) => Ok(TokenKind::Identifier),
            ch if ch.is_whitespace() => Ok(TokenKind::Whitespace),

            _ => Err(LexerError::new("Unexpected character", self.line)),
        }
    }

    fn match_either(&mut self, c: char, a: TokenKind, b: TokenKind) -> TokenKind {
        if self.match_char(c) {
            a
        } else {
            b
        }
    }

    fn make_token(&self, token_type: TokenKind) -> Token {
        let mut token = Token::new(token_type);
        token.start = self.start;
        token.length = self.current - self.start;
        token.line = self.line;

        token
    }

    fn skip_comment(&mut self) {
        self.advance();
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }
    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        self.skip_comment();
                    } else {
                        break;
                    }
                }
                c => {
                    if c.is_whitespace() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn identifier(&mut self) -> Token {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        match self.keywords.get(text) {
            Some(token_type) => self.make_token(*token_type),
            None => self.make_token(TokenKind::Identifier),
        }
    }

    fn string(&mut self) -> Result<Token, LexerError> {
        // Opening quote
        while self.peek() != '"' && !self.is_at_end() {
            // Allow newlines inside strings
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LexerError::new("Unterminated string", self.line));
        }

        // Closing quote
        self.advance();

        Ok(self.make_token(TokenKind::String))
    }

    fn number(&mut self) -> Token {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn get_char(&self, index: usize) -> char {
        self.source
            .chars()
            .nth(index)
            .expect("Failed to get character")
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.get_char(self.current + 1)
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.get_char(self.current)
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        self.get_char(self.current - 1)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
pub struct LexerError {
    pub message: &'static str,
    pub line: usize,
}
impl LexerError {
    pub fn new(message: &'static str, line: usize) -> Self {
        Self { message, line }
    }
}

#[derive(Copy, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub length: usize,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Self {
            kind,
            start: 0,
            length: 0,
            line: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Percent,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Or,
    Class,
    Super,
    This,
    If,
    Else,
    True,
    False, // Also literals
    Fun,
    Return,
    For,
    While,
    Break,
    Continue,
    Var,
    Nil, // Also literal

    Print,
    Sleep,

    Whitespace,
    Eof,
}
