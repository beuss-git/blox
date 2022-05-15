pub type Value = f64;

pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }
    fn add_value(&mut self, value: Value) {
        self.values.push(value);
    }
}
