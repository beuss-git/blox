use core::fmt;
use std::rc::Rc;

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

pub fn clock(_: &[Value]) -> Value {
    Value::Number(1234.0)
}
