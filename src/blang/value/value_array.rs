use super::Value;

#[derive(Clone, PartialEq, Debug)]
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
    pub fn set_value(&mut self, index: usize, value: Value) {
        self.values[index] = value;
    }
    pub fn get_value(&self, index: usize) -> Value {
        self.values[index].clone()
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::blang::value::{value_array::ValueArray, Value};

    #[test]
    fn test_value_array() {
        let mut array = ValueArray::new();

        array.add_value(Value::Number(1.0));
        array.add_value(Value::Number(2.0));
        array.add_value(Value::Number(3.0));
        assert_eq!(array.get_value(0), Value::Number(1.0));
        assert_eq!(array.get_value(1), Value::Number(2.0));
        assert_eq!(array.get_value(2), Value::Number(3.0));
        assert_eq!(array.len(), 3);

        array.add_value(Value::Boolean(true));
        array.add_value(Value::Boolean(false));
        assert_eq!(array.get_value(3), Value::Boolean(true));
        assert_eq!(array.get_value(4), Value::Boolean(false));
        assert_eq!(array.len(), 5);

        array.add_value(Value::Nil);
        assert_eq!(array.get_value(5), Value::Nil);
        assert_eq!(array.len(), 6);
    }
}
