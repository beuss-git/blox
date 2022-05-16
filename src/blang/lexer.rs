use std::collections::HashMap;
pub struct Lexer {
    pub source: String,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    keywords: HashMap<&'static str, TokenType>,
}
// TODO: move this
fn is_alpha(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_digit(c: char) -> bool {
    return c.is_digit(10);
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from([
                ("and", TokenType::And),
                ("or", TokenType::Or),
                ("class", TokenType::Class),
                ("super", TokenType::Super),
                ("this", TokenType::This),
                ("if", TokenType::If),
                ("else", TokenType::Else),
                ("true", TokenType::True),
                ("false", TokenType::False),
                ("fun", TokenType::Fun),
                ("return", TokenType::Return),
                ("for", TokenType::For),
                ("while", TokenType::While),
                ("break", TokenType::Break),
                ("continue", TokenType::Continue),
                ("var", TokenType::Var),
                ("nil", TokenType::Nil),
                ("print", TokenType::Print),
                ("sleep", TokenType::Sleep),
            ]),
        }
    }

    pub fn get_lexeme(&self, token: &Token) -> String {
        return self.source[token.start..token.start + token.length].to_string();
    }

    pub fn scan_token(&mut self) -> Result<Token, LexerError> {
        // Advance to the next valid character
        self.skip_whitespace();

        self.start = self.current;

        // Check if EOF
        if self.is_at_end() {
            return Ok(self.make_token(TokenType::Eof));
        }

        let c = self.advance();

        if is_alpha(c) {
            return Ok(self.identifier());
        }
        // We only support decimals and integers
        if is_digit(c) {
            return Ok(self.number());
        }

        match c {
            '(' => Ok(self.make_token(TokenType::LeftParen)),
            ')' => Ok(self.make_token(TokenType::RightParen)),
            '{' => Ok(self.make_token(TokenType::LeftBrace)),
            '}' => Ok(self.make_token(TokenType::RightBrace)),
            ';' => Ok(self.make_token(TokenType::Semicolon)),
            ',' => Ok(self.make_token(TokenType::Comma)),
            '.' => Ok(self.make_token(TokenType::Dot)),
            '-' => Ok(self.make_token(TokenType::Minus)),
            '+' => Ok(self.make_token(TokenType::Plus)),
            '/' => Ok(self.make_token(TokenType::Slash)),
            '*' => Ok(self.make_token(TokenType::Star)),

            '!' => Ok(self.make_token_compound('=', TokenType::BangEqual, TokenType::Bang)),
            '=' => Ok(self.make_token_compound('=', TokenType::EqualEqual, TokenType::Equal)),
            '<' => Ok(self.make_token_compound('=', TokenType::LessEqual, TokenType::Less)),
            '>' => Ok(self.make_token_compound('=', TokenType::GreaterEqual, TokenType::Greater)),

            '"' => self.string(),
            _ => Err(LexerError::new("Unexpected character", self.line)),
        }
    }

    fn make_token_compound(&mut self, c: char, a: TokenType, b: TokenType) -> Token {
        if self.match_char(c) {
            self.make_token(a)
        } else {
            self.make_token(b)
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        let mut token = Token::new(token_type);
        token.start = self.start;
        token.length = self.current - self.start;
        token.line = self.line;

        token
    }

    fn skip_comment(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.peek().is_whitespace() {
            match self.peek() {
                '\n' => self.line += 1, // Increment line count
                '/' => {
                    // Skip comment
                    if self.peek_next() == '/' {
                        self.skip_comment();
                    }
                }
                _ => (),
            }

            self.advance();
        }
    }

    fn identifier(&mut self) -> Token {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        match self.keywords.get(text) {
            Some(token_type) => self.make_token(*token_type),
            None => self.make_token(TokenType::Identifier),
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

        Ok(self.make_token(TokenType::String))
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

        self.make_token(TokenType::Number)
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

pub struct Token {
    pub t_type: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
}

impl Token {
    pub fn new(t_type: TokenType) -> Self {
        Self {
            t_type,
            start: 0,
            length: 0,
            line: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
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

    Eof,
}
