use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: Rc<str>,
    pub chunk_index: usize,
    pub arity: usize,
}

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
}
