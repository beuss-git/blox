use core::fmt;
use std::rc::Rc;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use super::Value;

#[derive(Clone)]
pub struct NativeFunction {
    name: Rc<str>,
    function: fn(&[Value]) -> Value,
}

impl NativeFunction {
    pub fn new(name: &str, function: fn(&[Value]) -> Value) -> Self {
        Self {
            name: Rc::from(name),
            function,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn call(&self, args: &[Value]) -> Value {
        (self.function)(args)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native {}>", self.name)
    }
}

// TODO: These should really be able to report errors to the VM
// Examples are invalid arity or invalid types

// Native clock function
pub fn clock(_: &[Value]) -> Value {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // Return the number of seconds since the UNIX epoch with *some* accuracy
    Value::Number(since_epoch.as_nanos() as f64 / 1_000_000_000.0)
}

pub fn read_line(_: &[Value]) -> Value {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    Value::String(Rc::from(input))
}

pub fn to_number(args: &[Value]) -> Value {
    if let Value::String(s) = args[0].clone() {
        Value::Number(s.parse().unwrap())
    } else {
        panic!("to_number: expected string")
    }
}

/*
    These are very dangerous functions at the moment, no arg checks
*/

pub fn test_func_single_arg(args: &[Value]) -> Value {
    args[0].clone()
}

pub fn test_func_add_two_args(args: &[Value]) -> Value {
    match args.len() {
        2 => match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => Value::Nil,
        },
        _ => Value::Nil,
    }
}
