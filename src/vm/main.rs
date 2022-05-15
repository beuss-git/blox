use super::chunk::*;
use super::opcode;

pub(crate) fn main() {
    let mut chunk = Chunk::new();
    chunk.add_constant(3.0);
    chunk.add_instruction(opcode::OP_RETURN);

    chunk.disassemble_chunk("test chunk");
}
