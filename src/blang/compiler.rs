use super::lexer::Token;
use super::opcode;
use super::value::Value;
use super::{chunk::Chunk, lexer::Lexer, lexer::TokenKind, parser::Parser};

pub struct Compiler<'a> {
    parser: Parser,
    lexer: Lexer,
    current_chunk: &'a mut Chunk,
}

impl<'a> Compiler<'a> {
    /*fn compile(source: &str) -> Result<Chunk, CompilerError> {
        /et mut chunk = Chunk::new();
        let mut parser = Parser::new(source);
        while let Some(statement) = parser.parse_statement() {
            //println!("{:?}", statement);
            chunk.write_statement(statement);
        }
        Ok(chunk)
    }
    */
    pub fn new(source: String, chunk: &'a mut Chunk) -> Self {
        Self {
            parser: Parser::new(),
            lexer: Lexer::new(source),
            current_chunk: chunk,
        }
    }

    pub fn compile(&mut self) -> bool {
        self.parser.had_error = false;
        self.parser.panic_mode = false;
        // Consume the first token.
        self.advance();

        while !self.match_token(TokenKind::Eof) {
            self.declaration();
        }

        self.end_compiler();

        !self.parser.had_error
    }

    pub fn disassemble(&self) {
        self.current_chunk.disassemble_chunk("code");
    }

    // TODO: move to parser?
    fn error(&mut self, message: &str) {
        self.error_at(self.parser.previous.line, message);
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.parser.current.line, message);
    }

    fn error_at(&mut self, line: usize, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        println!("[line {}] Error: {}", line, message);
        self.parser.had_error = true;
    }
    fn advance(&mut self) {
        self.parser.previous = self.parser.current;
        loop {
            match self.lexer.scan_token() {
                Ok(token) => {
                    self.parser.current = token;
                    // TODO: Fix this hack
                    if self.parser.previous.kind == TokenKind::Eof {
                        self.parser.previous = self.parser.current;
                    }
                    break;
                }
                Err(err) => {
                    self.error_at(err.line, err.message);
                }
            }
        }
    }
    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.parser.current.kind == kind {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        self.parser.current.kind == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if !self.check(kind) {
            return false;
        }
        self.advance();
        true
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk
            .write_byte(byte, self.parser.previous.line);
    }
    /*fn emit_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.emit_byte(*byte);
        }
    }*/
    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(opcode::OP_RETURN);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant_index = self
            .current_chunk
            .add_constant(value, self.parser.previous.line);

        if constant_index > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
        }
        constant_index as u8
    }
    fn emit_constant(&mut self, constant: Value) {
        let constant_index = self.make_constant(constant);
        self.emit_bytes(opcode::OP_CONSTANT, constant_index);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }
    fn string(&mut self) {
        let token = &self.parser.previous;
        let lexeme = self.lexer.get_lexeme(token);
        // Remove the quotes
        let trimmed = &lexeme[1..lexeme.len() - 1];

        self.emit_constant(Value::String(trimmed.to_string()));
    }
    fn named_variable(&mut self, name: Token) {
        let constant_index = self.identifier_constant(name);
        self.emit_bytes(opcode::OP_GET_GLOBAL, constant_index);
    }
    fn variable(&mut self) {
        self.named_variable(self.parser.previous);
    }
    fn number(&mut self) {
        let token = &self.parser.previous;
        let lexeme = self.lexer.get_lexeme(token);
        let value = lexeme.parse::<Value>().expect("Failed to parse lexeme");
        self.emit_constant(value);
    }
    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }
    fn expression(&mut self) {
        //self.parser.binary_expression();
        self.parse_expression(Precedence::Assignment);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.match_token(TokenKind::Equal) {
            // Consume the expression
            self.expression();
        } else {
            // If no explicit assignment is made, use the default value nil
            self.emit_byte(opcode::OP_NIL);
        }
        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_byte(opcode::OP_POP);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_byte(opcode::OP_PRINT);
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;
        while self.parser.current.kind != TokenKind::Eof {
            if self.parser.previous.kind == TokenKind::Semicolon {
                return;
            }
            match self.parser.current.kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenKind::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn declaration(&mut self) {
        if self.match_token(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
    }

    fn unary(&mut self) {
        let operator_kind = self.parser.previous.kind;

        // Compile the operand
        self.parse_expression(Precedence::Unary);

        // Emit the operator instruction
        match operator_kind {
            TokenKind::Bang => self.emit_byte(opcode::OP_NOT),
            TokenKind::Minus => self.emit_byte(opcode::OP_NEGATE),
            _ => (),
        }
    }
    fn binary(&mut self) {
        let operator_kind = self.parser.previous.kind;

        let precedence = Precedence::from(operator_kind);

        self.parse_expression(precedence);

        // TODO: make operations such as != >= and <= a single instruction
        match operator_kind {
            TokenKind::BangEqual => self.emit_bytes(opcode::OP_EQUAL, opcode::OP_NOT),
            TokenKind::EqualEqual => self.emit_byte(opcode::OP_EQUAL),
            TokenKind::Greater => self.emit_byte(opcode::OP_GREATER),
            TokenKind::GreaterEqual => self.emit_bytes(opcode::OP_LESS, opcode::OP_NOT),
            TokenKind::Less => self.emit_byte(opcode::OP_LESS),
            TokenKind::LessEqual => self.emit_bytes(opcode::OP_GREATER, opcode::OP_NOT),
            TokenKind::Plus => self.emit_byte(opcode::OP_ADD),
            TokenKind::Minus => self.emit_byte(opcode::OP_SUBTRACT),
            TokenKind::Star => self.emit_byte(opcode::OP_MULTIPLY),
            TokenKind::Slash => self.emit_byte(opcode::OP_DIVIDE),
            _ => (),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.kind {
            TokenKind::False => self.emit_byte(opcode::OP_FALSE),
            TokenKind::True => self.emit_byte(opcode::OP_TRUE),
            TokenKind::Nil => self.emit_byte(opcode::OP_NIL),
            _ => (),
        }
    }

    fn parse_prefix(&mut self) {
        match self.parser.previous.kind {
            TokenKind::LeftParen => self.grouping(),
            TokenKind::Minus | TokenKind::Bang => self.unary(),
            TokenKind::Number => self.number(),
            TokenKind::String => self.string(),
            TokenKind::True | TokenKind::False | TokenKind::Nil => self.literal(),
            TokenKind::Identifier => self.variable(),
            _ => {
                self.error("Expect prefix expression.");
                return;
            }
        }
    }

    fn parse_infix(&mut self) {
        match self.parser.previous.kind {
            TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::BangEqual
            | TokenKind::EqualEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => self.binary(),
            _ => {
                self.error("Expect infix expression.");
                return;
            }
        }
    }
    fn parse_expression(&mut self, precedence: Precedence) {
        self.advance();

        self.parse_prefix();

        while !self.parser.is_at_end() {
            let next_precedence = Precedence::from(self.parser.current.kind);
            if precedence > next_precedence {
                break;
            }
            self.advance();
            self.parse_infix();
        }
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        let lexeme = self.lexer.get_lexeme(&token);

        self.make_constant(Value::String(lexeme))
    }

    fn parse_variable(&mut self, message: &str) -> u8 {
        // Consume the identifier
        self.consume(TokenKind::Identifier, message);

        // Make identifier constant
        self.identifier_constant(self.parser.previous)
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(opcode::OP_DEFINE_GLOBAL, global);
    }
}

#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > >= <=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl<'a> From<TokenKind> for Precedence {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Minus | TokenKind::Plus => Precedence::Term,
            TokenKind::Slash | TokenKind::Star => Precedence::Factor,
            TokenKind::BangEqual | TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => Precedence::Comparison,
            _ => Precedence::None,
        }
    }
}
