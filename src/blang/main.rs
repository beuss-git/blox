use super::chunk::*;
use super::opcode;
use super::vm::VM;

pub(crate) fn main() {
    let mut chunk = Chunk::new();
    /*
    chunk.add_constant(3.5, 0);
    chunk.add_instruction(opcode::OP_NEGATE, 0);
    chunk.add_instruction(opcode::OP_RETURN, 0);
    */

    /*chunk.add_constant(3.0, 0);
    chunk.add_constant(2.0, 0);
    chunk.add_instruction(opcode::OP_MULTIPLY, 0);
    //chunk.add_instruction(opcode::OP_RETURN, 0);
    chunk.add_constant(2.0, 0);
    chunk.add_instruction(opcode::OP_SUBTRACT, 0);
    chunk.add_instruction(opcode::OP_RETURN, 0);*/

    // 1 + 2 * 3 - 4 / -5
    chunk.add_constant(1.0, 0);
    chunk.add_constant(2.0, 0);
    chunk.add_constant(3.0, 0);
    chunk.add_instruction(opcode::OP_MULTIPLY, 0);

    chunk.add_constant(4.0, 0);
    chunk.add_constant(5.0, 0);
    chunk.add_instruction(opcode::OP_NEGATE, 0);
    chunk.add_instruction(opcode::OP_DIVIDE, 0);
    chunk.add_instruction(opcode::OP_SUBTRACT, 0);
    chunk.add_instruction(opcode::OP_ADD, 0);

    chunk.add_instruction(opcode::OP_RETURN, 0);

    //chunk.disassemble_chunk("test chunk");

    let mut vm = VM::new(chunk);
    vm.interpret();
}
