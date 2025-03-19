use std::{num::ParseIntError, ops::Range};

use logos::Logos;
use thiserror::Error;

use crate::syntax::*;

fn line_column(input: &str, index: usize) -> Option<(u32, u32)> {
    let mut line = 1;
    let mut column = 1;

    for (i, c) in input.chars().enumerate() {
        if i == index {
            return Some((line, column));
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    if index == input.len() {
        return Some((line, column));
    }

    None
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Lexing error")]
    LexingError,
    #[error("Unexpected end of input")]
    Eoi,
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("Invalid note name: {0} (expected one of `abcdefg`)")]
    InvalidNoteName(String),
    #[error("Invalid note length: {0}")]
    InvalidNoteLength(i32),
    #[error("Expected integer")]
    ParseIntError(#[from] ParseIntError),
}

impl ParsingError {
    pub fn spanned(self, input: &str, span: Range<usize>) -> SpannedParsingError {
        let (line, column) = line_column(input, span.start).unwrap();
        SpannedParsingError {
            input: input.to_string(),
            span,
            line,
            column,
            error: self,
        }
    }

    pub fn spanned_from_token(self, token: &SpannedToken) -> SpannedParsingError {
        let (line, column) = line_column(token.input, token.span.start).unwrap();
        SpannedParsingError {
            input: token.input.to_string(),
            span: token.span.clone(),
            line,
            column,
            error: self,
        }
    }
}

#[derive(Debug, Error)]
#[error("Parsing error at {}:{} (`{}`): {}", self.line, self.column, self.slice(), self.error)]
pub struct SpannedParsingError {
    input: String,
    span: Range<usize>,
    line: u32,
    column: u32,
    error: ParsingError,
}

impl SpannedParsingError {
    pub fn slice(&self) -> &str {
        &self.input[self.span.clone()]
    }

    pub fn unexpected_token(token: &SpannedToken) -> Self {
        let (line, column) = token.start_line_column();
        Self {
            input: token.input.to_string(),
            span: token.span.clone(),
            line,
            column,
            error: ParsingError::UnexpectedToken(token.token),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken<'i> {
    input: &'i str,
    span: Range<usize>,
    token: Token,
}

impl<'i> SpannedToken<'i> {
    pub fn slice(&self) -> &'i str {
        &self.input[self.span.clone()]
    }

    pub fn start_line_column(&self) -> (u32, u32) {
        line_column(self.input, self.span.start).unwrap()
    }

    pub fn end_line_column(&self) -> (u32, u32) {
        line_column(self.input, self.span.end).unwrap()
    }
}

pub struct TokenStream<'t, 'i> {
    tokens: &'t [SpannedToken<'i>],
    current: usize,
    checkpoints: Vec<usize>,
}

impl<'t, 'i> TokenStream<'t, 'i> {
    pub fn new(tokens: &'t [SpannedToken<'i>]) -> Self {
        Self {
            tokens,
            current: 0,
            checkpoints: Vec::new(),
        }
    }

    /// Returns the current token and advances the stream.
    pub fn bump(&mut self) -> ParseResult<SpannedToken<'i>> {
        if self.current >= self.tokens.len() {
            return Err(
                ParsingError::Eoi.spanned(self.tokens[0].input, self.tokens[0].span.clone())
            );
        }
        let token = &self.tokens[self.current];
        self.current += 1;
        Ok(token.clone())
    }

    /// Returns the current token without advancing the stream.
    pub fn peek(&self) -> ParseResult<&SpannedToken<'i>> {
        self.tokens.get(self.current).ok_or_else(|| {
            ParsingError::Eoi.spanned(self.tokens[0].input, self.tokens[0].span.clone())
        })
    }

    /// Bumps the current token and expects it to be of the given type.
    /// If the token is not of the expected type, it backtracks and returns an error.
    pub fn expect(&mut self, expected: Token) -> ParseResult<SpannedToken<'i>> {
        let token = self.bump()?;
        if token.token == expected {
            Ok(token)
        } else {
            self.current -= 1; // backtrack
            Err(ParsingError::UnexpectedToken(token.token).spanned_from_token(&token))
        }
    }

    /// Advances the stream if the current token is a whitespace token, and discards it.
    pub fn skip_whitespace(&mut self) {
        if let Ok(token) = self.peek() {
            if token.token == Token::Whitespace {
                self.bump().unwrap();
            }
        }
    }

    /// Saves the current position in the stream and returns the current checkpoint depth.
    pub fn push_checkpoint(&mut self) -> usize {
        self.checkpoints.push(self.current);
        self.checkpoints.len()
    }

    /// Resets the current position to the last checkpoint and returns the current checkpoint depth.
    pub fn pop_checkpoint(&mut self) -> usize {
        if let Some(checkpoint) = self.checkpoints.pop() {
            self.current = checkpoint;
        }
        self.checkpoints.len()
    }

    /// Resets the current position to the given checkpoint.
    pub fn reset_to_checkpoint(&mut self, checkpoint: usize) {
        if checkpoint < self.checkpoints.len() {
            self.current = self.checkpoints[checkpoint];
        }
    }

    /// Resets the current position to the given checkpoint and removes all checkpoints after it.
    /// Returns the current checkpoint depth.
    pub fn pop_to_checkpoint(&mut self, checkpoint: usize) -> usize {
        if checkpoint < self.checkpoints.len() {
            self.checkpoints.truncate(checkpoint);
            self.current = self.checkpoints[checkpoint];
        }
        self.checkpoints.len()
    }

    /// Returns the current checkpoint depth.
    pub fn checkpoint_depth(&self) -> usize {
        self.checkpoints.len()
    }

    /// Returns the current position in the stream.
    pub fn current_position(&self) -> usize {
        self.current
    }

    pub fn slice(&self, range: Range<usize>) -> &[SpannedToken<'i>] {
        &self.tokens[range]
    }

    pub fn remaining(&self) -> &[SpannedToken<'i>] {
        &self.tokens[self.current..]
    }

    pub fn is_eoi(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

pub type ParseResult<T> = Result<T, SpannedParsingError>;

pub fn parse(input: &str) -> ParseResult<Root> {
    let lexer = Token::lexer(input).spanned();
    let mut tokens = Vec::new();
    for (token, span) in lexer {
        match token {
            Ok(token) => tokens.push(SpannedToken { input, span, token }),
            Err(_) => return Err(ParsingError::LexingError.spanned(input, span)),
        }
    }

    let mut token_stream = TokenStream::new(&tokens);

    let mut ast = Root::default();

    while !token_stream.is_eoi() {
        let token = token_stream.peek()?;
        match token.token {
            Token::Whitespace => {
                token_stream.bump()?;
            }
            Token::OBrace => {
                token_stream.bump()?;
                let sequence = parse_sequence(&mut token_stream)?;
                ast.statements.push(Statement::Sequence(sequence));
            }
            _ => {
                return Err(ParsingError::UnexpectedToken(token.token).spanned_from_token(token));
            }
        }
    }

    Ok(ast)
}

fn parse_sequence(token_stream: &mut TokenStream) -> ParseResult<Sequence> {
    let mut sequence = Sequence::default();

    loop {
        let token = token_stream.peek()?;
        match token.token {
            Token::Whitespace => {
                token_stream.bump()?;
            }
            Token::CBrace => {
                token_stream.bump()?;
                break;
            }
            Token::Backslash => {
                token_stream.bump()?;
                // let chord = parse_named_chord(token_stream)?;
                // sequence.notes.push(SeqEvent::Chord(chord));
            }
            _ => {
                let note = parse_single_note(token_stream)?;
                sequence.notes.push(SeqEvent::Single(note));
            }
        }
    }

    Ok(sequence)
}

fn parse_single_note(token_stream: &mut TokenStream) -> ParseResult<SingleNote> {
    let mut accidental = Accidental::Natural;

    let token = token_stream.expect(Token::NoteName)?;
    let note_name = match token.slice() {
        "A" => NoteName::A,
        "B" => NoteName::B,
        "C" => NoteName::C,
        "D" => NoteName::D,
        "E" => NoteName::E,
        "F" => NoteName::F,
        "G" => NoteName::G,
        _ => {
            return Err(
                ParsingError::InvalidNoteName(token.slice().to_string()).spanned_from_token(&token)
            );
        }
    };

    let token = token_stream.peek()?;
    if token.token == Token::Sharp || token.token == Token::Flat || token.token == Token::Natural {
        accidental = match token.token {
            Token::Sharp => Accidental::Sharp,
            Token::Flat => Accidental::Flat,
            Token::Natural => Accidental::Natural,
            _ => unreachable!(),
        };
        token_stream.bump()?;
    }

    let token = token_stream.expect(Token::Number)?;
    let octave_number = match token.slice().parse::<i32>() {
        Ok(octave) => octave,
        Err(e) => {
            return Err(ParsingError::ParseIntError(e).spanned_from_token(&token));
        }
    };

    token_stream.expect(Token::Colon)?;

    let token = token_stream.expect(Token::Number)?;
    let note_length = match token.slice().parse::<i32>() {
        Ok(length) => {
            if length <= 0 {
                return Err(ParsingError::InvalidNoteLength(length).spanned_from_token(&token));
            }
            length
        }
        Err(e) => {
            return Err(ParsingError::ParseIntError(e).spanned_from_token(&token));
        }
    };

    let note_length = match note_length {
        1 => NoteLength::Whole,
        2 => NoteLength::Half,
        4 => NoteLength::Quarter,
        8 => NoteLength::Eighth,
        16 => NoteLength::Sixteenth,
        32 => NoteLength::ThirtySecond,
        64 => NoteLength::SixtyFourth,
        _ => {
            return Err(ParsingError::InvalidNoteLength(note_length).spanned_from_token(&token));
        }
    };

    Ok(SingleNote {
        note_name,
        octave_number,
        note_length,
        accidental,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    impl SpannedParsingError {}

    #[test]
    fn test_parse_one_bar() {
        let ast = parse("{ Bb4:4 D4:8 D4:8 E4:2 }");
        if let Err(e) = ast {
            panic!("{}", e);
        }
        // let ast = ast.unwrap();
        // dbg!(&ast);
    }
}
