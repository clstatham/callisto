#[derive(Debug, Clone, PartialEq)]
pub enum Syntax {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Symbol(String),
    Operator(String),
    List(Vec<Syntax>),
}
