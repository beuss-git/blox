use std::io::Write;

mod vm;

fn main() {
    vm::main::main();
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
