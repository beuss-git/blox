use super::{
    chunk::Chunk,
    compiler::Compiler,
    opcode,
    value::{Printer, Value},
};

const DEBUG_TRACE_EXECUTION: bool = true;
pub struct VM {
    chunk: Chunk,
    pc: usize,
    stack: Vec<Value>,
}

macro_rules! binary_op {
        ($self:ident, $op:tt) => {
            match ($self.pop(), $self.pop()) {
                (Value::Number(a), Value::Number(b)) => $self.push(Value::Number(a $op b)),
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
            // Decode the instruction
            match self.read_byte() {
                opcode::OP_ADD => binary_op!(self, +),
                opcode::OP_SUBTRACT => binary_op!(self, -),
                opcode::OP_MULTIPLY => binary_op!(self, *),
                opcode::OP_DIVIDE => binary_op!(self, /),
                opcode::OP_NEGATE => match self.stack.pop() {
                    Some(Value::Number(n)) => self.stack.push(Value::Number(-n)),
                    _ => {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                },
                opcode::OP_RETURN => {
                    self.pop().print();
                    println!();
                    return InterpretResult::Ok;
                }
                opcode::OP_CONSTANT => {
                    let constant: Value = self.read_constant();
                    //constant.print();
                    //println!();
                    self.push(constant);
                }
                _ => {
                    println!("Unknown opcode: {}", self.read_byte());
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
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}
