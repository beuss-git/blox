use super::lexer::{Token, TokenKind};
#[derive(Clone)]
pub struct Parser {
    pub current: Token,
    pub previous: Token,
    pub had_error: bool,
    pub panic_mode: bool,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            current: Token::new(TokenKind::Eof),
            previous: Token::new(TokenKind::Eof),
            had_error: false,
            panic_mode: false,
        }
    }
    pub fn is_at_end(&self) -> bool {
        self.current.kind == TokenKind::Eof
    }
}
