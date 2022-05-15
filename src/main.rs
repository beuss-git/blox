use std::io::Write;

mod vm;
use crate::vm::chunk::*;
use crate::vm::opcodes::*;

fn main() {
    let mut chunk = Chunk { code: Vec::new() };
    chunk.code.push(OpCode::Return);

    chunk.disassemble_chunk("test chunk");

    println!("Capacity: {}", chunk.code.capacity());
    println!("Len: {}", chunk.code.len());
    println!("Data: {:?}", chunk.code);

    // REPL
    /*
    let mut input = String::new();
    loop {
        print!("> ");
        std::io::stdout()
            .flush()
            .expect("Failed to flush standard output");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if input.trim() == "exit" {
            break;
        }

        input.clear();
    }
    */
}
