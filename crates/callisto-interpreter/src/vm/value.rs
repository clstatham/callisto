use super::RuntimeError;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Number,
    Symbol,
    String,
    Boolean,
    List,
    Null,
}

impl ValueType {
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Number(_) => ValueType::Number,
            Value::Symbol(_) => ValueType::Symbol,
            Value::String(_) => ValueType::String,
            Value::Boolean(_) => ValueType::Boolean,
            Value::List(_) => ValueType::List,
            Value::Null => ValueType::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Null,
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        ValueType::from_value(self)
    }

    pub fn add(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a.clone() + b)),
            (Value::List(a), Value::List(b)) => {
                let mut new_list = a.clone();
                new_list.extend(b.clone());
                Ok(Value::List(new_list))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "+".to_string(),
                left: self.value_type(),
                right: other.value_type(),
            }),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(RuntimeError::InvalidOperation {
                operation: "-".to_string(),
                left: self.value_type(),
                right: other.value_type(),
            }),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),

            _ => Err(RuntimeError::InvalidOperation {
                operation: "*".to_string(),
                left: self.value_type(),
                right: other.value_type(),
            }),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Value::Number(a / b))
            }

            _ => Err(RuntimeError::InvalidOperation {
                operation: "/".to_string(),
                left: self.value_type(),
                right: other.value_type(),
            }),
        }
    }
}
