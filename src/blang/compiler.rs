use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use super::lexer::Token;
use super::opcode;
use super::value::{
    function::{Function, FunctionType},
    Value,
};
use super::{chunk::Chunk, lexer::Lexer, lexer::TokenKind, locals::Locals, parser::Parser};

pub struct Compiler {
    parser: Parser,
    lexer: Lexer,
    //current_chunk: &'a mut Chunk, // The current chunk we are compiling into
    pub locals: Locals, // All locals
    function: Function, // Active function being built
    function_type: FunctionType,
    chunks: Vec<Chunk>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut compiler = Self {
            parser: Parser::new(),
            lexer: Lexer::new(),
            //current_chunk: chunk,
            locals: Locals::new(),
            function: Function::new(),
            function_type: FunctionType::Script,
            chunks: Vec::new(),
        };

        // Set for the initial function
        compiler.chunks.push(Chunk::new());

        compiler.locals.declare(String::from("")); // Reserve slot 0 for the vm
        compiler
    }

    pub fn chunk_at_index(&self, index: usize) -> Rc<Chunk> {
        // TODO: don't clone, and don't derive from it in chunk and valuaarray
        Rc::from(self.chunks[index].clone())
    }

    pub fn compile(&mut self, source: String) -> Option<Rc<Function>> {
        self.lexer.set_source(source);

        self.parser.had_error = false;
        self.parser.panic_mode = false;
        // Consume the first token.
        self.advance();

        while !self.match_token(TokenKind::Eof) {
            self.declaration();
        }

        let function = self.end_compiler();

        if self.parser.had_error {
            None
        } else {
            Some(function)
        }
    }

    pub fn disassemble(&self) {
        self.current_chunk().disassemble_chunk("code");
    }

    pub fn current_chunk(&self) -> &Chunk {
        &self.chunks[self.function.chunk_index()]
    }

    pub fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunks[self.function.chunk_index()]
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
        let line_num = self.parser.previous.line;
        self.current_chunk_mut().write_byte(byte, line_num);
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

    fn emit_jump_back(&mut self, to: usize) {
        self.emit_byte(opcode::OP_JUMP_BACK);

        let offset = self.current_chunk().code.len() - to + 2;

        if offset > u16::MAX as usize {
            self.error("Jump exceeds 16-bit maximum.");
        }

        // Encode offset into the 16-bit jump instruction
        self.emit_byte((offset >> 8) as u8);
        self.emit_byte(offset as u8);
    }

    fn emit_jump(&mut self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_bytes(0xff, 0xff);
        // Return the offset of the jump instruction
        self.current_chunk().code.len() - 2
    }

    fn emit_return(&mut self) {
        // Default return value is nil
        self.emit_byte(opcode::OP_NIL);
        self.emit_byte(opcode::OP_RETURN);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let line_num = self.parser.previous.line;
        let constant_index = self.current_chunk_mut().add_constant(value, line_num);

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
        let jump_offset = self.current_chunk().code.len() - offset - 2;

        if jump_offset > u16::MAX as usize {
            self.error("Jump exceeds 16-bit maximum.");
        }

        // Encode offset into the 16-bit jump instruction
        self.current_chunk_mut().code[offset] = (jump_offset >> 8) as u8;
        self.current_chunk_mut().code[offset + 1] = jump_offset as u8;
    }

    // Returns compiled function, the compiler only operates on functions
    fn end_compiler(&mut self) -> Rc<Function> {
        self.emit_return();

        if !self.parser.had_error {
            let chunk_name = if self.function.name().is_empty() {
                "<script>"
            } else {
                &self.function.name()
            };
            self.current_chunk().disassemble_chunk(chunk_name)
        }

        Rc::from(self.function.clone())
    }

    fn begin_scope(&mut self) {
        self.locals.begin_scope();
    }

    fn end_scope_func(&mut self) {
        self.locals.end_scope();
    }
    fn end_scope(&mut self) {
        for _ in 0..self.locals.end_scope() {
            self.emit_byte(opcode::OP_POP);
        }
    }
    fn is_scoped(&self) -> bool {
        self.locals.scope_depth() > 0
    }
    fn string(&mut self) {
        let token = &self.parser.previous;
        let lexeme = self.lexer.get_lexeme(token).to_string();
        // Remove the quotes

        self.emit_constant(Value::String(Rc::from(&lexeme[1..lexeme.len() - 1])));
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

    fn function(&mut self, function_type: FunctionType) {
        let old_function_type = self.function_type;
        let old_function = self.function.clone();
        self.function_type = function_type;
        self.chunks.push(Chunk::new());

        let function_name = self.lexer.get_lexeme(&self.parser.previous).to_string();
        self.function.set_name(function_name.clone());
        self.function.set_chunk_index(self.chunks.len() - 1);

        self.begin_scope();

        self.consume(TokenKind::LeftParen, "Expect '(' after function name.");

        // If we have parameters, add them
        while !self.check(TokenKind::RightParen) {
            self.function.inc_arity();
            if self.function.arity() > 255 {
                self.error_at_current("Can't have more than 255 parameters");
            }
            let param_index = self.parse_variable("Expect parameter name");
            self.define_variable(param_index);
            if self.check(TokenKind::RightParen) {
                break;
            }
            self.consume(TokenKind::Comma, "Expect ',' after parameter.");
        }

        self.consume(
            TokenKind::RightParen,
            "Expect ')' after function parameters.",
        );
        self.consume(TokenKind::LeftBrace, "Expect '{' before function body.");

        // Parse in the body
        self.block();

        let function = self.end_compiler();
        self.end_scope_func();

        self.function_type = old_function_type;
        self.function = old_function;

        let constant = self.make_constant(Value::Function(function));
        self.emit_bytes(opcode::OP_CONSTANT, constant);
    }

    fn function_declaration(&mut self) {
        // Get the name of the function
        let global = self.parse_variable("Expect function name.");

        // Define it, aka mark it as initialized
        if self.is_scoped() {
            self.mark_initialized();
        }

        self.function(FunctionType::Function);

        // Define it as a global, will also try to define the local, but that has already been done
        self.define_variable(global);
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

        // Pop then
        self.emit_byte(opcode::OP_POP);

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

    fn for_statement(&mut self) {
        self.begin_scope();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.");
        if self.match_token(TokenKind::Semicolon) {
            // No initializer, so we just jump to the loop condition
            //let loop_start = self.current_chunk.code.len();
            //self.emit_jump(opcode::OP_JUMP);
            //self.loop_stack.push(loop_start);
        } else if self.match_token(TokenKind::Var) {
            self.var_declaration();
        } else {
            // Initialize is an expression
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().code.len();
        let mut loop_end = None;
        if !self.match_token(TokenKind::Semicolon) {
            // Compile condition
            self.expression();
            self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.");

            // Jump over loop body if condition is false
            loop_end = Some(self.emit_jump(opcode::OP_JUMP_IF_FALSE));
            self.emit_byte(opcode::OP_POP);
        }
        if !self.match_token(TokenKind::RightParen) {
            // Jump to body
            let body_jump = self.emit_jump(opcode::OP_JUMP);
            let increment_start = self.current_chunk().code.len();
            // Compile the increment expression
            self.expression();
            self.emit_byte(opcode::OP_POP);

            // Pop the value of the increment expression
            //self.emit_byte(opcode::OP_POP);

            self.consume(TokenKind::RightParen, "Expect ')' after for clauses.");

            // Jump back to start of loop
            self.emit_jump_back(loop_start);

            loop_start = increment_start;

            // Patch the jump to the start of the body
            self.patch_jump(body_jump);
        }
        self.statement();

        self.emit_jump_back(loop_start);

        match loop_end {
            Some(loop_end) => {
                self.patch_jump(loop_end);
                // Pop the condition value from stack
                self.emit_byte(opcode::OP_POP);
            }
            None => {}
        }

        self.end_scope();
    }
    fn while_statement(&mut self) {
        // Start address of loop
        let loop_start = self.current_chunk().code.len();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.");
        // Compile the condition expression
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let jump_to_end = self.emit_jump(opcode::OP_JUMP_IF_FALSE);

        // Pop the condition value from stack
        self.emit_byte(opcode::OP_POP);

        // Compile the body statement
        self.statement();

        self.emit_jump_back(loop_start);

        self.patch_jump(jump_to_end);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_byte(opcode::OP_PRINT);
    }

    fn return_statement(&mut self) {
        if self.function_type == FunctionType::Script {
            self.error("Cannot return from top-level code.");
        }
        if self.match_token(TokenKind::Semicolon) {
            // Just return nil
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenKind::Semicolon, "Expect ';' after return value.");
            self.emit_byte(opcode::OP_RETURN);
        }
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
        } else if self.match_token(TokenKind::Return) {
            self.return_statement();
        } else if self.match_token(TokenKind::For) {
            self.for_statement();
        } else if self.match_token(TokenKind::While) {
            self.while_statement();
        } else if self.match_token(TokenKind::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn declaration(&mut self) {
        if self.match_token(TokenKind::Fun) {
            self.function_declaration();
        } else if self.match_token(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        // Synchronize after the declaration if in panic mode
        if self.parser.panic_mode {
            self.synchronize();
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
            TokenKind::Percent => self.emit_byte(opcode::OP_MODULO),
            TokenKind::Plus => self.emit_byte(opcode::OP_ADD),
            TokenKind::Minus => self.emit_byte(opcode::OP_SUBTRACT),
            TokenKind::Star => self.emit_byte(opcode::OP_MULTIPLY),
            TokenKind::Slash => self.emit_byte(opcode::OP_DIVIDE),
            _ => (),
        }
    }

    fn call(&mut self) {
        let argument_count = self.argument_list();
        self.emit_bytes(opcode::OP_CALL, argument_count);
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
            TokenKind::Percent
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::BangEqual
            | TokenKind::EqualEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => self.binary(),
            TokenKind::And => self.and(),
            TokenKind::Or => self.or(),
            TokenKind::LeftParen => self.call(),
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
        let lexeme = self.lexer.get_lexeme(&token).to_string();

        self.make_constant(Value::String(Rc::from(lexeme)))
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

    fn mark_initialized(&mut self) {
        if !self.is_scoped() {
            return;
        }
        self.locals.define();
    }

    fn define_variable(&mut self, global: u8) {
        if self.is_scoped() {
            // We are in a scope, so define the local so it is ready for use
            self.mark_initialized();
            return;
        }
        self.emit_bytes(opcode::OP_DEFINE_GLOBAL, global);
    }

    fn argument_list(&mut self) -> u8 {
        let mut argument_count = 0;
        if !self.check(TokenKind::RightParen) {
            // Continue parsing argument expressions until we see no more commas
            loop {
                self.expression();
                if argument_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                argument_count += 1;
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expect ')' after arguments.");

        argument_count
    }

    /// Compiles an 'and' statement
    fn and(&mut self) {
        // Short circuit the jump if the left operand is falsey
        let end_jump = self.emit_jump(opcode::OP_JUMP_IF_FALSE);

        // Pop the result of the expression
        self.emit_byte(opcode::OP_POP);

        // Parse the right operand
        self.parse_expression(Precedence::And);

        self.patch_jump(end_jump);
    }

    /// Compiles an 'or' statement
    fn or(&mut self) {
        // Jump to next statement if the left operand is falsey
        let else_jump = self.emit_jump(opcode::OP_JUMP_IF_FALSE);

        // Short circuit the 'or' expression if the left operand is truthy
        let end_jump = self.emit_jump(opcode::OP_JUMP);

        self.patch_jump(else_jump);

        // Pop the result of the expression
        self.emit_byte(opcode::OP_POP);

        // Parse the right operand
        self.parse_expression(Precedence::Or);

        self.patch_jump(end_jump);
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
            TokenKind::Slash | TokenKind::Star | TokenKind::Percent => Precedence::Factor,
            TokenKind::BangEqual | TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => Precedence::Comparison,
            TokenKind::And => Precedence::And,
            TokenKind::Or => Precedence::Or,
            TokenKind::LeftParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}
