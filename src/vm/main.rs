/*mod vm;
use crate::vm::chunk::*;
use crate::vm::opcodes::*;*/

use super::chunk::*;
use super::opcodes::*;
use super::value::*;

pub(crate) fn main() {
    let mut chunk = Chunk::new();
    chunk.add_instruction(OpCode::Return);
    chunk.add_instruction(OpCode::Return);
    chunk.add_constant(3.0);

    chunk.disassemble_chunk("test chunk");
}