use syntax::Syntax;
use thiserror::Error;

use crate::lexer::{LexingError, token::Token, token_stream::TokenStream, tokenize};

pub mod syntax;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParsingError {
    #[error("Lexing error: {0}")]
    LexingError(#[from] LexingError),

    #[error("Unexpected token: {token:?}")]
    UnexpectedToken { token: Token },
}

pub fn parse_str(input: &str) -> Result<Vec<Syntax>, ParsingError> {
    parse(&mut tokenize(input))
}

pub fn parse(input: &mut TokenStream) -> Result<Vec<Syntax>, ParsingError> {
    let mut syntax_tree = Vec::new();

    while !input.is_empty() {
        let syntax = parse_expression(input)?;
        syntax_tree.push(syntax);
    }

    Ok(syntax_tree)
}

fn parse_expression(input: &mut TokenStream) -> Result<Syntax, ParsingError> {
    let token = input.bump()?;
    match token {
        Token::LeftParen => parse_list(input),
        Token::Identifier(tok) => Ok(Syntax::Identifier(tok)),
        Token::Symbol(tok) => Ok(Syntax::Symbol(tok)),
        Token::Operator(tok) => Ok(Syntax::Operator(tok)),
        Token::Boolean(tok) => Ok(Syntax::Boolean(tok)),
        Token::Number(tok) => Ok(Syntax::Number(tok)),
        token => Err(ParsingError::UnexpectedToken { token }),
    }
}

fn parse_list(input: &mut TokenStream) -> Result<Syntax, ParsingError> {
    let mut elements = Vec::new();

    while !input.is_empty() {
        let token = input
            .peek()
            .ok_or(ParsingError::LexingError(LexingError::EndOfInput))?;
        if *token == Ok(Token::RightParen) {
            input.bump()?; // consume the right parenthesis
            break;
        }
        let element = parse_expression(input)?;
        elements.push(element);
    }

    Ok(Syntax::List(elements))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expression() {
        let input = "(define x 42)";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(
            syntax_tree[0],
            Syntax::List(vec![
                Syntax::Identifier("define".to_string()),
                Syntax::Identifier("x".to_string()),
                Syntax::Number(42.0),
            ])
        );
    }

    #[test]
    fn test_parse_list() {
        let input = "(+ 1 2)";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(
            syntax_tree[0],
            Syntax::List(vec![
                Syntax::Operator("+".to_string()),
                Syntax::Number(1.0),
                Syntax::Number(2.0),
            ])
        );
    }

    #[test]
    fn test_parse_identifier() {
        let input = "x";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(syntax_tree[0], Syntax::Identifier("x".to_string()));
    }

    #[test]
    fn test_parse_boolean() {
        let input = "true false";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 2);
        assert_eq!(syntax_tree[0], Syntax::Boolean(true));
        assert_eq!(syntax_tree[1], Syntax::Boolean(false));
    }

    #[test]
    fn test_parse_symbol() {
        let input = ":symbol";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(syntax_tree[0], Syntax::Symbol("symbol".to_string()));
    }

    #[test]
    fn test_parse_operator() {
        let input = "+ - * /";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 4);
        assert_eq!(syntax_tree[0], Syntax::Operator("+".to_string()));
        assert_eq!(syntax_tree[1], Syntax::Operator("-".to_string()));
        assert_eq!(syntax_tree[2], Syntax::Operator("*".to_string()));
        assert_eq!(syntax_tree[3], Syntax::Operator("/".to_string()));
    }

    #[test]
    fn test_parse_empty_list() {
        let input = "()";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(syntax_tree[0], Syntax::List(vec![]));
    }

    #[test]
    fn test_parse_nested_list() {
        let input = "(+ 1 (* 2 3))";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(
            syntax_tree[0],
            Syntax::List(vec![
                Syntax::Operator("+".to_string()),
                Syntax::Number(1.0),
                Syntax::List(vec![
                    Syntax::Operator("*".to_string()),
                    Syntax::Number(2.0),
                    Syntax::Number(3.0),
                ]),
            ])
        );
    }

    #[test]
    fn test_parse_function_definition() {
        let input = "(define (add a b) (+ a b))";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 1);
        assert_eq!(
            syntax_tree[0],
            Syntax::List(vec![
                Syntax::Identifier("define".to_string()),
                Syntax::List(vec![
                    Syntax::Identifier("add".to_string()),
                    Syntax::Identifier("a".to_string()),
                    Syntax::Identifier("b".to_string()),
                ]),
                Syntax::List(vec![
                    Syntax::Operator("+".to_string()),
                    Syntax::Identifier("a".to_string()),
                    Syntax::Identifier("b".to_string()),
                ]),
            ])
        );
    }

    #[test]
    fn test_parse_multiple_expressions() {
        let input = "(define x 42) (define y 43)";
        let mut token_stream = tokenize(input);
        let syntax_tree = parse(&mut token_stream).unwrap();
        assert_eq!(syntax_tree.len(), 2);
        assert_eq!(
            syntax_tree[0],
            Syntax::List(vec![
                Syntax::Identifier("define".to_string()),
                Syntax::Identifier("x".to_string()),
                Syntax::Number(42.0),
            ])
        );
        assert_eq!(
            syntax_tree[1],
            Syntax::List(vec![
                Syntax::Identifier("define".to_string()),
                Syntax::Identifier("y".to_string()),
                Syntax::Number(43.0),
            ])
        );
    }
}
