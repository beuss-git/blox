use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::chunk::Chunk;
use super::value::native_function::{self, NativeFunction};
use super::{compiler::Compiler, opcode};

use super::value::{function::Function, Value};

const MAX_FRAMES: usize = 255;

struct CallFrame {
    function: Rc<Function>, // The function being called
    slot_offset: usize,     // The index of the first local slot in the call frame
    return_addr: usize,     // The address to return to after executing this callframe
}

pub struct Settings {
    pub trace_execution: bool,
    pub trace_stack: bool,
    pub disassembly: bool,
    pub frame_info: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            trace_execution: false,
            trace_stack: false,
            disassembly: false,
            frame_info: false,
        }
    }
}

impl CallFrame {
    fn new(function: Rc<Function>, slot_offset: usize, return_addr: usize) -> Self {
        Self {
            function,
            slot_offset,
            return_addr,
        }
    }
}
pub struct VM {
    chunk: Chunk,
    value_stack: Vec<Value>,
    last_printed: Option<Value>,
    globals: BTreeMap<Rc<str>, Value>,

    frame_stack: Vec<CallFrame>,
    pc: usize,
    settings: Settings,
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
    pub fn new(settings: Settings) -> Self {
        let mut vm = Self {
            chunk: Chunk::new(),
            value_stack: Vec::with_capacity(8192),
            last_printed: None,
            globals: BTreeMap::new(),
            frame_stack: Vec::with_capacity(MAX_FRAMES),
            pc: 0,
            settings,
        };
        vm.define_native("clock", native_function::clock);
        vm.define_native(
            "test_func_single_arg",
            native_function::test_func_single_arg,
        );
        vm.define_native(
            "test_func_add_two_args",
            native_function::test_func_add_two_args,
        );
        vm
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new();
        let compile_result = compiler.compile(source, &mut self.chunk, self.settings.disassembly);

        match &compile_result {
            Some(function) => {
                // Push the entry function onto the stack.
                self.push(Value::Function(function.clone()));

                // Call the entry function
                self.call(function, 0);

                self.run()
            }
            None => InterpretResult::CompileError,
        }
    }

    fn frame(&self) -> &CallFrame {
        //let frame_count = self.frame_count;
        let frame_count = self.frame_stack.len();

        &self.frame_stack[frame_count - 1]
    }

    fn get_value(&mut self, slot: usize) -> &Value {
        let absolute_slot = self.frame().slot_offset + slot;
        &self.value_stack[absolute_slot]
    }
    fn set_value(&mut self, slot: usize, value: &Value) {
        let absolute_slot = self.frame().slot_offset + slot;
        self.value_stack[absolute_slot] = value.clone();
    }
    fn run(&mut self) -> InterpretResult {
        // Set none (for testing purposes), we didn't print anything for the context
        self.last_printed = None;

        loop {
            if self.settings.trace_execution {
                self.chunk.disassemble_instruction(self.pc);
            }
            if self.settings.trace_stack {
                self.print_value_stack();
            }
            if self.settings.frame_info {
                println!("Frame count: {}", self.frame_stack.len());
                println!("Current slot: {}", self.frame().slot_offset);
            }

            // Decode the instruction
            match self.read_byte() {
                opcode::OP_GREATER => binary_op!(self, Boolean, >),
                opcode::OP_LESS => binary_op!(self, Boolean, <),
                //opcode::OP_ADD => binary_op!(self, Number, +),
                opcode::OP_MODULO => binary_op!(self, Number, %),
                opcode::OP_ADD => match (self.pop(), self.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
                    (Value::String(b), Value::String(a)) => {
                        self.push(Value::String(Rc::from(a.to_string() + &b.to_string())))
                    }
                    (Value::Number(b), Value::String(a)) => {
                        self.push(Value::String(Rc::from(a.to_string() + &b.to_string())))
                    }
                    (Value::Boolean(b), Value::String(a)) => {
                        self.push(Value::String(Rc::from(a.to_string() + &b.to_string())))
                    }
                    (Value::Nil, Value::String(b)) => {
                        self.push(Value::String(Rc::from(b.to_string() + "nil")))
                    }
                    (b, a) => {
                        self.runtime_error(
                            format!("Operands must be numbers. Got {:?} and {:?}", a, b).as_str(),
                        );
                        return InterpretResult::RuntimeError;
                    }
                },
                opcode::OP_SUBTRACT => binary_op!(self, Number, -),
                opcode::OP_MULTIPLY => binary_op!(self, Number, *),
                opcode::OP_DIVIDE => binary_op!(self, Number, /),
                opcode::OP_NOT => match self.value_stack.pop() {
                    Some(x) => self.push(Value::Boolean(x.is_falsey())),
                    _ => {
                        self.runtime_error("Stack is empty.");
                        return InterpretResult::RuntimeError;
                    }
                },
                opcode::OP_NEGATE => match self.value_stack.pop() {
                    Some(Value::Number(n)) => self.push(Value::Number(-n)),
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
                    //self.frame_mut().dec_pc(offset as usize);
                }
                opcode::OP_JUMP => {
                    let offset = self.read_short();
                    self.pc += offset as usize;
                    //self.frame_mut().add_pc(offset as usize);
                }
                opcode::OP_JUMP_IF_FALSE => {
                    let offset = self.read_short();
                    if self.peek().is_falsey() {
                        //self.frame_mut().add_pc(offset as usize);
                        self.pc += offset as usize;
                    }
                    // Else keep on churning
                }
                opcode::OP_CALL => {
                    let arg_count = self.read_byte() as usize;
                    let function = self.peek_n(arg_count).clone();
                    //self.push(Value::Number(self.pc as f64));
                    if !self.call_function(function, arg_count as u8) {
                        return InterpretResult::RuntimeError;
                    }
                }
                opcode::OP_RETURN => {
                    //return InterpretResult::Ok;
                    let result = self.pop();
                    // Gather all frame data before popping it
                    let slot = self.frame().slot_offset;
                    let return_addr = self.frame().return_addr;

                    self.frame_stack.pop();

                    if self.frame_stack.is_empty() {
                        self.pop();
                        return InterpretResult::Ok;
                    }

                    self.value_stack.truncate(slot);

                    self.pc = return_addr;

                    self.push(result);
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

                    let value = self.peek().clone();
                    // Set the value via the current frame
                    self.set_value(slot, &value);
                }
                opcode::OP_GET_LOCAL => {
                    let slot = self.read_byte() as usize;
                    // Get the value via the current frame
                    let value = self.get_value(slot).clone();
                    self.push(value);
                }
                opcode::OP_GET_GLOBAL => {
                    let name = self.read_constant();
                    match &name {
                        Value::String(str) => {
                            let value = self.globals.get(str).cloned();
                            if let Some(value) = value {
                                self.push(value);
                            } else {
                                self.runtime_error(&format!("Undefined variable '{}'.", name));
                                return InterpretResult::RuntimeError;
                            }
                        }
                        _ => {
                            self.runtime_error("Expected a string.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                opcode::OP_DEFINE_GLOBAL => {
                    // TODO: Check if it is a string
                    let name = self.read_constant();
                    let value = self.pop();
                    match name {
                        Value::String(str) => {
                            self.globals.insert(str, value);
                        }
                        _ => {
                            self.runtime_error("Expected a string.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                opcode::OP_SET_GLOBAL => {
                    let name = self.read_constant();
                    // Possible to get an iter instead of checking and then inserting?
                    // Can also just insert, check ret value and return error if it is not None, but make sure to delete value in there
                    match name {
                        Value::String(str) => {
                            let value = self.peek().clone();
                            if let Entry::Occupied(mut entry) = self.globals.entry(str) {
                                entry.insert(value);
                            } else {
                                self.runtime_error("Undefined variable.");
                                return InterpretResult::RuntimeError;
                            };
                        }
                        _ => {
                            self.runtime_error("Expected a string.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                opcode::OP_EQUAL => {
                    let (a, b) = (self.pop(), self.pop());
                    self.push(Value::Boolean(Value::is_same(a, b)));
                }
                _ => {
                    let op = self.read_byte();
                    self.runtime_error(format!("Unknown opcode: {}", op).as_str());
                    return InterpretResult::RuntimeError;
                }
            }
        }
    }

    fn peek_n(&self, n: usize) -> &Value {
        let stack_len = self.value_stack.len();
        &self.value_stack[stack_len - 1 - n]
    }
    fn peek(&self) -> &Value {
        self.value_stack.last().expect("Stack empty")
    }

    fn call(&mut self, function: &Rc<Function>, arg_count: u8) -> bool {
        if arg_count as usize != function.arity() {
            self.runtime_error(&format!(
                "Expected {} arguments, but got {}.",
                function.arity(),
                arg_count
            ));
            return false;
        }
        if self.frame_stack.len() == MAX_FRAMES {
            self.runtime_error("Stack overflow.");
            return false;
        }
        // Insert a new callframe
        let frame = CallFrame::new(
            function.clone(),
            self.value_stack.len() - arg_count as usize - 1,
            self.pc,
        );
        self.pc = function.start_address();
        self.frame_stack.push(frame);
        true
    }

    fn stack_trace(&self) {
        for frame in self.frame_stack.iter().rev() {
            let chunk = &self.chunk;
            let line = chunk.get_line(self.pc);
            println!(
                "[line {}] in {}",
                line,
                Value::Function(frame.function.clone())
            );
        }
    }
    // Is it proper to use Result like this? Are there better ways?
    fn call_function(&mut self, function: Value, arg_count: u8) -> bool {
        match &function {
            Value::Function(f) => self.call(f, arg_count),
            Value::NativeFunction(native) => {
                // Pop the arguments off the stack
                let args = self.pop_n(arg_count as usize);
                // Call the native function
                let result = native.call(&args);

                self.push(result);

                true
            }
            x => {
                self.runtime_error(format!("Can only call functions. Got {:?}", x).as_str());
                false
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        //self.frame_mut().add_pc(1);
        self.pc += 1;
        self.chunk.read_chunk(self.pc - 1)
        ////self.frame().byte_relative(-1)
    }

    fn read_short(&mut self) -> u16 {
        //self.frame_mut().add_pc(2);
        self.pc += 2;
        //((self.frame().byte_relative(-2) as u16) << 8) | (self.frame().byte_relative(-1) as u16)
        ((self.chunk.read_chunk(self.pc - 2) as u16) << 8)
            | (self.chunk.read_chunk(self.pc - 1) as u16)
        //((self.frame().byte_relative(-2) as u16) << 8) | (self.frame().byte_relative(-1) as u16)
    }
    fn read_constant(&mut self) -> Value {
        let constant_index = self.read_byte();
        //self.frame().get_value(constant_index as usize)
        self.chunk.get_value(constant_index as usize)
    }
    fn print_value_stack(&self) {
        for value in self.value_stack.iter() {
            print!("[ {} ]", value);
        }
        println!();
    }
    fn push(&mut self, value: Value) {
        self.value_stack.push(value);
    }
    fn stack_empty(&self) -> bool {
        self.value_stack.is_empty()
    }
    fn pop(&mut self) -> Value {
        self.value_stack.pop().expect("Stack is empty")
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            values.push(self.pop());
        }
        values
    }

    /// Resets the stack to the default state with reserved value
    fn reset_stack(&mut self) {
        //self.value_stack.truncate(0);
        self.value_stack.clear();
        self.frame_stack.clear();
    }

    fn runtime_error(&mut self, message: &str) {
        println!("[line {}] {}", self.chunk.get_line(self.pc), message);
        self.chunk.disassemble_instruction(self.pc);
        //self.frame().print_line(message);

        self.stack_trace();

        // Reset the stack to default state
        self.reset_stack();
    }

    fn define_native(&mut self, name: &str, function: fn(&[Value]) -> Value) {
        let native_function = NativeFunction::new(name, function);
        let name_ptr = Rc::from(name);
        self.push(Value::String(name_ptr));
        self.push(Value::NativeFunction(Rc::from(native_function.clone())));

        self.globals.insert(
            Rc::from(name),
            Value::NativeFunction(Rc::from(native_function)),
        );

        self.pop();
        self.pop();
    }

    #[allow(dead_code)]
    fn last_value(&self) -> Option<Value> {
        self.last_printed.clone()
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Settings::new())
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
    use std::rc::Rc;

    use crate::blox::{
        value::Value,
        vm::{InterpretResult, Settings},
    };

    use super::VM;

    // TODO: Remove this when proper chunk support is implemented
    fn new_vm() -> VM {
        VM::new(Settings::new())
    }

    fn expect_value(vm: &mut VM, expr: &str, expected: Value) {
        let res = vm.interpret(expr.to_string());
        assert_eq!(res, InterpretResult::Ok);
        assert_eq!(vm.last_value().unwrap(), expected);
    }

    fn expect_none(vm: &mut VM, expr: &str) {
        vm.interpret(expr.to_string());
        assert!(vm.last_value().is_none());
    }

    fn expect_interpreter_result(vm: &mut VM, expr: &str, expected: InterpretResult) {
        let res = vm.interpret(expr.to_string());
        assert_eq!(res, expected);
    }

    #[test]
    fn test_arithmetic() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 1+3*4;", Value::Number(13.0));
        expect_value(&mut vm, "print (1+3*3)/5+(4*3);", Value::Number(14.0));
    }

    #[test]
    fn test_modulo() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 5%2;", Value::Number(1.0));
        expect_value(&mut vm, "print 5%3;", Value::Number(2.0));
    }

    #[test]
    fn test_addition() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 1+3;", Value::Number(4.0));
        expect_value(&mut vm, "print 4+3;", Value::Number(7.0));
    }
    #[test]
    fn test_string_concatenation() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"print "Hello" + " " + "World!";"#,
            Value::String(Rc::from("Hello World!")),
        );
        expect_value(
            &mut vm,
            r#"print "Hel" + "lo" + ", " + "Wo" + "rld!";"#,
            Value::String(Rc::from("Hello, World!")),
        );
        expect_value(
            &mut vm,
            r#"print "one" + "two";"#,
            Value::String(Rc::from("onetwo")),
        );
        expect_value(
            &mut vm,
            r#"print "one" + 2;"#,
            Value::String(Rc::from("one2")),
        );
        expect_value(
            &mut vm,
            r#"print "one" + 2.1;"#,
            Value::String(Rc::from("one2.1")),
        );
        expect_value(
            &mut vm,
            r#"print "one" + true;"#,
            Value::String(Rc::from("onetrue")),
        );
        expect_value(
            &mut vm,
            r#"print "one" + false;"#,
            Value::String(Rc::from("onefalse")),
        );
        expect_value(
            &mut vm,
            r#"print "one" + nil;"#,
            Value::String(Rc::from("onenil")),
        );
    }
    #[test]
    fn test_subtraction() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 1-3;", Value::Number(-2.0));
        expect_value(&mut vm, "print 6-2;", Value::Number(4.0));
    }

    #[test]
    fn test_multiplication() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 2*10;", Value::Number(20.0));
        expect_value(&mut vm, "print 3*2*1;", Value::Number(6.0));
        expect_value(&mut vm, "print 1*2*3;", Value::Number(6.0));
    }

    #[test]
    fn test_division() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 2/2;", Value::Number(1.0));
        expect_value(&mut vm, "print 4/2;", Value::Number(2.0));
        expect_value(&mut vm, "print 2/4;", Value::Number(0.5));
        expect_value(&mut vm, "print 3/2/1;", Value::Number(1.5));
    }

    #[test]
    fn test_not() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print !true;", Value::Boolean(false));
        expect_value(&mut vm, "print !false;", Value::Boolean(true));
    }

    #[test]
    fn test_negation() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print -1;", Value::Number(-1.0));
        expect_value(&mut vm, "print -2;", Value::Number(-2.0));
        expect_value(&mut vm, "print -3;", Value::Number(-3.0));
        expect_value(&mut vm, "print --3;", Value::Number(3.0));
        expect_value(&mut vm, "print ---3;", Value::Number(-3.0));
    }

    #[test]
    fn test_nil() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print nil;", Value::Nil);
    }

    #[test]
    fn test_boolean() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print true;", Value::Boolean(true));
        expect_value(&mut vm, "print false;", Value::Boolean(false));
    }

    #[test]
    fn test_comments() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 1+3*4; // comment", Value::Number(13.0));
        expect_none(&mut vm, "// 1+3*4");
        expect_value(&mut vm, "print 1; //+3*4", Value::Number(1.0));
        expect_value(
            &mut vm,
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
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            "print !(5 - 4 > 3 * 2 == !nil);",
            Value::Boolean(true),
        );
    }

    #[test]
    fn test_not_equal() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 5 != 4;", Value::Boolean(true));
        expect_value(&mut vm, "print 5 != 5;", Value::Boolean(false));
        expect_value(&mut vm, "print true != true;", Value::Boolean(false));
        expect_value(&mut vm, "print false != false;", Value::Boolean(false));
        expect_value(&mut vm, "print true != false;", Value::Boolean(true));
        expect_value(&mut vm, "print false != true;", Value::Boolean(true));
        expect_value(&mut vm, r#"print "str" != "str";"#, Value::Boolean(false));
        expect_value(&mut vm, r#"print "str" != "st2";"#, Value::Boolean(true));
        expect_value(&mut vm, r#"print "str" != "st";"#, Value::Boolean(true));
    }

    #[test]
    fn test_equal() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 1 == 1;", Value::Boolean(true));
        expect_value(&mut vm, "print 1 == 2;", Value::Boolean(false));
        expect_value(&mut vm, "print 1 == 1.0;", Value::Boolean(true));
        expect_value(&mut vm, "print 1.0 == 1;", Value::Boolean(true));
        expect_value(&mut vm, "print 1.0 == 1.0;", Value::Boolean(true));
        expect_value(&mut vm, "print 1.0 == 2.0;", Value::Boolean(false));
        expect_value(&mut vm, "print 1.0 == 1.0;", Value::Boolean(true));
        expect_value(&mut vm, "print 1.0 == 2.0;", Value::Boolean(false));
        expect_value(&mut vm, "print 1.0 == 1.0;", Value::Boolean(true));
        expect_value(&mut vm, "print 1.0 == 2.0;", Value::Boolean(false));
        expect_value(&mut vm, "print true == true;", Value::Boolean(true));
        expect_value(&mut vm, "print false == false;", Value::Boolean(true));
        expect_value(&mut vm, "print true == false;", Value::Boolean(false));
        expect_value(&mut vm, "print false == true;", Value::Boolean(false));
        expect_value(&mut vm, r#"print "str" == "str";"#, Value::Boolean(true));
        expect_value(&mut vm, r#"print "str" == "st2";"#, Value::Boolean(false));
        expect_value(&mut vm, r#"print "str" == "st";"#, Value::Boolean(false));
    }

    #[test]
    fn test_greater() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 5 > 4;", Value::Boolean(true));
        expect_value(&mut vm, "print 5 > 5;", Value::Boolean(false));
        expect_value(&mut vm, "print 5 > 6;", Value::Boolean(false));
    }

    #[test]
    fn test_greater_equal() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 5 >= 4;", Value::Boolean(true));
        expect_value(&mut vm, "print 5 >= 5;", Value::Boolean(true));
        expect_value(&mut vm, "print 5 >= 6;", Value::Boolean(false));
    }

    #[test]
    fn test_less() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 5 < 4;", Value::Boolean(false));
        expect_value(&mut vm, "print 5 < 5;", Value::Boolean(false));
        expect_value(&mut vm, "print 5 < 6;", Value::Boolean(true));
    }

    #[test]
    fn test_less_equal() {
        let mut vm = new_vm();
        expect_value(&mut vm, "print 5 <= 4;", Value::Boolean(false));
        expect_value(&mut vm, "print 5 <= 5;", Value::Boolean(true));
        expect_value(&mut vm, "print 5 <= 6;", Value::Boolean(true));
    }

    #[test]
    fn test_global_variable_declaration() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = 1;
        var b = a + 3;
        print b + a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            &mut vm,
            r#"
        var a = 1;
        var b = 3 + 1;
        print b + a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            &mut vm,
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
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = 1;
        a = 2;
        print a;
        "#,
            Value::Number(2.0),
        );

        expect_value(
            &mut vm,
            r#"
        var a = 1;
        a = a + 2;
        print a;
        "#,
            Value::Number(3.0),
        );

        // Assign to invalid assignment target
        expect_interpreter_result(
            &mut vm,
            r#"
                a + b = c;
            "#,
            InterpretResult::CompileError,
        );

        // Assign to invalid assignment target
        expect_interpreter_result(
            &mut vm,
            r#"
                    var c = 3;
                    a + b = c;
                "#,
            InterpretResult::CompileError,
        );

        // Assign to invalid assignment target
        expect_interpreter_result(
            &mut vm,
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
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a;
        print a;
        "#,
            Value::Nil,
        );
    }

    #[test]
    fn test_nil_value() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = nil;
        print a;
        "#,
            Value::Nil,
        );
    }

    #[test]
    fn test_number_value() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = 5.0;
        print a;
        "#,
            Value::Number(5.0),
        );
    }
    #[test]
    fn test_string_value() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = "hello";
        print a;
        "#,
            Value::String(Rc::from("hello")),
        );
    }

    #[test]
    fn test_bool_value() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = true;
        print a;
        "#,
            Value::Boolean(true),
        );
    }

    #[test]
    fn test_value_assignment() {
        let mut vm = new_vm();

        // Number
        expect_value(
            &mut vm,
            r#"
        var a;
        a = 1.0;
        print a;
        "#,
            Value::Number(1.0),
        );

        // Bool
        expect_value(
            &mut vm,
            r#"
        var a;
        a = false;
        print a;
        "#,
            Value::Boolean(false),
        );

        // String
        expect_value(
            &mut vm,
            r#"
        var a;
        a = "hello";
        print a;
        "#,
            Value::String(Rc::from("hello")),
        );

        // Nil
        expect_value(
            &mut vm,
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
        let mut vm = new_vm();
        expect_value(
            &mut vm,
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
            &mut vm,
            r#"
            {
                var a = "outer";
                {
                    var a = 3;
                }
                print a;
            }

        "#,
            Value::String(Rc::from("outer")),
        );

        expect_value(
            &mut vm,
            r#"
            {
                var a = "outer";
                {
                    print a;
                }
            }

        "#,
            Value::String(Rc::from("outer")),
        );

        expect_value(
            &mut vm,
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
        let mut vm = new_vm();
        // Test undefined in local sope
        expect_interpreter_result(
            &mut vm,
            r#"
            {
                print a;
            }

        "#,
            InterpretResult::RuntimeError,
        );

        // Test gone out of scope
        expect_interpreter_result(
            &mut vm,
            r#"
            {
                var a = 3;
            }
            print a;

        "#,
            InterpretResult::RuntimeError,
        );

        // Test gone out of scope
        expect_interpreter_result(
            &mut vm,
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
        expect_interpreter_result(
            &mut vm,
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
        expect_interpreter_result(
            &mut vm,
            r#"
            print a;
            "#,
            InterpretResult::RuntimeError,
        );
    }
    #[test]
    fn test_if() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        if (true) {
            print "hello";
        }
        "#,
            Value::String(Rc::from("hello")),
        );

        expect_none(
            &mut vm,
            r#"
        if (false) {
            print "hello";
        }
        "#,
        );

        expect_value(
            &mut vm,
            r#"
        if (true) {
            print "hello";
        } else {
            print "world";
        }
        "#,
            Value::String(Rc::from("hello")),
        );

        expect_value(
            &mut vm,
            r#"
        if (false) {
            print "hello";
        } else {
            print "world";
        }
        "#,
            Value::String(Rc::from("world")),
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
        let mut vm = new_vm();
        expect_value(&mut vm, "print true and true;", Value::Boolean(true));
        expect_value(&mut vm, "print false and true;", Value::Boolean(false));
        expect_value(&mut vm, "print true and false;", Value::Boolean(false));
        expect_value(&mut vm, "print false and false;", Value::Boolean(false));

        expect_value(&mut vm, "print true or true;", Value::Boolean(true));
        expect_value(&mut vm, "print false or true;", Value::Boolean(true));
        expect_value(&mut vm, "print true or false;", Value::Boolean(true));
        expect_value(&mut vm, "print false or false;", Value::Boolean(false));
    }

    #[test]
    fn test_while_loop() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
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

    #[test]
    fn test_for_loop() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
        var a = 0;
        for (;a < 5;) {
            a = a + 1;
        }
        print a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            &mut vm,
            r#"
        var a = 0;
        for (;a < 5; a = a + 1) { }
        print a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            &mut vm,
            r#"
        var a;
        for (a = 0;a < 5; a = a + 1) { }
        print a;
        "#,
            Value::Number(5.0),
        );

        expect_value(
            &mut vm,
            r#"
        var a;
        var b = 0;
        for (a = 3;a < 5; a = a + 1) {
            b = b + 1;
        }
        print b;
        "#,
            Value::Number(2.0),
        );

        expect_value(
            &mut vm,
            r#"
        var b = 0;
        for (var a = 3;a < 5; a = a + 1) {
            b = b + 1;
        }
        print b;
        "#,
            Value::Number(2.0),
        );

        expect_value(
            &mut vm,
            r#"
            var b = 0;
        var a = 3;
        for ( a = 3;a < 5; a = a + 1) {
            b = b + 1;
        }
        print b;
        "#,
            Value::Number(2.0),
        );
    }

    #[test]
    fn test_function() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
            fun test() {
                return 5;
            }
            print test();
        "#,
            Value::Number(5.0),
        );

        expect_interpreter_result(
            &mut vm,
            r#"
            fun printer(x) {
                return "hello";
            }
            print printer();
        "#,
            InterpretResult::RuntimeError,
        );

        expect_value(
            &mut vm,
            r#"
            fun printer2(x) {
                return "hello";
            }
            print printer2(2);
        "#,
            Value::String(Rc::from("hello")),
        );

        expect_value(
            &mut vm,
            r#"
            fun printer3(x) {
                return x;
            }
            print printer3(2);
        "#,
            Value::Number(2.0),
        );

        expect_value(
            &mut vm,
            r#"
            fun fib(n) {
                if (n < 2) return n;
                return fib(n - 1) + fib(n - 2); 
            }

            print fib(10);
        "#,
            Value::Number(55.0),
        );

        expect_value(
            &mut vm,
            r#"
fun fib_non_recursive(n) {

    var a = 0;
    var b = 1;

    for (var i = 0; i < n; i = i + 1) {
        var tmp = a;
        a = b;
        b = tmp + b;
    }
    return a;
}
var n = 10;

print fib_non_recursive(n);


        "#,
            Value::Number(55.0),
        );

        // See if locals work properly in function in nested scope, they should reset and just use the function slot as local offset
        expect_value(
            &mut vm,
            r#"
        {
            fun returner(x) {
                print x;
                return x;
            }

            print returner(2);

        }
        "#,
            Value::Number(2.0),
        );

        expect_value(
            &mut vm,
            r#"
        {
            var a = 3;

            fun test(n) {
                return n;
            }

            print test(a);

        }
        "#,
            Value::Number(3.0),
        );

        expect_value(
            &mut vm,
            r#"
            var a = 3;

            fun test(n) {
                return n;
            }

            print test(a);
        "#,
            Value::Number(3.0),
        );
    }

    #[test]
    fn test_function_multiple_params() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
            fun test(x, y) {
                return x + y;
            }
            print test(2, 3);
        "#,
            Value::Number(5.0),
        );

        expect_interpreter_result(
            &mut vm,
            r#"
            fun test(x, y) {
                return x + y;
            }
            print test(2, 3, 4);
        "#,
            InterpretResult::RuntimeError,
        );

        expect_interpreter_result(
            &mut vm,
            r#"
            fun test(x) {
                return x + y;
            }
            print test(2, 3, 4);
        "#,
            InterpretResult::RuntimeError,
        );

        expect_interpreter_result(
            &mut vm,
            r#"
            fun test(x) {
                return x;
            }
            print test(2, 3, 4);
        "#,
            InterpretResult::RuntimeError,
        );

        expect_interpreter_result(
            &mut vm,
            r#"
            fun test(x, y) {
                return x;
            }
            print test();
        "#,
            InterpretResult::RuntimeError,
        );

        expect_interpreter_result(
            &mut vm,
            r#"
            fun test(x, y) {
                return x;
            }
            print test(1);
        "#,
            InterpretResult::RuntimeError,
        );

        expect_value(
            &mut vm,
            r#"
            fun test(x, y, z) {
                return x + y + z;
            }
            print test(2, 3, 4);
        "#,
            Value::Number(9.0),
        );
    }

    #[test]
    fn test_native_function() {
        let mut vm = new_vm();
        expect_value(
            &mut vm,
            r#"
            print test_func_single_arg(1234);
            "#,
            Value::Number(1234.0),
        );

        expect_value(
            &mut vm,
            r#"
            print test_func_add_two_args(9, 8);
            "#,
            Value::Number(17.0),
        );

        // Test more random placement
        expect_value(
            &mut vm,
            r#"
            fun clk() {
                return clock();
            }
            print clk();

            fun tester() {
                return 1 + 2;
            }

            print tester();
            "#,
            Value::Number(3.0),
        );
    }
}
