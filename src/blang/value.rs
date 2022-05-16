//pub type Value = f64;

use core::fmt;
use std::str::FromStr;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
}

pub struct ValueArray {
    values: Vec<Value>,
}

pub trait Printer {
    fn print(&self);
}

impl Printer for Value {
    fn print(&self) {
        print!("{}", self);
    }
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }
    pub fn add_value(&mut self, value: Value) {
        self.values.push(value);
    }
    pub fn get_value(&self, index: usize) -> Value {
        self.values[index]
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
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
            _ => Ok(Value::Number(
                s.parse::<f64>().expect("Failed to parse number"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_value_array() {
        let mut array = ValueArray::new();
        array.add_value(Value::Number(1.0));
        array.add_value(Value::Number(2.0));
        array.add_value(Value::Number(3.0));
        assert_eq!(array.get_value(0), Value::Number(1.0));
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn test_value_size() {
        assert_eq!(size_of::<Value>(), 8);
    }
}
