use std::io::Write;

fn main() {
    let mut input = String::new();

    // REPL
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
}
