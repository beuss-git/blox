use std::collections::HashMap;

use super::{chunk::Chunk, compiler::Compiler, opcode, value::Value};

const DEBUG_TRACE_EXECUTION: bool = false;
const DEBUG_DISASSEMBLY: bool = true;
pub struct VM {
    chunk: Chunk,
    pc: usize,
    stack: Vec<Value>,
    last_printed: Option<Value>,
    globals: HashMap<String, Value>,
}

macro_rules! binary_op {
        ($self:ident, $value_type:ident, $op:tt) => {
            match ($self.pop(), $self.pop()) {
                (Value::Number(b), Value::Number(a)) => $self.push(Value::$value_type(a $op b)),
                _ => {
                    $self.runtime_error("Operands must be numbers.");
                    return InterpretResult::RuntimeError;
                }
            }
        };
}
impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            pc: 0,
            stack: Vec::new(),
            last_printed: None,
            globals: HashMap::new(),
        }
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(source, &mut self.chunk);
        if !compiler.compile() {
            return InterpretResult::CompileError;
        }

        if DEBUG_DISASSEMBLY {
            compiler.disassemble();
        }

        self.pc = 0;

        self.run()
    }
    fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                self.print_stacktrace();
                self.chunk.disassemble_instruction(self.pc);
                //.disassemble_instruction(self.pc - self.chunk.code.len());
            }
            // TODO: make operations such as != >= and <= a single instruction
            // Decode the instruction
            match self.read_byte() {
                opcode::OP_GREATER => binary_op!(self, Boolean, >),
                opcode::OP_LESS => binary_op!(self, Boolean, <),
                //opcode::OP_ADD => binary_op!(self, Number, +),
                opcode::OP_MODULO => binary_op!(self, Number, %),
                opcode::OP_ADD => match (self.pop(), self.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
                    (Value::String(b), Value::String(a)) => self.push(Value::String(a + &b)),
                    (Value::Number(b), Value::String(a)) => {
                        self.push(Value::String(a + &b.to_string()))
                    }
                    (Value::Boolean(b), Value::String(a)) => {
                        self.push(Value::String(a + &b.to_string()))
                    }
                    (Value::Nil, Value::String(b)) => self.push(Value::String(b + "nil")),
                    _ => {
                        self.runtime_error("Operands must be numbers.");
                        return InterpretResult::RuntimeError;
                    }
                },
                opcode::OP_SUBTRACT => binary_op!(self, Number, -),
                opcode::OP_MULTIPLY => binary_op!(self, Number, *),
                opcode::OP_DIVIDE => binary_op!(self, Number, /),
                opcode::OP_NOT => match self.stack.pop() {
                    Some(x) => self.push(Value::Boolean(x.is_falsey())),
                    _ => {
                        self.runtime_error("Stack is empty.");
                        return InterpretResult::RuntimeError;
                    }
                },
                opcode::OP_NEGATE => match self.stack.pop() {
                    Some(Value::Number(n)) => self.stack.push(Value::Number(-n)),
                    _ => {
                        self.runtime_error("Stack is empty");
                        return InterpretResult::RuntimeError;
                    }
                },
                opcode::OP_PRINT => {
                    let value = self.pop();
                    self.last_printed = Some(value.clone());
                    println!("{}", value);
                }
                opcode::OP_JUMP_BACK => {
                    let offset = self.read_short();
                    self.pc -= offset as usize;
                }
                opcode::OP_JUMP => {
                    let offset = self.read_short();
                    self.pc += offset as usize;
                }
                opcode::OP_JUMP_IF_FALSE => {
                    let offset = self.read_short();
                    if self.peek().is_falsey() {
                        self.pc += offset as usize;
                    }
                    // Else keep on churning
                }
                opcode::OP_RETURN => {
                    return InterpretResult::Ok;
                }
                opcode::OP_CONSTANT => {
                    let constant: Value = self.read_constant();
                    self.push(constant);
                }
                opcode::OP_NIL => self.push(Value::Nil),
                opcode::OP_TRUE => self.push(Value::Boolean(true)),
                opcode::OP_FALSE => self.push(Value::Boolean(false)),
                opcode::OP_POP => {
                    if !self.stack_empty() {
                        self.pop();
                    }
                }
                opcode::OP_SET_LOCAL => {
                    let slot = self.read_byte() as usize;

                    let value = self.peek();
                    self.stack[slot] = value;
                }
                opcode::OP_GET_LOCAL => {
                    let slot = self.read_byte() as usize;
                    let value = self.stack[slot].clone();
                    self.push(value);
                }
                opcode::OP_GET_GLOBAL => {
                    let name = self.read_constant();
                    let value = self.globals.get(&name.to_string()).cloned();
                    if let Some(value) = value {
                        self.push(value);
                    } else {
                        self.runtime_error(&format!("Undefined variable '{}'.", name));
                        return InterpretResult::RuntimeError;
                    }
                }
                opcode::OP_DEFINE_GLOBAL => {
                    // TODO: Check if it is a string
                    let name = self.read_constant();
                    let value = self.pop();
                    self.globals.insert(name.to_string(), value);
                }
                opcode::OP_SET_GLOBAL => {
                    let name = self.read_constant();
                    let value = self.peek();
                    // Possible to get an iter instead of checking and then inserting?
                    // Can also just insert, check ret value and return error if it is not None, but make sure to delete value in there
                    if self.globals.contains_key(&name.to_string()) {
                        self.globals.insert(name.to_string(), value);
                    } else {
                        self.runtime_error(&format!("Undefined variable '{}'.", name));
                        return InterpretResult::RuntimeError;
                    }
                }
                opcode::OP_EQUAL => match (self.pop(), self.pop()) {
                    (a, b) => self.push(Value::Boolean(Value::is_same(a, b))),
                },
                _ => {
                    let op = self.read_byte();
                    self.runtime_error(format!("Unknown opcode: {}", op).as_str());
                    return InterpretResult::RuntimeError;
                }
            }
        }
        //InterpretResult::Ok
    }

    fn peek(&self) -> Value {
        self.stack.last().expect("Stack empty").clone()
    }

    fn read_byte(&mut self) -> u8 {
        self.pc += 1;
        self.chunk.code[self.pc - 1]
    }

    fn read_short(&mut self) -> u16 {
        self.pc += 2;
        ((self.chunk.code[self.pc - 2] as u16) << 8) | self.chunk.code[self.pc - 1] as u16
    }
    fn read_constant(&mut self) -> Value {
        let constant_index = self.read_byte();
        self.chunk.get_value(constant_index as usize)
    }
    fn print_stacktrace(&self) {
        for value in self.stack.iter() {
            print!("[ {} ]", value);
        }
        println!();
    }
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    fn stack_empty(&self) -> bool {
        self.stack.is_empty()
    }
    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack is empty")
    }

    fn runtime_error(&self, message: &str) {
        println!("[line {}] {}", self.chunk.get_line(self.pc), message);
        println!("{}", self.chunk.disassemble_instruction(self.pc));
    }

    #[allow(dead_code)]
    fn last_value(&self) -> Option<Value> {
        self.last_printed.clone()
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

#[cfg(test)]
mod tests {
    use crate::blang::{chunk::Chunk, value::Value, vm::InterpretResult};

    use super::VM;

    // TODO: Remove this when proper chunk support is implemented
    fn new_vm() -> VM {
        VM::new(Chunk::new())
    }

    fn expect_value(expr: &str, expected: Value) {
        let mut vm = new_vm();
        let res = vm.interpret(expr.to_string());
        assert_eq!(res, InterpretResult::Ok);
        assert_eq!(vm.last_value().unwrap(), expected);
    }

    fn expect_none(expr: &str) {
        let mut vm = new_vm();
        let res = vm.interpret(expr.to_string());
        assert!(vm.last_value().is_none());
    }

    fn expect_compile_result(expr: &str, expected: InterpretResult) {
        let mut vm = new_vm();
        let res = vm.interpret(expr.to_string());
        assert_eq!(res, expected);
    }

    #[test]
    fn test_arithmetic() {
        expect_value("print 1+3*4;", Value::Number(13.0));
        expect_value("print (1+3*3)/5+(4*3);", Value::Number(14.0));
    }

    #[test]
    fn test_modulo() {
        expect_value("print 5%2;", Value::Number(1.0));
        expect_value("print 5%3;", Value::Number(2.0));
    }

    #[test]
    fn test_addition() {
        expect_value("print 1+3;", Value::Number(4.0));
        expect_value("print 4+3;", Value::Number(7.0));
    }
    #[test]
    fn test_string_concatenation() {
        expect_value(
            r#"print "Hello" + " " + "World!";"#,
            Value::String("Hello World!".to_string()),
        );
        expect_value(
            r#"print "Hel" + "lo" + ", " + "Wo" + "rld!";"#,
            Value::String("Hello, World!".to_string()),
        );
        expect_value(
            r#"print "one" + "two";"#,
            Value::String("onetwo".to_string()),
        );
        expect_value(r#"print "one" + 2;"#, Value::String("one2".to_string()));
        expect_value(r#"print "one" + 2.1;"#, Value::String("one2.1".to_string()));
        expect_value(
            r#"print "one" + true;"#,
            Value::String("onetrue".to_string()),
        );
        expect_value(
            r#"print "one" + false;"#,
            Value::String("onefalse".to_string()),
        );
        expect_value(r#"print "one" + nil;"#, Value::String("onenil".to_string()));
    }
    #[test]
    fn test_subtraction() {
        expect_value("print 1-3;", Value::Number(-2.0));
        expect_value("print 6-2;", Value::Number(4.0));
    }

    #[test]
    fn test_multiplication() {
        expect_value("print 2*10;", Value::Number(20.0));
        expect_value("print 3*2*1;", Value::Number(6.0));
        expect_value("print 1*2*3;", Value::Number(6.0));
    }

    #[test]
    fn test_division() {
        expect_value("print 2/2;", Value::Number(1.0));
        expect_value("print 4/2;", Value::Number(2.0));
        expect_value("print 2/4;", Value::Number(0.5));
        expect_value("print 3/2/1;", Value::Number(1.5));
    }

    #[test]
    fn test_not() {
        expect_value("print !true;", Value::Boolean(false));
        expect_value("print !false;", Value::Boolean(true));
    }

    #[test]
    fn test_negation() {
        expect_value("print -1;", Value::Number(-1.0));
        expect_value("print -2;", Value::Number(-2.0));
        expect_value("print -3;", Value::Number(-3.0));
        expect_value("print --3;", Value::Number(3.0));
        expect_value("print ---3;", Value::Number(-3.0));
    }

    #[test]
    fn test_nil() {
        expect_value("print nil;", Value::Nil);
    }

    #[test]
    fn test_boolean() {
        expect_value("print true;", Value::Boolean(true));
        expect_value("print false;", Value::Boolean(false));
    }

    #[test]
    fn test_comments() {
        expect_value("print 1+3*4; // comment", Value::Number(13.0));
        expect_none("// 1+3*4");
        expect_value("print 1; //+3*4", Value::Number(1.0));
        expect_value(
            r#"
            var b = 2;
            //b = 14;
            print b;
        "#,
            Value::Number(2.0),
        );
    }

    #[test]
    fn test_comparison() {
        expect_value("print !(5 - 4 > 3 * 2 == !nil);", Value::Boolean(true));
    }

    #[test]
    fn test_not_equal() {
        expect_value("print 5 != 4;", Value::Boolean(true));
        expect_value("print 5 != 5;", Value::Boolean(false));
        expect_value("print true != true;", Value::Boolean(false));
        expect_value("print false != false;", Value::Boolean(false));
        expect_value("print true != false;", Value::Boolean(true));
        expect_value("print false != true;", Value::Boolean(true));
        expect_value(r#"print "str" != "str";"#, Value::Boolean(false));
        expect_value(r#"print "str" != "st2";"#, Value::Boolean(true));
        expect_value(r#"print "str" != "st";"#, Value::Boolean(true));
    }

    #[test]
    fn test_equal() {
        expect_value("print 1 == 1;", Value::Boolean(true));
        expect_value("print 1 == 2;", Value::Boolean(false));
        expect_value("print 1 == 1.0;", Value::Boolean(true));
        expect_value("print 1.0 == 1;", Value::Boolean(true));
        expect_value("print 1.0 == 1.0;", Value::Boolean(true));
        expect_value("print 1.0 == 2.0;", Value::Boolean(false));
        expect_value("print 1.0 == 1.0;", Value::Boolean(true));
        expect_value("print 1.0 == 2.0;", Value::Boolean(false));
        expect_value("print 1.0 == 1.0;", Value::Boolean(true));
        expect_value("print 1.0 == 2.0;", Value::Boolean(false));
        expect_value("print true == true;", Value::Boolean(true));
        expect_value("print false == false;", Value::Boolean(true));
        expect_value("print true == false;", Value::Boolean(false));
        expect_value("print false == true;", Value::Boolean(false));
        expect_value(r#"print "str" == "str";"#, Value::Boolean(true));
        expect_value(r#"print "str" == "st2";"#, Value::Boolean(false));
        expect_value(r#"print "str" == "st";"#, Value::Boolean(false));
    }

    #[test]
    fn test_greater() {
        expect_value("print 5 > 4;", Value::Boolean(true));
        expect_value("print 5 > 5;", Value::Boolean(false));
        expect_value("print 5 > 6;", Value::Boolean(false));
    }

    #[test]
    fn test_greater_equal() {
        expect_value("print 5 >= 4;", Value::Boolean(true));
        expect_value("print 5 >= 5;", Value::Boolean(true));
        expect_value("print 5 >= 6;", Value::Boolean(false));
    }

    #[test]
    fn test_less() {
        expect_value("print 5 < 4;", Value::Boolean(false));
        expect_value("print 5 < 5;", Value::Boolean(false));
        expect_value("print 5 < 6;", Value::Boolean(true));
    }

    #[test]
    fn test_less_equal() {
        expect_value("print 5 <= 4;", Value::Boolean(false));
        expect_value("print 5 <= 5;", Value::Boolean(true));
        expect_value("print 5 <= 6;", Value::Boolean(true));
    }

    #[test]
    fn test_global_variable_declaration() {
        expect_value(
            r#"
        var a = 1;
        var b = a + 3;
        print b + a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            r#"
        var a = 1;
        var b = 3 + 1;
        print b + a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            r#"
        var a = 1;
        var b = 3 + 1;
        print a + b;
        "#,
            Value::Number(5.0),
        );
    }

    #[test]
    fn test_global_variable_assignment() {
        expect_value(
            r#"
        var a = 1;
        a = 2;
        print a;
        "#,
            Value::Number(2.0),
        );

        expect_value(
            r#"
        var a = 1;
        a = a + 2;
        print a;
        "#,
            Value::Number(3.0),
        );

        // Assign to invalid assignment target
        expect_compile_result(
            r#"
                a + b = c;
            "#,
            InterpretResult::CompileError,
        );

        // Assign to invalid assignment target
        expect_compile_result(
            r#"
                    var c = 3;
                    a + b = c;
                "#,
            InterpretResult::CompileError,
        );

        // Assign to invalid assignment target
        expect_compile_result(
            r#"
                    var c = 3;
                    var a = 1;
                    var b = 2;
                    a + b = c;
                "#,
            InterpretResult::CompileError,
        );
    }

    #[test]
    fn test_default_nil() {
        expect_value(
            r#"
        var a;
        print a;
        "#,
            Value::Nil,
        );
    }

    #[test]
    fn test_nil_value() {
        expect_value(
            r#"
        var a = nil;
        print a;
        "#,
            Value::Nil,
        );
    }

    #[test]
    fn test_number_value() {
        expect_value(
            r#"
        var a = 5.0;
        print a;
        "#,
            Value::Number(5.0),
        );
    }

    #[test]
    fn test_string_value() {
        expect_value(
            r#"
        var a = "hello";
        print a;
        "#,
            Value::String("hello".to_string()),
        );
    }

    #[test]
    fn test_bool_value() {
        expect_value(
            r#"
        var a = true;
        print a;
        "#,
            Value::Boolean(true),
        );
    }

    #[test]
    fn test_value_assignment() {
        // Number
        expect_value(
            r#"
        var a;
        a = 1.0;
        print a;
        "#,
            Value::Number(1.0),
        );

        // Bool
        expect_value(
            r#"
        var a;
        a = false;
        print a;
        "#,
            Value::Boolean(false),
        );

        // String
        expect_value(
            r#"
        var a;
        a = "hello";
        print a;
        "#,
            Value::String("hello".to_string()),
        );

        // Nil
        expect_value(
            r#"
        var a;
        a = nil;
        print a;
        "#,
            Value::Nil,
        );
    }
    // TODO: Scope test

    #[test]
    fn test_scope() {
        expect_value(
            r#"
            {
                var a = "outer";
                {
                    var a = 3;
                    print a;
                }
            }

        "#,
            Value::Number(3.0),
        );

        expect_value(
            r#"
            {
                var a = "outer";
                {
                    var a = 3;
                }
                print a;
            }

        "#,
            Value::String("outer".to_string()),
        );

        expect_value(
            r#"
            {
                var a = "outer";
                {
                    print a;
                }
            }

        "#,
            Value::String("outer".to_string()),
        );

        expect_value(
            r#"
            {
                var a = "outer";
                {
                    print a;
                }
                a = 3;
                print a;
            }

        "#,
            Value::Number(3.0),
        );
    }
    #[test]
    fn test_undefined_variable() {
        // Test undefined in local sope
        expect_compile_result(
            r#"
            {
                print a;
            }

        "#,
            InterpretResult::RuntimeError,
        );

        // Test gone out of scope
        expect_compile_result(
            r#"
            {
                var a = 3;
            }
            print a;

        "#,
            InterpretResult::RuntimeError,
        );

        // Test gone out of scope
        expect_compile_result(
            r#"
            {
                {
                    var a = 3;
                }
            }
            print a;

        "#,
            InterpretResult::RuntimeError,
        );

        // Test gone out of scope
        expect_compile_result(
            r#"
            {
                {
                    var a = 3;
                }
                print a;
            }

        "#,
            InterpretResult::RuntimeError,
        );

        // Test undefined in global scope
        expect_compile_result(
            r#"
            print a;
            "#,
            InterpretResult::RuntimeError,
        );
    }
    #[test]
    fn test_if() {
        expect_value(
            r#"
        if (true) {
            print "hello";
        }
        "#,
            Value::String("hello".to_string()),
        );

        expect_none(
            r#"
        if (false) {
            print "hello";
        }
        "#,
        );

        expect_value(
            r#"
        if (true) {
            print "hello";
        } else {
            print "world";
        }
        "#,
            Value::String("hello".to_string()),
        );

        expect_value(
            r#"
        if (false) {
            print "hello";
        } else {
            print "world";
        }
        "#,
            Value::String("world".to_string()),
        );

        /*vm = new_vm();
        let res = vm.interpret(
            r#"
        if (true) {
            print "hello";
        } else if (false) {
            print "world";
        } else {
            print "!";
        }
        "#
            .to_string(),
        );
        assert_eq!(res, InterpretResult::Ok);
        assert_eq!(vm.last_value().unwrap(), Value::String("hello".to_string()));*/
    }
    #[test]
    fn test_logical_operators() {
        expect_value("print true and true;", Value::Boolean(true));
        expect_value("print false and true;", Value::Boolean(false));
        expect_value("print true and false;", Value::Boolean(false));
        expect_value("print false and false;", Value::Boolean(false));

        expect_value("print true or true;", Value::Boolean(true));
        expect_value("print false or true;", Value::Boolean(true));
        expect_value("print true or false;", Value::Boolean(true));
        expect_value("print false or false;", Value::Boolean(false));
    }

    #[test]
    fn test_while_loop() {
        expect_value(
            r#"
        var a = 0;
        while (a < 5) {
            a = a + 1;
        }
        print a;
        "#,
            Value::Number(5.0),
        );
    }
}
