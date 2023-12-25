extern crate rustyline;
extern crate tiny_library;

use rustyline::{error::ReadlineError, DefaultEditor};
use std::io::Read;
use tiny_library::reader::Reader;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 1 {
        process_files(&args);
    } else {
        run_stdin();
        // inter();
    }
}

fn process_files(args: &Vec<String>) {
    for file in args {
        let content = std::fs::read_to_string(&file).unwrap();
        process(&file, &content, true);
    }
}

#[allow(dead_code)]
fn run_stdin() {
    let mut content = String::new();
    std::io::stdin()
        .read_to_string(&mut content)
        .expect("error readin stdin");
    process("_stdin_.tiny", &content, false);
}

#[allow(dead_code)]
fn inter() {
    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history("./history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        match rl.readline(">>> ") {
            Ok(line) => {
                process("_repl_.tiny", &line, false);
                rl.add_history_entry(line.as_str()).unwrap();
            }
            Err(ReadlineError::Eof) => break,
            Err(ReadlineError::Interrupted) => {}
            Err(err) => {
                eprintln!("Error: {err}");
                break;
            }
        };
    }
    rl.save_history("./history.txt").unwrap();
}

fn process(name: &str, content: &str, verbose: bool) {
    let mut reader = Reader::new(name, content);
    if verbose {
        eprintln!("Compiling {}", reader.name);
    }
    let dump = match std::env::var("DUMP") {
        Ok(val) if val == "1" => true,
        _ => false,
    };
    loop {
        match reader.read() {
            Some(Ok(form)) => {
                if dump {
                    form.dump("")
                }
            }
            Some(Err(err)) => {
                println!("Error: {:?}", err);
                break;
            }
            None => break,
        }
    }
}
