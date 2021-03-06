use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    name: Rc<str>,        // name of the function
    arity: usize,         // number of arguments
    start_address: usize, // start address of the function
}

#[derive(Clone, Copy, PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

impl Function {
    pub fn new() -> Self {
        Self {
            name: Rc::from(""),
            arity: 0,
            start_address: 0,
        }
    }
    pub fn start_address(&self) -> usize {
        self.start_address
    }
    pub fn set_arity(&mut self, arity: usize) {
        self.arity = arity;
    }
    pub fn inc_arity(&mut self) {
        self.arity += 1;
    }
    pub fn set_start_address(&mut self, address: usize) {
        self.start_address = address;
    }
    pub fn set_name(&mut self, name: String) {
        self.name = Rc::from(name);
    }
    pub fn arity(&self) -> usize {
        self.arity
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
