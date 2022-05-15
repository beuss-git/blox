use super::chunk::*;
use super::opcode;

pub(crate) fn main() {
    let mut chunk = Chunk::new();
    chunk.add_constant(3.5, 0);
    chunk.add_instruction(opcode::OP_RETURN, 0);
    chunk.add_instruction(opcode::OP_RETURN, 0);
    chunk.add_instruction(opcode::OP_RETURN, 1);
    chunk.add_instruction(opcode::OP_RETURN, 2);
    chunk.add_instruction(opcode::OP_RETURN, 2);

    chunk.disassemble_chunk("test chunk");
}
