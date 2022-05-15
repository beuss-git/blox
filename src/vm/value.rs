pub type Value = f64;

pub struct ValueArray {
    values: Vec<Value>,
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
    pub fn print_value(&self, index: usize) {
        println!("{}", self.values[index]);
    }
}
