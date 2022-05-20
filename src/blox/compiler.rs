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
    locals: Locals,                                     // All locals
    current_function: Function,                         // Active function being built
    function_start_states: Vec<(Lexer, Parser, usize)>, // Lexer and parser state for all function declaration starts as well as the function constant index
    function_type: FunctionType,
    output: bool,

    // VERY HACKY: If true the compiler will commit the changes to the chunk
    // In order to keep functions in the same chunk and be able to declare functions anywhere I have to defer function compilation to the end of the script
    // But since this is a single pass compiler I can't do this in the parser when the function declaration is first encountered,
    //  and I need to use this commit flag to skip over function declarations and bodies and not commit them
    // If I had more time I would have loved to convert this to a proper AST and use a visitor pattern to handle this
    commit: bool,
}

impl Compiler {
    // Create a new compiler
    pub fn new() -> Self {
        let mut compiler = Self {
            parser: Parser::new(),
            lexer: Lexer::new(),
            locals: Locals::new(),
            current_function: Function::new(),
            function_start_states: Vec::new(),
            function_type: FunctionType::Script,
            output: false,
            commit: true,
        };

        compiler.locals.declare(String::from("")); // Reserve slot 0 for the vm
        compiler
    }

    // Compiles all functions in the source
    fn compile_functions(&mut self, chunk: &mut Chunk) {
        // Iterate all the function start states (a snapshot taken at every function declaration)
        while !self.function_start_states.is_empty() {
            // Pop the last function start state
            let function_start = self.function_start_states.pop().unwrap();

            // Save the start address of this function
            let start_address = chunk.code.len();

            // Compile the function using the stored state
            self.compile_function(chunk, (function_start.0, function_start.1));

            // Look up the function constant index
            let function_constant_index = function_start.2;
            // Get the actual function through the constant index
            let function = chunk.get_constant(function_constant_index as usize);

            // The function constant was emitted on the original compiler pass, but we didn't know where its start address would be
            // so replace the constant now that we know its actual start address
            let mut new_function = Function::new();
            if let Value::Function(f) = function {
                // My rust skills are really showing here, I made an early choice to make it Rc
                // so I can't just mutate it, I need to replace it

                // A poor man's Rc clone :)
                new_function.set_arity(f.arity());
                new_function.set_name(f.name().to_string());
                new_function.set_start_address(start_address);
            }

            // Patch it with the new function data (only really new start address)
            chunk.patch_constant(
                function_constant_index as usize,
                Value::Function(Rc::from(new_function)),
            );
        }
    }

    // Compile the given source code
    pub fn compile(
        &mut self,
        source: String,    // The source code to compile
        chunk: &mut Chunk, // The chunk to compile into
        output: bool,      // If true the compiler will output the compiled code
    ) -> Option<Rc<Function>> {
        // Set output flag
        self.output = output;

        // Set the sourcecode for lexer
        self.lexer.set_source(source);

        // Start of current function, used to later set the address for the function object
        // this will change as the chunk grows through the REPL
        let start_address = chunk.code.len();

        // Reset error flags
        self.parser.had_error = false;
        self.parser.panic_mode = false;

        // Consume the first token.
        self.advance();

        // Parse declarations until EOF
        while !self.match_token(TokenKind::Eof) {
            self.declaration(chunk);
        }

        // Get the compiled function
        let function = self.end_compiler(chunk, start_address);

        // Compile functions found in the script and append them to the end of the chunk
        self.compile_functions(chunk);

        if output {
            // Print a newline after final disassembly output
            println!();
        }

        if self.parser.had_error {
            None
        } else {
            Some(function)
        }
    }

    // Outputs error message at the previous token and sets the had_error flag
    fn error(&mut self, message: &str) {
        self.error_at(self.parser.previous, self.parser.previous.line, message);
    }

    // Outputs error message at the current token and sets the had_error flag
    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.parser.current, self.parser.current.line, message);
    }

    // Outputs error message at the given token and sets the had_error flag
    fn error_at(&mut self, token: Token, line: usize, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        let lexeme = self.lexer.get_lexeme(&token);
        println!("[line {}] Error: at '{}' {}", line, lexeme, message);
        self.parser.had_error = true;
    }

    // Advances the lexer and parser by one token
    fn advance(&mut self) {
        self.parser.previous = self.parser.current;
        loop {
            match self.lexer.scan_token() {
                Ok(token) => {
                    self.parser.current = token;
                    break;
                }
                Err(err) => {
                    self.error_at(self.parser.current, err.line, err.message);
                }
            }
        }
    }
    // Checks if the current token matches the given kind and consumes it
    // If the token doesn't match it will print an error
    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.parser.current.kind == kind {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    // Checks if the current token matches the given kind
    fn check(&mut self, kind: TokenKind) -> bool {
        self.parser.current.kind == kind
    }

    // Checks if the current token matches the given kind
    // If it doesn't it will return false, if it does it will consume the token and return true
    fn match_token(&mut self, kind: TokenKind) -> bool {
        if !self.check(kind) {
            return false;
        }
        self.advance();
        true
    }

    // Writes a single byte into the chunk
    fn emit_byte(&mut self, chunk: &mut Chunk, byte: u8) {
        if !self.commit {
            return;
        }

        let line_num = self.parser.previous.line;
        chunk.write_byte(byte, line_num);
    }

    // Writes two bytes into the chunk
    fn emit_bytes(&mut self, chunk: &mut Chunk, byte1: u8, byte2: u8) {
        self.emit_byte(chunk, byte1);
        self.emit_byte(chunk, byte2);
    }

    // Writes a JUMP_BACK instruction into the chunk
    fn emit_jump_back(&mut self, chunk: &mut Chunk, to: usize) {
        self.emit_byte(chunk, opcode::OP_JUMP_BACK);

        let offset = chunk.code.len() - to + 2;

        if offset > u16::MAX as usize {
            self.error("Jump exceeds 16-bit maximum.");
        }

        // Encode offset into the 16-bit jump instruction
        self.emit_byte(chunk, (offset >> 8) as u8);
        self.emit_byte(chunk, offset as u8);
    }

    // Writes a given jump instruction into the chunk
    fn emit_jump(&mut self, chunk: &mut Chunk, instruction: u8) -> usize {
        if !self.commit {
            return 0;
        }
        self.emit_byte(chunk, instruction);

        // Encode offset into the 16-bit jump instruction
        // We currently emit a dummy address to be patched once we know how far to jump
        self.emit_bytes(chunk, 0xff, 0xff);

        // Return the offset of the jump instruction
        chunk.code.len() - 2
    }

    // Writes nil and a return instruction into the chunk
    fn emit_return(&mut self, chunk: &mut Chunk) {
        if !self.commit {
            return;
        }
        // Default return value is nil
        self.emit_byte(chunk, opcode::OP_NIL);
        self.emit_byte(chunk, opcode::OP_RETURN);
    }

    // Adds a constant to the chunk and returns its index
    fn make_constant(&mut self, chunk: &mut Chunk, value: Value) -> u8 {
        if !self.commit {
            return 0;
        }
        let constant_index = chunk.add_constant(value);

        if constant_index > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
        }
        constant_index as u8
    }

    // Adds a constant to the chunk and writes it to the chunk code
    fn emit_constant(&mut self, chunk: &mut Chunk, constant: Value) -> u8 {
        if !self.commit {
            return 0;
        }
        let constant_index = self.make_constant(chunk, constant);
        self.emit_bytes(chunk, opcode::OP_CONSTANT, constant_index);
        constant_index
    }

    // Patches a jump instruction address to the current code position
    fn patch_jump(&mut self, chunk: &mut Chunk, offset: usize) {
        if !self.commit {
            return;
        }
        let jump_offset = chunk.code.len() - offset - 2;

        if jump_offset > u16::MAX as usize {
            self.error("Jump exceeds 16-bit maximum.");
        }

        // Encode offset into the 16-bit jump instruction
        chunk.code[offset] = (jump_offset >> 8) as u8;
        chunk.code[offset + 1] = jump_offset as u8;
    }

    // Returns compiled function, the compiler only operates on functions
    fn end_compiler(&mut self, chunk: &mut Chunk, start_address: usize) -> Rc<Function> {
        self.emit_return(chunk);

        if !self.parser.had_error {
            // Get the chunk name from the function name
            let chunk_name = if self.current_function.name().is_empty() {
                "<script>"
            } else {
                self.current_function.name()
            };

            // Disassemble the chunk if we have code to disassemble
            if self.output && chunk.code.len() - start_address > 0 {
                chunk.disassemble_chunk_from(chunk_name, start_address);
            }

            // Update the start address of the function (this is mainly used for the 'main' function aka global script function)
            self.current_function.set_start_address(start_address);
        }

        Rc::from(self.current_function.clone())
    }

    // Begins a scope
    fn begin_scope(&mut self) {
        self.locals.begin_scope();
    }

    // Ends a scope. Pops all the locals used in the scope
    fn end_scope(&mut self, chunk: &mut Chunk) {
        for _ in 0..self.locals.end_scope() {
            self.emit_byte(chunk, opcode::OP_POP);
        }
    }

    // Check if we are in a scope
    fn is_scoped(&self) -> bool {
        self.locals.scope_depth() > 0
    }

    // Parses and emits a string constant
    fn string(&mut self, chunk: &mut Chunk) {
        let token = &self.parser.previous;
        let lexeme = self.lexer.get_lexeme(token).to_string();
        // Remove the quotes

        self.emit_constant(chunk, Value::String(Rc::from(&lexeme[1..lexeme.len() - 1])));
    }

    // Resolves a local variable in the current scope
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

    // Parses and compiles a variable declaration or assignment
    fn named_variable(&mut self, chunk: &mut Chunk, name: Token, can_assign: bool) {
        // See if we can find a local variable with this name
        let (var_index, get_op, set_op) = match self.resolve_local(name) {
            Some(local_index) => (
                local_index as u8,
                opcode::OP_GET_LOCAL,
                opcode::OP_SET_LOCAL,
            ),
            // Assume it's global
            None => (
                self.identifier_constant(chunk, name),
                opcode::OP_GET_GLOBAL,
                opcode::OP_SET_GLOBAL,
            ),
        };

        if can_assign && self.match_token(TokenKind::Equal) {
            // If we match with an equals sign, we know it's a variable assignment
            self.expression(chunk);
            self.emit_bytes(chunk, set_op, var_index);
        } else {
            // If not it's a variable access
            self.emit_bytes(chunk, get_op, var_index);
        }
    }

    // Parses and compiles a variable declaration
    fn variable(&mut self, chunk: &mut Chunk, can_assign: bool) {
        self.named_variable(chunk, self.parser.previous, can_assign);
    }

    // Parses and compiles a number constant
    fn number(&mut self, chunk: &mut Chunk) {
        let token = &self.parser.previous;
        let lexeme = self.lexer.get_lexeme(token);
        let value = lexeme.parse::<Value>().expect("Failed to parse lexeme");
        self.emit_constant(chunk, value);
    }

    // Parses and compiles a grouping expression
    fn grouping(&mut self, chunk: &mut Chunk) {
        self.expression(chunk);
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }

    // Parses and compiles an expression
    fn expression(&mut self, chunk: &mut Chunk) {
        self.parse_expression(chunk, Precedence::Assignment);
    }

    // Parses and compiles a block (scope)
    fn block(&mut self, chunk: &mut Chunk) {
        // Iterate all declarations in the block and compile them
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            self.declaration(chunk);
        }
        self.consume(TokenKind::RightBrace, "Expect '}' after block.");
    }

    // Parses and compiles a function
    fn function(&mut self, chunk: &mut Chunk, function_type: FunctionType) -> Rc<Function> {
        let old_function_type = self.function_type;
        let old_function = self.current_function.clone();
        self.function_type = function_type;
        let start_addr = chunk.code.len();
        let old_locals = self.locals.clone();

        self.locals = Locals::new();
        self.locals.declare(String::from("")); // Reserve slot 0 for the vm

        let function_name = self.lexer.get_lexeme(&self.parser.previous).to_string();
        self.current_function.set_name(function_name);

        self.begin_scope();

        self.consume(TokenKind::LeftParen, "Expect '(' after function name.");

        // If we have parameters, add them
        while !self.check(TokenKind::RightParen) {
            self.current_function.inc_arity();
            if self.current_function.arity() > 255 {
                self.error_at_current("Can't have more than 255 parameters");
            }
            let param_index = self.parse_variable(chunk, "Expect parameter name");
            self.define_variable(chunk, param_index);
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
        self.block(chunk);

        self.end_scope(chunk);

        let function = self.end_compiler(chunk, start_addr);

        self.function_type = old_function_type;
        self.current_function = old_function;
        self.locals = old_locals;

        function
    }

    // Parses and compiles a function declaration
    fn function_declaration(&mut self, chunk: &mut Chunk) {
        // Get the name of the function
        let global = self.parse_variable(chunk, "Expect function name.");

        // Defer function compilation, so store lexer state
        let lexer_state = self.lexer.clone();
        let parser_state = self.parser.clone();

        // We don't want to commit the function code to the chunk yet, we just want to parse past it
        self.commit = false;
        let function = self.compile_function(chunk, (self.lexer.clone(), self.parser.clone()));
        self.commit = true;

        // Emit the function constant immediately, don't defer this
        // We need to be able to access it when executing
        let constant = self.make_constant(chunk, Value::Function(function));
        self.emit_bytes(chunk, opcode::OP_CONSTANT, constant);

        // Store the lexer state, parser state and function constant so we can actually compile it at the end of compilation
        self.function_start_states
            .push((lexer_state, parser_state, constant as usize));

        // Define global variable for the function
        self.define_variable(chunk, global);
    }

    // This does the actual function compilation at the end of the compilation process
    fn compile_function(&mut self, chunk: &mut Chunk, state: (Lexer, Parser)) -> Rc<Function> {
        // Set state to a state where the function name has been parsed and the global has been defined
        self.lexer = state.0;
        self.parser = state.1;

        // Define it, aka mark it as initialized
        if self.is_scoped() {
            self.mark_initialized();
        }

        self.function(chunk, FunctionType::Function)

        // TODO: Restore lexer and parser state? Realistically it won't be used again since we are in a state of only compiling functions
    }

    // Parses and compiles a variable declaration
    fn var_declaration(&mut self, chunk: &mut Chunk) {
        let global = self.parse_variable(chunk, "Expect variable name.");
        if self.match_token(TokenKind::Equal) {
            // Consume the expression
            self.expression(chunk);
        } else {
            // If no explicit assignment is made, use the default value nil
            self.emit_byte(chunk, opcode::OP_NIL);
        }
        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(chunk, global);
    }

    // Parses and compiles an expression statement
    fn expression_statement(&mut self, chunk: &mut Chunk) {
        self.expression(chunk);
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_byte(chunk, opcode::OP_POP);
    }

    // Parses and compiles an if statement
    fn if_statement(&mut self, chunk: &mut Chunk) {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        // Compile condition expression
        self.expression(chunk);
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(chunk, opcode::OP_JUMP_IF_FALSE);

        // Pop then
        self.emit_byte(chunk, opcode::OP_POP);

        // Compile statement for if branch
        self.statement(chunk);

        // This is to jump over potential else branch after finishing execution of the then statement
        let else_jump = self.emit_jump(chunk, opcode::OP_JUMP);

        // Patch the jump to the end of the if branch (that we jump to if condition is false)
        // we now know how long the if branch is
        self.patch_jump(chunk, then_jump);

        // Clean up the statement value from stack
        self.emit_byte(chunk, opcode::OP_POP);

        if self.match_token(TokenKind::Else) {
            // Compile statement for else branch
            self.statement(chunk);
        }
        // Patch the jump to the end of the else statement
        self.patch_jump(chunk, else_jump);
    }

    // Parses and compiles a for loop statement
    fn for_statement(&mut self, chunk: &mut Chunk) {
        // Start loop scope
        self.begin_scope();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.");
        if self.match_token(TokenKind::Semicolon) {
            // No initializer
        } else if self.match_token(TokenKind::Var) {
            self.var_declaration(chunk);
        } else {
            // Initialize is an expression
            self.expression_statement(chunk);
        }

        let mut loop_start = chunk.code.len();
        let mut loop_end = None;
        if !self.match_token(TokenKind::Semicolon) {
            // Compile condition
            self.expression(chunk);
            self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.");

            // Jump over loop body if condition is false
            loop_end = Some(self.emit_jump(chunk, opcode::OP_JUMP_IF_FALSE));
            self.emit_byte(chunk, opcode::OP_POP);
        }
        if !self.match_token(TokenKind::RightParen) {
            // Jump to body
            let body_jump = self.emit_jump(chunk, opcode::OP_JUMP);
            let increment_start = chunk.code.len();

            // Compile the increment expression
            self.expression(chunk);

            // Pop the value of the increment expression
            self.emit_byte(chunk, opcode::OP_POP);

            self.consume(TokenKind::RightParen, "Expect ')' after for clauses.");

            // Jump back to start of loop
            self.emit_jump_back(chunk, loop_start);

            loop_start = increment_start;

            // Patch the jump to the start of the body
            self.patch_jump(chunk, body_jump);
        }
        // Compile the loop body
        self.statement(chunk);

        // Jump back to top
        self.emit_jump_back(chunk, loop_start);

        if let Some(loop_end) = loop_end {
            // Patch the jump to the end of the loop
            self.patch_jump(chunk, loop_end);
            // Pop the condition value from stack
            self.emit_byte(chunk, opcode::OP_POP);
        }

        // End loop scope
        self.end_scope(chunk);
    }
    fn while_statement(&mut self, chunk: &mut Chunk) {
        // Start address of loop
        let loop_start = chunk.code.len();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.");

        // Compile the condition expression
        self.expression(chunk);

        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let jump_to_end = self.emit_jump(chunk, opcode::OP_JUMP_IF_FALSE);

        // Pop the condition value from stack
        self.emit_byte(chunk, opcode::OP_POP);

        // Compile the body statement
        self.statement(chunk);

        // Jump back to start of loop
        self.emit_jump_back(chunk, loop_start);

        // Patch the jump to the end of the loop now that we know how long the loop body is
        self.patch_jump(chunk, jump_to_end);
    }

    // Parses and compiles a print statement
    fn print_statement(&mut self, chunk: &mut Chunk) {
        self.expression(chunk);
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_byte(chunk, opcode::OP_PRINT);
    }

    // Parses and compiles a return statement
    fn return_statement(&mut self, chunk: &mut Chunk) {
        if self.function_type == FunctionType::Script {
            self.error("Cannot return from top-level code.");
        }
        if self.match_token(TokenKind::Semicolon) {
            // Just return nil
            self.emit_return(chunk);
        } else {
            self.expression(chunk);
            self.consume(TokenKind::Semicolon, "Expect ';' after return value.");
            self.emit_byte(chunk, opcode::OP_RETURN);
        }
    }

    // Synchronizes the lexer and parser to a valid state, after the erronous declaration
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

    // Parses and compiles a statement
    fn statement(&mut self, chunk: &mut Chunk) {
        if self.match_token(TokenKind::Print) {
            self.print_statement(chunk);
        } else if self.match_token(TokenKind::If) {
            self.if_statement(chunk);
        } else if self.match_token(TokenKind::Return) {
            self.return_statement(chunk);
        } else if self.match_token(TokenKind::For) {
            self.for_statement(chunk);
        } else if self.match_token(TokenKind::While) {
            self.while_statement(chunk);
        } else if self.match_token(TokenKind::LeftBrace) {
            self.begin_scope();
            self.block(chunk);
            self.end_scope(chunk);
        } else {
            self.expression_statement(chunk);
        }
    }

    // Parses and compiles a declaration
    fn declaration(&mut self, chunk: &mut Chunk) {
        if self.match_token(TokenKind::Fun) {
            self.function_declaration(chunk);
        } else if self.match_token(TokenKind::Var) {
            self.var_declaration(chunk);
        } else {
            self.statement(chunk);
        }

        // Synchronize after the declaration if in panic mode
        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    // Parses and compiles a unary expression
    fn unary(&mut self, chunk: &mut Chunk) {
        let operator_kind = self.parser.previous.kind;

        // Compile the operand
        self.parse_expression(chunk, Precedence::Unary);

        // Emit the operator instruction
        match operator_kind {
            TokenKind::Bang => self.emit_byte(chunk, opcode::OP_NOT),
            TokenKind::Minus => self.emit_byte(chunk, opcode::OP_NEGATE),
            _ => (),
        }
    }

    // Parses and compiles a binary expression
    fn binary(&mut self, chunk: &mut Chunk) {
        let operator_kind = self.parser.previous.kind;

        let precedence = Precedence::from(operator_kind);

        self.parse_expression(chunk, precedence);

        // TODO: make operations such as != >= and <= a single instruction
        match operator_kind {
            TokenKind::BangEqual => self.emit_bytes(chunk, opcode::OP_EQUAL, opcode::OP_NOT),
            TokenKind::EqualEqual => self.emit_byte(chunk, opcode::OP_EQUAL),
            TokenKind::Greater => self.emit_byte(chunk, opcode::OP_GREATER),
            TokenKind::GreaterEqual => self.emit_bytes(chunk, opcode::OP_LESS, opcode::OP_NOT),
            TokenKind::Less => self.emit_byte(chunk, opcode::OP_LESS),
            TokenKind::LessEqual => self.emit_bytes(chunk, opcode::OP_GREATER, opcode::OP_NOT),
            TokenKind::Percent => self.emit_byte(chunk, opcode::OP_MODULO),
            TokenKind::Plus => self.emit_byte(chunk, opcode::OP_ADD),
            TokenKind::Minus => self.emit_byte(chunk, opcode::OP_SUBTRACT),
            TokenKind::Star => self.emit_byte(chunk, opcode::OP_MULTIPLY),
            TokenKind::Slash => self.emit_byte(chunk, opcode::OP_DIVIDE),
            _ => (),
        }
    }

    // Parses and compiles a call instruction
    fn call(&mut self, chunk: &mut Chunk) {
        // Emit the return address constant
        let argument_count = self.argument_list(chunk);
        self.emit_bytes(chunk, opcode::OP_CALL, argument_count);
    }

    // Compiles a literal
    fn literal(&mut self, chunk: &mut Chunk) {
        match self.parser.previous.kind {
            TokenKind::False => self.emit_byte(chunk, opcode::OP_FALSE),
            TokenKind::True => self.emit_byte(chunk, opcode::OP_TRUE),
            TokenKind::Nil => self.emit_byte(chunk, opcode::OP_NIL),
            _ => (),
        }
    }

    // Parses and compiles a prefix expression
    fn parse_prefix(&mut self, chunk: &mut Chunk, can_assign: bool) {
        match self.parser.previous.kind {
            TokenKind::LeftParen => self.grouping(chunk),
            TokenKind::Minus | TokenKind::Bang => self.unary(chunk),
            TokenKind::Number => self.number(chunk),
            TokenKind::String => self.string(chunk),
            TokenKind::True | TokenKind::False | TokenKind::Nil => self.literal(chunk),
            TokenKind::Identifier => self.variable(chunk, can_assign),
            _ => {
                self.error("Expect prefix expression.");
            }
        }
    }

    // Parses and compiles an infix expression
    fn parse_infix(&mut self, chunk: &mut Chunk) {
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
            | TokenKind::LessEqual => self.binary(chunk),
            TokenKind::And => self.and(chunk),
            TokenKind::Or => self.or(chunk),
            TokenKind::LeftParen => self.call(chunk),
            _ => {
                self.error("Expect infix expression.");
            }
        }
    }

    // Parses an compiles a full expression statement (prefix and infix)
    fn parse_expression(&mut self, chunk: &mut Chunk, precedence: Precedence) {
        self.advance();

        let can_assign = precedence <= Precedence::Assignment;
        self.parse_prefix(chunk, can_assign);

        while !self.parser.is_at_end() {
            let next_precedence = Precedence::from(self.parser.current.kind);
            if precedence > next_precedence {
                break;
            }
            self.advance();
            self.parse_infix(chunk);
        }

        if can_assign && self.match_token(TokenKind::Equal) {
            self.error("Invalid assignment target.");
            // NOTE: I am not sure if this will be valid in all contexts
            self.advance();
        }
    }

    // Adds an identifier constant to the chunk
    fn identifier_constant(&mut self, chunk: &mut Chunk, token: Token) -> u8 {
        let lexeme = self.lexer.get_lexeme(&token).to_string();

        self.make_constant(chunk, Value::String(Rc::from(lexeme)))
    }

    // Adds a local variable to scope
    fn add_local(&mut self, name: String) {
        if self.locals.is_full() {
            self.error("Too many local variables in function.");
            return;
        }
        self.locals.declare(name);
    }

    // Gets variable name and adds it to the scope
    fn declare_variable(&mut self) {
        // Global, ignore
        if !self.is_scoped() {
            return;
        }

        let name = self.lexer.get_lexeme(&self.parser.previous).to_string();

        if self.locals.contains(&name) {
            self.error("Variable with this name already declared in this scope.");
        }

        self.add_local(name);
    }

    // Parses a variable expression and adds it to the scope and constants
    fn parse_variable(&mut self, chunk: &mut Chunk, message: &str) -> u8 {
        // Consume the identifier

        self.consume(TokenKind::Identifier, message);

        self.declare_variable();

        if self.is_scoped() {
            return 0;
        }

        // Make identifier constant
        self.identifier_constant(chunk, self.parser.previous)
    }

    // Marks a local as initialized
    fn mark_initialized(&mut self) {
        if !self.is_scoped() {
            return;
        }
        self.locals.define();
    }

    // Defines a variable
    fn define_variable(&mut self, chunk: &mut Chunk, global: u8) {
        if self.is_scoped() {
            // We are in a scope, so define the local so it is ready for use
            self.mark_initialized();
            return;
        }
        self.emit_bytes(chunk, opcode::OP_DEFINE_GLOBAL, global);
    }

    // Parses an argument list and returns the number of arguments
    fn argument_list(&mut self, chunk: &mut Chunk) -> u8 {
        let mut argument_count = 0;

        if !self.check(TokenKind::RightParen) {
            // Continue parsing argument expressions until we see no more commas
            loop {
                self.expression(chunk);
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

    // Compiles an 'and' statement
    fn and(&mut self, chunk: &mut Chunk) {
        // Short circuit the jump if the left operand is falsey
        let end_jump = self.emit_jump(chunk, opcode::OP_JUMP_IF_FALSE);

        // Pop the result of the expression
        self.emit_byte(chunk, opcode::OP_POP);

        // Parse the right operand
        self.parse_expression(chunk, Precedence::And);

        self.patch_jump(chunk, end_jump);
    }

    // Compiles an 'or' statement
    fn or(&mut self, chunk: &mut Chunk) {
        // Jump to next statement if the left operand is falsey
        let else_jump = self.emit_jump(chunk, opcode::OP_JUMP_IF_FALSE);

        // Short circuit the 'or' expression if the left operand is truthy
        let end_jump = self.emit_jump(chunk, opcode::OP_JUMP);

        self.patch_jump(chunk, else_jump);

        // Pop the result of the expression
        self.emit_byte(chunk, opcode::OP_POP);

        // Parse the right operand
        self.parse_expression(chunk, Precedence::Or);

        self.patch_jump(chunk, end_jump);
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
}

// Retrieves the precedence of the current token
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
