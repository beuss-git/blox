use std::{fs, io::Write};

mod blox;

fn main() {
    //let source = fs::read_to_string("sources/test.bl").unwrap();
    let mut vm = blox::vm::VM::new();
    //vm.interpret(source);
    // REPL
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
        vm.interpret(input.clone());

        input.clear();
    }
}
