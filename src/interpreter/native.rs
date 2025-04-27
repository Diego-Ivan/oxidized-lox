use crate::interpreter::{InterpreterResult, LoxValue};
use std::time::SystemTime;

pub(super) fn clock_native(_args: &[LoxValue]) -> InterpreterResult<LoxValue> {
    let time = SystemTime::now();

    // TODO: turn this into an interpreter error
    let unix_time = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();

    Ok(LoxValue::Number(unix_time.as_secs_f64()))
}
