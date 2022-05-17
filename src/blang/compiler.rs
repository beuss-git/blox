use super::lexer::Token;
use super::opcode;
use super::value::Value;
use super::{chunk::Chunk, lexer::Lexer, lexer::TokenKind, parser::Parser};

pub struct Compiler<'a> {
    parser: Parser,
    lexer: Lexer,
    current_chunk: &'a mut Chunk,
    locals: Locals,
}
pub struct Locals {
    stack: Vec<Local>,
    locals_count: u8,
    scope_depth: usize,
}
impl Locals {
    pub fn new() -> Self {
        //let mut stack = Vec::with_capacity(u8::MAX as usize);
        //stack.set_len(u8::MAX as usize);
        Self {
            stack: vec![Local::new(); u8::MAX as usize],
            locals_count: 0,
            scope_depth: 0,
        }
    }
    pub fn is_full(&self) -> bool {
        self.locals_count >= u8::MAX
    }
    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Returns the amount of locals removed from the stack.
    pub fn end_scope(&mut self) -> usize {
        self.scope_depth -= 1;

        let previous_count = self.locals_count;

        // I would have loved to make this more functional, but I'm not sure how to do that with local arrays limited by locals_count.
        // it would have sacrificed performance
        for i in (0..self.locals_count).rev() {
            let local = &self.stack[i as usize];
            if local.depth <= self.scope_depth {
                break;
            }
            self.locals_count -= 1;
        }
        (previous_count - self.locals_count) as usize
    }
    /// Declares a local variable
    pub fn declare(&mut self, name: String) {
        self.stack[self.locals_count as usize] = Local {
            name: name,
            depth: self.scope_depth,
            initialized: false,
        };
        self.locals_count += 1;
    }

    /// Marks the local variable as initialized
    pub fn define(&mut self) {
        self.stack[self.locals_count as usize - 1].initialized = true;
    }

    pub fn contains(&self, name: &str) -> bool {
        // TODO: Optimize, also limit to locals_count
        self.stack
            .iter()
            .rev()
            .any(|local| local.depth == self.scope_depth && local.name == name)
    }
    pub fn index_of(&self, name: &str) -> Option<(usize, bool)> {
        // Start with the most recent local and work backwards
        for i in (0..self.locals_count).rev() {
            let local = &self.stack[i as usize];
            if local.name == name {
                return Some((i as usize, local.initialized));
            }
        }
        None
        /*self.stack.iter().position(|local| {
            //local.depth == self.scope_depth && local.name == name
            // Search at any depth
            local.name == name
        })*/
    }

    /*pub fn declare(&mut self, name: &str) -> usize {
        let index = self.stack.len();
        self.stack.push(Local {
            name: name.to_string(),
            depth: self.scope_depth,
        });
        index
    }
    pub fn index_of(&self, name: &str) -> Option<usize> {
        for (i, local) in self.stack.iter().enumerate().rev() {
            if local.name == name && local.depth == self.scope_depth {
                return Some(i);
            }
        }
        None
    }*/
}
#[derive(Debug, Clone)]
pub struct Local {
    name: String,
    depth: usize,
    initialized: bool,
}

impl Local {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            depth: 0,
            initialized: false,
        }
    }
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
            locals: Locals::new(),
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

    fn emit_jump(&mut self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_bytes(0xff, 0xff);
        // Return the offset of the jump instruction
        self.current_chunk.code.len() - 2
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

    fn patch_jump(&mut self, offset: usize) {
        let jump_offset = self.current_chunk.code.len() - offset as usize - 2;

        if jump_offset > u16::MAX as usize {
            self.error("Jump exceeds 16-bit maximum.");
        }

        // Encode offset into the 16-bit jump instruction
        self.current_chunk.code[offset] = (jump_offset >> 8) as u8;
        self.current_chunk.code[offset + 1] = jump_offset as u8;
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn begin_scope(&mut self) {
        self.locals.begin_scope();
    }
    fn end_scope(&mut self) {
        for _ in 0..self.locals.end_scope() {
            self.emit_byte(opcode::OP_POP);
        }
    }
    fn is_scoped(&self) -> bool {
        self.locals.scope_depth > 0
    }
    fn string(&mut self) {
        let token = &self.parser.previous;
        let lexeme = self.lexer.get_lexeme(token);
        // Remove the quotes
        let trimmed = &lexeme[1..lexeme.len() - 1];
        let trimmed_str = trimmed.to_string();

        self.emit_constant(Value::String(trimmed_str));
    }
    fn resolve_local(&mut self, name: Token) -> Option<usize> {
        let lexeme = self.lexer.get_lexeme(&name);

        match self.locals.index_of(lexeme) {
            Some((index, initialized)) => {
                if !initialized {
                    self.error("Can't read local variable in its own initializer");
                }
                Some(index)
            }
            None => None,
        }
    }
    fn named_variable(&mut self, name: Token, can_assign: bool) {
        // See if we can find a local variable with this name
        let (var_index, get_op, set_op) = match self.resolve_local(name) {
            Some(local_index) => (
                local_index as u8,
                opcode::OP_GET_LOCAL,
                opcode::OP_SET_LOCAL,
            ),
            // Assume it's global
            None => (
                self.identifier_constant(name),
                opcode::OP_GET_GLOBAL,
                opcode::OP_SET_GLOBAL,
            ),
        };

        if can_assign && self.match_token(TokenKind::Equal) {
            // If we match with an equals sign, we know it's a variable assignment
            self.expression();
            self.emit_bytes(set_op, var_index);
        } else {
            // If not it's a variable access
            self.emit_bytes(get_op, var_index);
        }
    }
    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous, can_assign);
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

    fn block(&mut self) {
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            self.declaration();
        }
        self.consume(TokenKind::RightBrace, "Expect '}' after block.");
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
    fn if_statement(&mut self) {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        // Compile condition expression
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(opcode::OP_JUMP_IF_FALSE);
        // Compile statement for if branch
        self.statement();

        // This is to jump over potential else branch after finishing execution of the then statement
        let else_jump = self.emit_jump(opcode::OP_JUMP);

        // Patch the jump to the end of the if branch (that we jump to if condition is false)
        // we now know how long the if branch is
        self.patch_jump(then_jump);

        // Clean up the statement value from stack
        self.emit_byte(opcode::OP_POP);

        if self.match_token(TokenKind::Else) {
            // Compile statement for else branch
            self.statement();
        }
        // Patch the jump to the end of the else statement
        self.patch_jump(else_jump);

        /*self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(opcode::OP_JUMP_IF_FALSE);
        self.emit_byte(opcode::OP_POP);
        self.statement();

        let else_jump = self.emit_jump(opcode::OP_JUMP);

        self.patch_jump(then_jump);
        self.emit_byte(opcode::OP_POP);

        if self.match_token(TokenKind::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);*/
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
        } else if self.match_token(TokenKind::If) {
            self.if_statement();
        } else if self.match_token(TokenKind::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
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

    fn parse_prefix(&mut self, can_assign: bool) {
        match self.parser.previous.kind {
            TokenKind::LeftParen => self.grouping(),
            TokenKind::Minus | TokenKind::Bang => self.unary(),
            TokenKind::Number => self.number(),
            TokenKind::String => self.string(),
            TokenKind::True | TokenKind::False | TokenKind::Nil => self.literal(),
            TokenKind::Identifier => self.variable(can_assign),
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

        let can_assign = precedence <= Precedence::Assignment;
        self.parse_prefix(can_assign);

        while !self.parser.is_at_end() {
            let next_precedence = Precedence::from(self.parser.current.kind);
            if precedence > next_precedence {
                break;
            }
            self.advance();
            self.parse_infix();
        }

        if can_assign && self.match_token(TokenKind::Equal) {
            self.error("Invalid assignment target.");
            // NOTE: I am not sure if this will be valid in all contexts
            self.advance();
        }
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        let lexeme = self.lexer.get_lexeme(&token);

        let constant = lexeme.to_string();
        self.make_constant(Value::String(constant))
    }

    fn add_local(&mut self, name: String) {
        if self.locals.is_full() {
            self.error("Too many local variables in function.");
            return;
        }
        self.locals.declare(name.to_string());
    }
    fn declare_variable(&mut self) {
        // Global
        if !self.is_scoped() {
            return;
        }

        let name = self.lexer.get_lexeme(&self.parser.previous).to_string();

        if self.locals.contains(&name) {
            self.error("Variable with this name already declared in this scope.");
        }

        self.add_local(name);
    }

    fn parse_variable(&mut self, message: &str) -> u8 {
        // Consume the identifier
        self.consume(TokenKind::Identifier, message);

        self.declare_variable();

        if self.is_scoped() {
            return 0;
        }

        // Make identifier constant
        self.identifier_constant(self.parser.previous)
    }

    fn define_variable(&mut self, global: u8) {
        if self.is_scoped() {
            // We are in a scope, so define the local so it is ready for use
            self.locals.define();
            return;
        }
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
