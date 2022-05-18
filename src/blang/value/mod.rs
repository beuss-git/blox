pub mod function;
pub mod value_array;

use core::fmt;
use std::{rc::Rc, str::FromStr};

use self::function::Function;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
    String(Rc<str>),
    Function(Rc<Function>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Boolean(b) => !b,
            _ => false,
        }
    }
    pub fn is_same(a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

pub trait Printer {
    fn print(&self);
}

impl Printer for Value {
    fn print(&self) {
        print!("{}", self);
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(fun) => write!(f, "<fun {}>", fun.name),
        }
    }
}

impl FromStr for Value {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(Value::Boolean(true)),
            "false" => Ok(Value::Boolean(false)),
            "nil" => Ok(Value::Nil),
            s => Ok(match s.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(_) => Value::String(Rc::from(s)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_value_size() {
        assert_eq!(size_of::<Value>(), 24);
    }

    #[test]
    fn test_is_falsey() {
        assert_eq!(Value::Boolean(true).is_falsey(), false);
        assert_eq!(Value::Boolean(false).is_falsey(), true);
        assert_eq!(Value::Nil.is_falsey(), true);
        assert_eq!(Value::Number(1.0).is_falsey(), false);
    }
}
