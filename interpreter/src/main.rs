mod interpreter;

use crate::interpreter::{Interpreter, InterpreterError};
use std::io::{Cursor, Read, Result as IOResult};
use std::path::Path;
use std::process::ExitCode;
use std::sync::Mutex;

static HAD_ERROR: Mutex<bool> = Mutex::new(false);
static HAD_RUNTIME_ERROR: Mutex<bool> = Mutex::new(false);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.is_empty() {
        println!("Usage: lox [script]");
        return ExitCode::FAILURE;
    }

    let interpreter = Interpreter::new();
    match args.get(1) {
        Some(script) => run_file(script),
        None => run_prompt(&interpreter).unwrap(),
    }

    if *HAD_ERROR.lock().unwrap() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(source: &str, interpreter: &Interpreter) {
    let scanner = syntax::Scanner::new(Cursor::new(source));

    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Syntax Error: {e}");
            return;
        }
    };

    let mut parser = syntax::Parser::new(&tokens);
    let statements = match parser.statements() {
        Ok(stmts) => stmts,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    if let Err(e) = interpreter.interpret(&statements) {
        runtime_error(e);
    }
}

fn run_file(path: impl AsRef<Path>) {
    let mut file = std::fs::File::open(path).unwrap();
    let mut contents = String::new();

    let interpreter = Interpreter::new();

    file.read_to_string(&mut contents).unwrap();
    run(&contents, &interpreter);
}

fn run_prompt(interpreter: &Interpreter) -> IOResult<()> {
    let reader = std::io::stdin();

    loop {
        print!(">");
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.is_empty() {
            break;
        }

        run(&line, interpreter);

        *HAD_ERROR.lock().unwrap() = false;
        *HAD_RUNTIME_ERROR.lock().unwrap() = false;
    }

    Ok(())
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn runtime_error(error: InterpreterError) {
    println!("{error}");
    *HAD_RUNTIME_ERROR.lock().unwrap() = true;
}

fn report(line: usize, s_where: &str, message: &str) {
    println!("[line {line}] Error {s_where}: {message}");
    *HAD_ERROR.lock().unwrap() = true;
}
