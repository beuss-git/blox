use std::{fs, io::Write};

mod blox;

mod argparse;
use argparse::ArgParse;

#[cfg(not(tarpaulin_include))]
fn main() {
    /*
    This works very similiarly to python and a lot of other languages
    if you provide a file path, it will try to compile and run it
    If nothing is provided, it will run the REPL
    */

    let mut parser = ArgParse::new("BLOX");
    parser
        .arg("--disassembly")
        .help("Prints the disassembly of the program")
        .arg("--trace_stack")
        .help("Prints value stack trace per instruction")
        .arg("--trace_execution")
        .help("Prints disassembly per instruction")
        .arg("--frame_info")
        .help("Prints frame information per instruction")
        .arg("--help")
        .help("Prints this message!");

    parser.parse();

    if parser.get("--help").is_some() {
        parser.print_help();
        return;
    }

    // Create settings for the VM
    let mut settings = blox::vm::Settings::new();

    if parser.get("--disassembly").is_some() {
        settings.disassembly = true;
    }
    if parser.get("--trace_stack").is_some() {
        settings.trace_stack = true;
    }
    if parser.get("--trace_execution").is_some() {
        settings.trace_execution = true;
    }
    if parser.get("--frame_info").is_some() {
        settings.frame_info = true;
    }

    let non_bound_args = parser.get_non_bound();

    if !non_bound_args.is_empty() {
        // Found non-bound arg(s) assume the first one is a source file path
        let path = non_bound_args.first().unwrap();

        if path.is_empty() {
            println!("No source file specified");
            return;
        }

        match fs::read_to_string(path) {
            Ok(source) => {
                // Create the VM
                let mut vm = blox::vm::VM::new(settings);

                // Interpret the source
                vm.interpret(source);
            }
            Err(err) => {
                println!("Failed to read source file: {}", err);
                return;
            }
        }
    } else {
        // No source file provided, run the REPL

        // Create the VM
        let mut vm = blox::vm::VM::new(settings);

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
}
