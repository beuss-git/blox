pub type Value = f64;

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
