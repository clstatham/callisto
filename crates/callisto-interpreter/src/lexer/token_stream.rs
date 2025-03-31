use super::{LexingError, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    pub(crate) tokens: Vec<Result<Token, LexingError>>,
    current: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Result<Token, LexingError>>) -> Self {
        TokenStream { tokens, current: 0 }
    }

    pub fn bump(&mut self) -> Result<Token, LexingError> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            token
        } else {
            Err(LexingError::EndOfInput)
        }
    }

    pub fn peek(&self) -> Option<&Result<Token, LexingError>> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

impl Iterator for TokenStream {
    type Item = Result<Token, LexingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.bump())
        }
    }
}
