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

impl Syntax {
    pub fn syntax_type(&self) -> SyntaxType {
        match self {
            Syntax::Number(_) => SyntaxType::Number,
            Syntax::String(_) => SyntaxType::String,
            Syntax::Boolean(_) => SyntaxType::Boolean,
            Syntax::Identifier(_) => SyntaxType::Identifier,
            Syntax::Symbol(_) => SyntaxType::Symbol,
            Syntax::Operator(_) => SyntaxType::Operator,
            Syntax::List(_) => SyntaxType::List,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxType {
    Number,
    String,
    Boolean,
    Identifier,
    Symbol,
    Operator,
    List,
}

impl SyntaxType {
    pub fn from_syntax(syntax: &Syntax) -> Self {
        match syntax {
            Syntax::Number(_) => SyntaxType::Number,
            Syntax::String(_) => SyntaxType::String,
            Syntax::Boolean(_) => SyntaxType::Boolean,
            Syntax::Identifier(_) => SyntaxType::Identifier,
            Syntax::Symbol(_) => SyntaxType::Symbol,
            Syntax::Operator(_) => SyntaxType::Operator,
            Syntax::List(_) => SyntaxType::List,
        }
    }
}
