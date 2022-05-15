use super::opcode;
use super::value::*;

pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
}

/// Prints the instruction and returns the offset to the next instruction.
fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_chunk(offset + 1);
    print!("{} {}, ", name, constant_index);
    chunk.print_value(constant_index as usize);
    offset + 2
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn read_chunk(&self, offset: usize) -> u8 {
        self.code[offset]
    }
    /// Adds byte to the chunk
    fn write_chunk(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn get_value(&self, index: usize) -> Value {
        self.constants.get_value(index)
    }

    pub fn print_value(&self, index: usize) {
        self.constants.print_value(index)
    }

    /// Adds instruction into our chunk
    pub fn add_instruction(&mut self, instruction: u8) {
        self.write_chunk(instruction);
    }

    /// Adds constant into our chunk and returns the index of the constant
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.write_chunk(opcode::OP_CONSTANT);
        self.constants.add_value(value);
        let index = self.constants.len() - 1;

        // NOTE: Currently limited to 255 constants
        self.write_chunk(index as u8);

        index
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
        let name = opcode::get_name(*instruction);

        match *instruction {
            opcode::OP_RETURN => simple_instruction(name, offset),
            opcode::OP_CONSTANT => constant_instruction(name, &self, offset),
            _ => {
                println!("Invalid opcode {}", self.code[offset] as u8);
                offset + 1
            }
        }
    }
}
