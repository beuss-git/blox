use super::opcode;
use super::value::*;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: ValueArray,
    line_data: Vec<u8>, // Lets assume that a line can't have more than 255 bytes
}

/// Prints the instruction and returns the offset to the next instruction.
fn simple_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    println!("{}: {}", chunk.get_line(offset), name);
    offset + 1
}

/// Prints the instruction and returns the offset to the next instruction.
fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_chunk(offset + 1);
    print!("{}: {} {}, ", chunk.get_line(offset), name, constant_index);
    chunk.get_value(constant_index as usize).print();
    println!();
    offset + 2
}

impl Chunk {
    pub fn new() -> Self {
        // TODO: Preallocate the code and line data arrays (?)
        Self {
            code: Vec::new(),
            constants: ValueArray::new(),
            line_data: Vec::new(),
        }
    }

    pub fn read_chunk(&self, offset: usize) -> u8 {
        self.code[offset]
    }
    /// Adds byte to the chunk
    fn write_chunk(&mut self, byte: u8, line: usize) {
        // RLE compression of line data
        if self.line_data.len() == line + 1 {
            self.line_data[line] += 1;
        } else {
            self.line_data.push(1);
        }
        self.code.push(byte);
    }

    // https://www.csfieldguide.org.nz/en/chapters/coding-compression/run-length-encoding/
    fn get_line(&self, offset: usize) -> usize {
        let mut total: usize = 0;
        for (line, length) in self.line_data.iter().enumerate() {
            total += *length as usize;
            if total > offset {
                return line;
            }
        }

        assert!(false);
        return 0;
    }

    pub fn get_value(&self, index: usize) -> Value {
        self.constants.get_value(index)
    }

    /// Adds instruction into our chunk
    pub fn add_instruction(&mut self, instruction: u8, line: usize) {
        self.write_chunk(instruction, line);
    }

    /// Adds constant into our chunk and returns the index of the constant
    pub fn add_constant(&mut self, value: Value, line: usize) -> usize {
        self.write_chunk(opcode::OP_CONSTANT, line);
        self.constants.add_value(value);
        let index = self.constants.len() - 1;

        // NOTE: Currently limited to 255 constants
        self.write_chunk(index as u8, line);

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
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        // Print out the instruction offset
        print!("{:04} ", offset);

        let instruction = &self.code[offset];
        // Format the instruction back to the OpCode name
        let name = opcode::get_name(*instruction);

        match *instruction {
            opcode::OP_NEGATE => simple_instruction(name, &self, offset),
            opcode::OP_RETURN => simple_instruction(name, &self, offset),
            opcode::OP_CONSTANT => constant_instruction(name, &self, offset),
            _ => {
                println!("Invalid opcode {}", self.code[offset] as u8);
                offset + 1
            }
        }
    }
}
