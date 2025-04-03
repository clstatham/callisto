use raug::prelude::*;

use super::RuntimeError;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Number,
    Symbol,
    String,
    Boolean,
    List,
    Null,
    Node,
    Input,
    Output,
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
            Value::Node(_) => ValueType::Node,
            Value::Input(_) => ValueType::Input,
            Value::Output(_) => ValueType::Output,
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
    Node(Node),
    Input(Input),
    Output(Output),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        ValueType::from_value(self)
    }

    pub fn make_node(&self, graph: &GraphBuilder) -> Result<Node, RuntimeError> {
        match self {
            Value::Node(node) => Ok(node.clone()),
            Value::Output(output) => Ok(output.clone().make_node()),
            Value::Number(val) => Ok(graph.constant(*val)),
            Value::Boolean(val) => Ok(graph.constant(*val)),
            _ => Err(RuntimeError::CannotConvertToNode(self.value_type())),
        }
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
            (Value::Node(a), Value::Node(b)) => Ok(Value::Output((a + b).output(0))),
            (Value::Output(a), Value::Output(b)) => Ok(Value::Output((a + b).output(0))),
            (Value::Output(a), Value::Node(b)) => Ok(Value::Output((a + b).output(0))),
            (Value::Node(a), Value::Output(b)) => {
                a.assert_single_output("+");
                Ok(Value::Output((a.output(0) + b).output(0)))
            }
            (Value::Node(a), b) => {
                let b = b.make_node(a.graph())?;
                a.assert_single_output("+");
                Ok(Value::Output((a.output(0) + b).output(0)))
            }
            (a, Value::Node(b)) => {
                let a = a.make_node(b.graph())?;
                Ok(Value::Output((a + b).output(0)))
            }
            (Value::Output(a), b) => {
                let b = b.make_node(a.node().graph())?;
                Ok(Value::Output((a + b).output(0)))
            }
            (a, Value::Output(b)) => {
                let a = a.make_node(b.node().graph())?;
                a.assert_single_output("+");
                Ok(Value::Output((a.output(0) + b).output(0)))
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
            (Value::Node(a), Value::Node(b)) => Ok(Value::Output((a - b).output(0))),
            (Value::Output(a), Value::Output(b)) => Ok(Value::Output((a - b).output(0))),
            (Value::Output(a), Value::Node(b)) => Ok(Value::Output((a - b).output(0))),
            (Value::Node(a), Value::Output(b)) => {
                a.assert_single_output("-");
                Ok(Value::Output((a.output(0) - b).output(0)))
            }
            (Value::Node(a), b) => {
                let b = b.make_node(a.graph())?;
                a.assert_single_output("-");
                Ok(Value::Output((a.output(0) - b).output(0)))
            }
            (a, Value::Node(b)) => {
                let a = a.make_node(b.graph())?;
                Ok(Value::Output((a - b).output(0)))
            }
            (Value::Output(a), b) => {
                let b = b.make_node(a.node().graph())?;
                Ok(Value::Output((a - b).output(0)))
            }
            (a, Value::Output(b)) => {
                let a = a.make_node(b.node().graph())?;
                a.assert_single_output("-");
                Ok(Value::Output((a.output(0) - b).output(0)))
            }
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
            (Value::Node(a), Value::Node(b)) => Ok(Value::Output((a * b).output(0))),
            (Value::Output(a), Value::Output(b)) => Ok(Value::Output((a * b).output(0))),
            (Value::Output(a), Value::Node(b)) => Ok(Value::Output((a * b).output(0))),
            (Value::Node(a), Value::Output(b)) => {
                a.assert_single_output("*");
                Ok(Value::Output((a.output(0) * b).output(0)))
            }
            (Value::Node(a), b) => {
                let b = b.make_node(a.graph())?;
                a.assert_single_output("*");
                Ok(Value::Output((a.output(0) * b).output(0)))
            }
            (a, Value::Node(b)) => {
                let a = a.make_node(b.graph())?;
                Ok(Value::Output((a * b).output(0)))
            }
            (Value::Output(a), b) => {
                let b = b.make_node(a.node().graph())?;
                Ok(Value::Output((a * b).output(0)))
            }
            (a, Value::Output(b)) => {
                let a = a.make_node(b.node().graph())?;
                a.assert_single_output("*");
                Ok(Value::Output((a.output(0) * b).output(0)))
            }
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
            (Value::Node(a), Value::Node(b)) => Ok(Value::Output((a / b).output(0))),
            (Value::Output(a), Value::Output(b)) => Ok(Value::Output((a / b).output(0))),
            (Value::Output(a), Value::Node(b)) => Ok(Value::Output((a / b).output(0))),
            (Value::Node(a), Value::Output(b)) => {
                a.assert_single_output("/");
                Ok(Value::Output((a.output(0) / b).output(0)))
            }
            (Value::Node(a), b) => {
                let b = b.make_node(a.graph())?;
                a.assert_single_output("/");
                Ok(Value::Output((a.output(0) / b).output(0)))
            }
            (a, Value::Node(b)) => {
                let a = a.make_node(b.graph())?;
                Ok(Value::Output((a / b).output(0)))
            }
            (Value::Output(a), b) => {
                let b = b.make_node(a.node().graph())?;
                Ok(Value::Output((a / b).output(0)))
            }
            (a, Value::Output(b)) => {
                let a = a.make_node(b.node().graph())?;
                a.assert_single_output("/");
                Ok(Value::Output((a.output(0) / b).output(0)))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "/".to_string(),
                left: self.value_type(),
                right: other.value_type(),
            }),
        }
    }
}
