use super::{Interpreter, InterpreterResult, LoxValue, Statement};
use std::fmt::Debug;

macro_rules! native_function {
    ($r_type: ident) => {
        struct $r_type;
        impl std::fmt::Debug for $r_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("<native fn>")
            }
        }
    };
}
pub trait Callable: Debug {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<LoxValue>,
    ) -> InterpreterResult<LoxValue>;
    fn arity(&self) -> usize;
}

struct LoxFunction {
    statement: Statement,
}

native_function!(ReadLine);
impl Callable for ReadLine {
    fn call(
        &self,
        _interpreter: &Interpreter,
        _arguments: Vec<LoxValue>,
    ) -> InterpreterResult<LoxValue> {
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        Ok(LoxValue::String(line))
    }

    fn arity(&self) -> usize {
        0
    }
}
