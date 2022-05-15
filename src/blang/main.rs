use super::chunk::*;
use super::opcode;
use super::vm::VM;

pub(crate) fn main() {
    let mut chunk = Chunk::new();
    chunk.add_constant(3.5, 0);
    chunk.add_instruction(opcode::OP_NEGATE, 0);
    chunk.add_instruction(opcode::OP_RETURN, 0);

    //chunk.disassemble_chunk("test chunk");

    let mut vm = VM::new(chunk);
    vm.interpret();
}
