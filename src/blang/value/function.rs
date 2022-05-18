use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    name: Rc<str>,
    chunk_index: usize,
    arity: usize,
}

#[derive(Clone, Copy)]
pub enum FunctionType {
    Native,
    UserDefined,
}

impl Function {
    pub fn new() -> Self {
        Self {
            name: Rc::from(""),
            chunk_index: 0,
            arity: 0,
        }
    }
    pub fn inc_arity(&mut self) {
        self.arity += 1;
    }
    pub fn set_chunk_index(&mut self, index: usize) {
        self.chunk_index = index;
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
    pub fn chunk_index(&self) -> usize {
        self.chunk_index
    }
}
