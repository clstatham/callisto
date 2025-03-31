use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Symbol(String),
    Number(f64),
    Boolean(bool),
    LeftParen,
    RightParen,
    Operator(String),
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
pub enum TokenKind {
    #[regex(r";;[^\n]*", logos::skip)]
    Comment,
    #[regex(r"\(")]
    LeftParen,
    #[regex(r"\)")]
    RightParen,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,
    #[regex(r":[a-zA-Z_][a-zA-Z0-9_]*")]
    Symbol,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    #[regex(r"-?\d+(\.\d+)?")]
    Number,
    #[regex(r"true|false")]
    Boolean,
    #[regex(r"[+\-*/=<>!&|]+")]
    Operator,
}

impl TokenKind {
    pub fn to_token(self, lexeme: &str) -> Token {
        match self {
            TokenKind::Comment => unreachable!(),
            TokenKind::LeftParen => Token::LeftParen,
            TokenKind::RightParen => Token::RightParen,
            TokenKind::Identifier => Token::Identifier(lexeme.to_string()),
            TokenKind::Symbol => Token::Symbol(lexeme[1..].to_string()),
            TokenKind::StringLiteral => Token::Symbol(lexeme[1..lexeme.len() - 1].to_string()),
            TokenKind::Number => Token::Number(lexeme.parse().unwrap()),
            TokenKind::Boolean => Token::Boolean(lexeme == "true"),
            TokenKind::Operator => Token::Operator(lexeme.to_string()),
        }
    }
}
