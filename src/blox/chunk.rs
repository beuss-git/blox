use super::{
    opcode,
    value::{value_array::ValueArray, Printer, Value},
};

#[derive(PartialEq, Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: ValueArray,
    line_data: Vec<u8>, // Lets assume that a line can't have more than 255 bytes
}

#[cfg(not(tarpaulin_include))]
// Prints the instruction and returns the offset to the next instruction.
fn simple_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    println!("{}: {}", chunk.get_line(offset), name);
    offset + 1
}

#[cfg(not(tarpaulin_include))]
// Prints the instruction and returns the offset to the next instruction.
fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.read_chunk(offset + 1);
    //print!("{}: {} {}, ", chunk.get_line(offset), name, constant_index);
    print!("{}: {}, slot {}, ", chunk.get_line(offset), name, slot);
    chunk.get_value(slot as usize).print();
    println!();
    offset + 2
}

#[cfg(not(tarpaulin_include))]
// Prints the instruction and returns the offset to the next instruction.
fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.read_chunk(offset + 1);
    println!("{}: {} {}", chunk.get_line(offset), name, slot);
    offset + 2
}

#[cfg(not(tarpaulin_include))]
// Prints the instruction and returns the offset to the next instruction.
fn jump_instruction(name: &str, positive: bool, chunk: &Chunk, offset: usize) -> usize {
    let offset_jump: u16 =
        ((chunk.read_chunk(offset + 1) as u16) << 8) | chunk.read_chunk(offset + 2) as u16;

    let line = chunk.get_line(offset);
    // Print 16-bit jump offset
    let address = if positive {
        offset + 3 + offset_jump as usize
    } else {
        offset + 3 - offset_jump as usize
    };
    println!("{}: {} {}", line, name, address);
    offset + 3
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

    // Adds byte to the chunk
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        // RLE compression of line data
        if self.line_data.len() == line + 1 {
            self.line_data[line] += 1;
        } else {
            self.line_data.push(1);
        }
        self.code.push(byte);
    }

    // https://www.csfieldguide.org.nz/en/chapters/coding-compression/run-length-encoding/
    pub fn get_line(&self, offset: usize) -> usize {
        let mut total: usize = 0;
        for (line, length) in self.line_data.iter().enumerate() {
            total += *length as usize;
            if total > offset {
                return line;
            }
        }

        unreachable!("Line should always be found");
    }

    pub fn get_value(&self, index: usize) -> Value {
        self.constants.get_value(index)
    }

    // Adds constant into our chunk and returns the index of the constant
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.add_value(value);

        self.constants.len() - 1
    }

    pub fn patch_constant(&mut self, index: usize, value: Value) {
        self.constants.set_value(index, value);
    }
    pub fn get_constant(&mut self, index: usize) -> Value {
        self.constants.get_value(index)
    }

    // Disassembles the chunk
    #[cfg(not(tarpaulin_include))]
    pub fn disassemble_chunk_from(&self, name: &str, start: usize) {
        println!("== {} ==", name);

        let mut offset = start;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    // Disassembles the instruction at the given offset
    #[cfg(not(tarpaulin_include))]
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        // Print out the instruction offset
        print!("{:04} ", offset);

        let instruction = &self.code[offset];
        // Format the instruction back to the OpCode name
        let name = opcode::get_name(*instruction);

        // TODO: rustify this, could also just check instruction type
        match *instruction {
            // TODO: Run through implementations
            opcode::OP_EQUAL => simple_instruction(name, self, offset),
            opcode::OP_GREATER => simple_instruction(name, self, offset),
            opcode::OP_LESS => simple_instruction(name, self, offset),
            opcode::OP_ADD => simple_instruction(name, self, offset),
            opcode::OP_SUBTRACT => simple_instruction(name, self, offset),
            opcode::OP_MULTIPLY => simple_instruction(name, self, offset),
            opcode::OP_DIVIDE => simple_instruction(name, self, offset),
            opcode::OP_NOT => simple_instruction(name, self, offset),
            opcode::OP_NEGATE => simple_instruction(name, self, offset),
            opcode::OP_PRINT => simple_instruction(name, self, offset),
            opcode::OP_JUMP_BACK => jump_instruction(name, false, self, offset),
            opcode::OP_JUMP => jump_instruction(name, true, self, offset),
            opcode::OP_JUMP_IF_FALSE => jump_instruction(name, true, self, offset),
            opcode::OP_CALL => byte_instruction(name, self, offset),
            opcode::OP_RETURN => simple_instruction(name, self, offset),
            opcode::OP_CONSTANT => constant_instruction(name, self, offset),
            opcode::OP_NIL => simple_instruction(name, self, offset),
            opcode::OP_TRUE => simple_instruction(name, self, offset),
            opcode::OP_FALSE => simple_instruction(name, self, offset),
            opcode::OP_POP => simple_instruction(name, self, offset),
            opcode::OP_GET_LOCAL => byte_instruction(name, self, offset),
            opcode::OP_SET_LOCAL => byte_instruction(name, self, offset),
            opcode::OP_GET_GLOBAL => constant_instruction(name, self, offset),
            opcode::OP_DEFINE_GLOBAL => constant_instruction(name, self, offset),
            opcode::OP_SET_GLOBAL => constant_instruction(name, self, offset),
            _ => {
                println!("Invalid opcode {}", self.code[offset] as u8);
                offset + 1
            }
        }
    }
}
