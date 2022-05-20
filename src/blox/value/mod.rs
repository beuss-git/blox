pub mod function;
pub mod native_function;
pub mod value_array;

use self::{function::Function, native_function::NativeFunction};
use core::fmt;
use std::{rc::Rc, str::FromStr};

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
    String(Rc<str>),
    Function(Rc<Function>),
    NativeFunction(Rc<NativeFunction>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl Value {
    // Checks if the value is falsey
    pub fn is_falsy(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Boolean(b) => !b,
            _ => false,
        }
    }

    // Checks if the values are the same
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

// Implements Display for Value
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(fun) => {
                // If it's empty it's a script
                if fun.name().is_empty() {
                    write!(f, "<script>")
                } else {
                    write!(f, "<fun '{}'>", fun.name())
                }
            }
            Value::NativeFunction(fun) => {
                write!(f, "<native fun '{}'>", fun.name())
            }
        }
    }
}

// Implements FromStr for Value
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
        assert_eq!(Value::Boolean(true).is_falsy(), false);
        assert_eq!(Value::Boolean(false).is_falsy(), true);
        assert_eq!(Value::Nil.is_falsy(), true);
        assert_eq!(Value::Number(1.0).is_falsy(), false);
    }
}
