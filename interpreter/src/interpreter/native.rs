use crate::interpreter::{LoxValue, NativeResult};
use rand::Rng;
use std::rc::Rc;
use std::time::SystemTime;

pub(super) fn clock(_args: &[LoxValue]) -> NativeResult<LoxValue> {
    let time = SystemTime::now();
    let unix_time = time.duration_since(SystemTime::UNIX_EPOCH)?;

    Ok(LoxValue::Number(unix_time.as_secs_f64()))
}

pub(super) fn read_line(_args: &[LoxValue]) -> NativeResult<LoxValue> {
    let stdin = std::io::stdin();
    let mut line = String::new();

    stdin.read_line(&mut line)?;
    line.pop();

    Ok(LoxValue::String(Rc::new(line)))
}

pub(super) fn random(args: &[LoxValue]) -> NativeResult<LoxValue> {
    let (mut inf, mut sup) = match (&args[0], &args[1]) {
        (LoxValue::Number(a), LoxValue::Number(b)) => (*a as i64, *b as i64),
        _ => {
            eprintln!("Parameters in random must be numbers");
            return Ok(LoxValue::Nil);
        }
    };

    if inf > sup {
        std::mem::swap(&mut inf, &mut sup);
    }

    let mut rand = rand::rng();
    let random = rand.random_range(inf..sup);

    Ok(LoxValue::Number(random as f64))
}

pub(super) fn string_to_number(args: &[LoxValue]) -> NativeResult<LoxValue> {
    let source = match &args[0] {
        LoxValue::String(str) => str.trim(),
        _ => {
            eprintln!("Argument is not a number");
            return Ok(LoxValue::Nil);
        }
    };

    let num: f64 = source.parse()?;
    Ok(LoxValue::Number(num))
}
