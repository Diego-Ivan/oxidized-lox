mod scanner;
mod token;
mod utf8;

use crate::scanner::Scanner;
use std::cell::RefCell;
use std::io::Result as IOResult;
use std::path::Path;
use std::process::ExitCode;

static mut HAD_ERROR: RefCell<bool> = RefCell::new(false);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        println!("Usage: lox [script]");
        return ExitCode::FAILURE;
    }

    match args.get(1) {
        Some(script) => run_file(script),
        None => run_prompt().unwrap(),
    }

    unsafe {
        if *HAD_ERROR.get_mut() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

fn run(source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{token}");
    }
}

fn run_file(path: impl AsRef<Path>) {
    todo!()
}

fn run_prompt() -> IOResult<()> {
    let reader = std::io::stdin();

    loop {
        print!(">");
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.is_empty() {
            break;
        }

        run(&line);
        unsafe {
            HAD_ERROR.replace(false);
        }
    }

    Ok(())
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, s_where: &str, message: &str) {
    println!("[line {line}] Error {s_where}: {message}");
    unsafe {
        HAD_ERROR.replace(true);
    }
}
