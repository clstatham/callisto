use logos::Logos;
use thiserror::Error;
use token::TokenKind;
use token_stream::TokenStream;

pub mod token;
pub mod token_stream;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum LexingError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Unexpected end of input")]
    EndOfInput,
}

pub fn tokenize(input: &str) -> TokenStream {
    let mut tokens = Vec::new();
    let lexer = TokenKind::lexer(input).spanned();

    for (kind, span) in lexer {
        let lexeme = &input[span];
        match kind {
            Ok(token) => tokens.push(Ok(token.to_token(lexeme))),
            Err(_) => tokens.push(Err(LexingError::InvalidToken(lexeme.to_string()))),
        }
    }

    TokenStream::new(tokens)
}

#[cfg(test)]
mod tests {
    use super::{token::Token, *};

    #[test]
    fn test_tokenize_comment() {
        let input = r#";; this is a comment"#;
        let expected_tokens: Vec<Result<Token, LexingError>> = vec![];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_number() {
        let input = "42";
        let expected_tokens = vec![Ok(Token::Number(42.0))];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_identifier() {
        let input = "x";
        let expected_tokens = vec![Ok(Token::Identifier("x".to_string()))];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_boolean() {
        let input = "true false";
        let expected_tokens = vec![Ok(Token::Boolean(true)), Ok(Token::Boolean(false))];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_symbol() {
        let input = ":symbol";
        let expected_tokens = vec![Ok(Token::Symbol("symbol".to_string()))];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_string() {
        let input = r#""hello world""#;
        let expected_tokens = vec![Ok(Token::Symbol("hello world".to_string()))];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_operator() {
        let input = "+ - * /";
        let expected_tokens = vec![
            Ok(Token::Operator("+".to_string())),
            Ok(Token::Operator("-".to_string())),
            Ok(Token::Operator("*".to_string())),
            Ok(Token::Operator("/".to_string())),
        ];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize_parentheses() {
        let input = "( )";
        let expected_tokens = vec![Ok(Token::LeftParen), Ok(Token::RightParen)];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }

    #[test]
    fn test_tokenize() {
        let input = "(define x 42)";
        let expected_tokens = vec![
            Ok(Token::LeftParen),
            Ok(Token::Identifier("define".to_string())),
            Ok(Token::Identifier("x".to_string())),
            Ok(Token::Number(42.0)),
            Ok(Token::RightParen),
        ];
        let token_stream = tokenize(input);
        assert_eq!(token_stream.tokens, expected_tokens);
    }
}
