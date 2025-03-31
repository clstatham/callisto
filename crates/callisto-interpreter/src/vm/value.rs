#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Null,
}
