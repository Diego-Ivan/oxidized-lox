mod expression;
mod interpreter;
mod once_cell;
mod parser;
mod scanner;
mod token;
mod utf8;

use crate::expression::Expression;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::cell::RefCell;
use std::io::{Read, Result as IOResult};
use std::path::Path;
use std::process::ExitCode;

static mut HAD_ERROR: RefCell<bool> = RefCell::new(false);
static mut HAD_RUNTIME_ERROR: RefCell<bool> = RefCell::new(false);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 1 {
        println!("Usage: lox [script]");
        return ExitCode::FAILURE;
    }

    let interpreter = Interpreter::new();
    match args.get(1) {
        Some(script) => run_file(script),
        None => run_prompt(&interpreter).unwrap(),
    }

    unsafe {
        if *HAD_ERROR.get_mut() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

fn run(source: &str, interpreter: &Interpreter) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
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
        unsafe {
            HAD_ERROR.replace(false);
            HAD_RUNTIME_ERROR.replace(false);
        }
    }

    Ok(())
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn runtime_error(error: InterpreterError) {
    println!("{error}");
    unsafe {
        HAD_RUNTIME_ERROR.replace(true);
    }
}

fn report(line: usize, s_where: &str, message: &str) {
    println!("[line {line}] Error {s_where}: {message}");
    unsafe {
        HAD_ERROR.replace(true);
    }
}
