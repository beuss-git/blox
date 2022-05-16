use std::{cell::RefCell, collections::HashMap};

use super::{
    chunk::Chunk,
    compiler::Compiler,
    opcode,
    value::{Printer, Value},
};

const DEBUG_TRACE_EXECUTION: bool = false;
pub struct VM {
    chunk: Chunk,
    pc: usize,
    stack: Vec<Value>,
    last_value: Option<Value>,
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
            last_value: None,
            globals: HashMap::new(),
        }
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(source, &mut self.chunk);
        if !compiler.compile() {
            return InterpretResult::CompileError;
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
                    println!("{}", self.pop());
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
                    self.last_value = Some(self.pop());
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

    fn read_byte(&mut self) -> u8 {
        self.pc += 1;
        self.chunk.code[self.pc - 1]
    }
    fn read_constant(&mut self) -> Value {
        let constant_index = self.read_byte();
        self.chunk.get_value(constant_index as usize)
    }
    fn print_stacktrace(&self) {
        println!("Stack trace:");
        for value in self.stack.iter() {
            print!("[ {} ]", value);
        }
        println!();
    }
    fn push(&mut self, value: Value) {
        self.stack.push(value);
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
        self.last_value.clone()
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

#[cfg(test)]
mod tests {
    use crate::blang::{chunk::Chunk, value::Value};

    use super::VM;

    // TODO: Remove this when proper chunk support is implemented
    fn new_vm() -> VM {
        VM::new(Chunk::new())
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = new_vm();

        vm.interpret("1+3*4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(13.0));

        vm = new_vm();
        vm.interpret("(1+3*3)/5+(4*3);".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(14.0));
    }
    #[test]
    fn test_addition() {
        let mut vm = new_vm();
        vm.interpret("1+3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(4.0));

        vm = new_vm();
        vm.interpret("4+3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(7.0));
    }
    #[test]
    fn test_string_concatenation() {
        let mut vm = new_vm();
        vm.interpret(r#""Hello" + " " + "World!";"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("Hello World!".to_string())
        );

        vm = new_vm();
        vm.interpret(r#""Hel" + "lo" + ", " + "Wo" + "rld!";"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("Hello, World!".to_string())
        );

        vm = new_vm();
        vm.interpret(r#""one" + "two";"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("onetwo".to_string())
        );

        vm = new_vm();
        vm.interpret(r#""one" + 2;"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::String("one2".to_string()));

        vm = new_vm();
        vm.interpret(r#""one" + 2.1;"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("one2.1".to_string())
        );

        vm = new_vm();
        vm.interpret(r#""one" + true;"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("onetrue".to_string())
        );

        vm = new_vm();
        vm.interpret(r#""one" + false;"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("onefalse".to_string())
        );

        vm = new_vm();
        vm.interpret(r#""one" + nil;"#.to_string());
        assert_eq!(
            vm.last_value().unwrap(),
            Value::String("onenil".to_string())
        );
    }
    #[test]
    fn test_subtraction() {
        let mut vm = new_vm();
        vm.interpret("1-3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(-2.0));

        vm = new_vm();
        vm.interpret("6-2;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(4.0));
    }

    #[test]
    fn test_multiplication() {
        let mut vm = new_vm();

        vm.interpret("2*10;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(20.0));

        vm = new_vm();
        vm.interpret("3*2*1;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(6.0));

        vm = new_vm();
        vm.interpret("1*2*3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(6.0));
    }

    #[test]
    fn test_division() {
        let mut vm = new_vm();

        vm.interpret("2/2;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(1.0));

        vm = new_vm();
        vm.interpret("4/2;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(2.0));

        vm = new_vm();
        vm.interpret("2/4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(0.5));

        vm = new_vm();
        vm.interpret("3/2/1;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(1.5));
    }

    #[test]
    fn test_not() {
        let mut vm = new_vm();

        vm.interpret("!true;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("!false;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_negation() {
        let mut vm = new_vm();

        vm.interpret("-1;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(-1.0));

        vm = new_vm();
        vm.interpret("-2;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(-2.0));

        vm = new_vm();
        vm.interpret("-3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(-3.0));

        vm = new_vm();
        vm.interpret("--3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(3.0));

        vm = new_vm();
        vm.interpret("---3;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(-3.0));
    }

    #[test]
    fn test_nil() {
        let mut vm = new_vm();

        vm.interpret("nil;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Nil);
    }

    #[test]
    fn test_boolean() {
        let mut vm = new_vm();

        vm.interpret("true;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("false;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_comments() {
        let mut vm = new_vm();

        vm.interpret("1+3*4; // comment".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(13.0));

        vm = new_vm();
        vm.interpret("// 1+3*4".to_string());
        assert!(vm.last_value().is_none());

        vm = new_vm();
        vm.interpret("1; //+3*4".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_comparison() {
        let mut vm = new_vm();

        vm.interpret("!(5 - 4 > 3 * 2 == !nil);".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_not_equal() {
        let mut vm = new_vm();

        vm.interpret("5 != 4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("5 != 5;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("true != true;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("false != false;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("true != false;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("false != true;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret(r#""str" != "str";"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret(r#""str" != "st2";"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret(r#""str" != "st";"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_equal() {
        let mut vm = new_vm();

        vm.interpret("1 == 1;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("1 == 2;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("1 == 1.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("1.0 == 1;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("1.0 == 1.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("1.0 == 2.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("1.0 == 1.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("1.0 == 2.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("1.0 == 1.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("1.0 == 2.0;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("true == true;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("false == false;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("true == false;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("false == true;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret(r#""str" == "str";"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret(r#""str" == "st2";"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret(r#""str" == "st";"#.to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_greater() {
        let mut vm = new_vm();

        vm.interpret("5 > 4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("5 > 5;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("5 > 6;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_greater_equal() {
        let mut vm = new_vm();

        vm.interpret("5 >= 4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("5 >= 5;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("5 >= 6;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_less() {
        let mut vm = new_vm();

        vm.interpret("5 < 4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("5 < 5;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("5 < 6;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_less_equal() {
        let mut vm = new_vm();

        vm.interpret("5 <= 4;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(false));

        vm = new_vm();
        vm.interpret("5 <= 5;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));

        vm = new_vm();
        vm.interpret("5 <= 6;".to_string());
        assert_eq!(vm.last_value().unwrap(), Value::Boolean(true));
    }
}
