use super::opcodes::OpCode;
use super::value::*;

pub struct Chunk {
    code: Vec<OpCode>,
    constants: ValueArray,
}

/// Prints the instruction and returns the offset to the next instruction.
fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: ValueArray::new(),
        }
    }
    /// Adds instruction into our chunk
    pub fn add_instruction(&mut self, instruction: OpCode) {
        self.code.push(instruction);
    }
    pub fn add_constant(&mut self, value: Value) {
        self.constants.add_value(value);
    }

    /// Disassembles the chunk
    pub fn disassemble_chunk(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    /// Disassembles the instruction at the given offset
    fn disassemble_instruction(&self, offset: usize) -> usize {
        // Print out the instruction offset
        print!("{:04} ", offset);

        let instruction = &self.code[offset];
        // Format the instruction back to the OpCode name
        let name = format!("{:?}", instruction);

        match instruction {
            OpCode::Return => simple_instruction(name.as_str(), offset),
            _ => {
                println!("Invalid opcode {}", self.code[offset] as u8);
                offset + 1
            }
        }
    }
}