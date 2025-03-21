use std::{num::ParseIntError, ops::Range, str::FromStr};

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
    #[error("Invalid chord quality: {0}")]
    InvalidChordQuality(String),
    #[error("Invalid chord extension: {0}")]
    InvalidChordExtension(String),
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
            Token::Tempo => {
                token_stream.bump()?;
                token_stream.expect(Token::Whitespace)?;
                let tempo = parse_tempo(token_stream)?;
                sequence.tempo = Some(Tempo { tempo });
            }
            Token::Time => {
                token_stream.bump()?;
                token_stream.expect(Token::Whitespace)?;
                let time_signature = parse_time_signature(token_stream)?;
                sequence.time_signature = Some(time_signature);
            }
            Token::Backslash => {
                token_stream.bump()?;
                let chord = parse_named_chord(token_stream)?;
                sequence.notes.push(SeqEvent::NamedChord(chord));
            }
            Token::OBracket => {
                token_stream.bump()?;
                let chord = parse_list_chord(token_stream)?;
                sequence.notes.push(SeqEvent::ListChord(chord));
            }
            Token::NoteName => {
                let note = parse_single_note(token_stream)?;
                sequence.notes.push(SeqEvent::Single(note));
            }
            Token::Rest => {
                token_stream.bump()?;
                let note_length = parse_note_length(token_stream)?;
                sequence.notes.push(SeqEvent::Rest(note_length));
            }
            _ => {
                return Err(ParsingError::UnexpectedToken(token.token).spanned_from_token(token));
            }
        }
    }

    Ok(sequence)
}

fn parse_tempo(token_stream: &mut TokenStream) -> ParseResult<u32> {
    let token = token_stream.expect(Token::Number)?;
    let tempo = token
        .slice()
        .parse::<u32>()
        .map_err(|e| ParsingError::ParseIntError(e).spanned_from_token(&token))?;
    Ok(tempo)
}

fn parse_time_signature(token_stream: &mut TokenStream) -> ParseResult<TimeSignature> {
    let token = token_stream.expect(Token::Number)?;
    let numerator = token
        .slice()
        .parse::<u8>()
        .map_err(|e| ParsingError::ParseIntError(e).spanned_from_token(&token))?;
    token_stream.skip_whitespace();
    let token = token_stream.expect(Token::Number)?;
    let denominator = token
        .slice()
        .parse::<u8>()
        .map_err(|e| ParsingError::ParseIntError(e).spanned_from_token(&token))?;
    Ok(TimeSignature::new(numerator, denominator))
}

fn parse_note_name(token_stream: &mut TokenStream) -> ParseResult<NoteName> {
    let token = token_stream.expect(Token::NoteName)?;
    NoteName::from_str(token.slice()).map_err(|_| {
        ParsingError::InvalidNoteName(token.slice().to_string()).spanned_from_token(&token)
    })
}

fn parse_accidental(token_stream: &mut TokenStream) -> ParseResult<Accidental> {
    let token = token_stream.peek()?;
    let accidental = match token.token {
        Token::Sharp => {
            token_stream.bump()?;
            Accidental::Sharp
        }
        Token::Flat => {
            token_stream.bump()?;
            Accidental::Flat
        }
        _ => Accidental::Natural,
    };
    Ok(accidental)
}

fn parse_octave_number(token_stream: &mut TokenStream) -> ParseResult<i32> {
    let token = token_stream.expect(Token::Number)?;
    match token.slice().parse::<i32>() {
        Ok(octave) => Ok(octave),
        Err(e) => Err(ParsingError::ParseIntError(e).spanned_from_token(&token)),
    }
}

fn parse_note_length(token_stream: &mut TokenStream) -> ParseResult<NoteLength> {
    let token = token_stream.peek()?;
    match token.token {
        Token::Colon => {
            token_stream.bump()?;
            let token = token_stream.expect(Token::Number)?;
            Ok(NoteLength::from_str(token.slice())
                .map_err(|_| ParsingError::InvalidNoteLength(0).spanned_from_token(&token))?)
        }
        Token::Bar => {
            token_stream.bump()?;
            let token = token_stream.expect(Token::Number)?;
            let length = token
                .slice()
                .parse::<u32>()
                .map_err(|_| ParsingError::InvalidNoteLength(0).spanned_from_token(&token))?;
            Ok(NoteLength::Bars(length))
        }
        _ => Err(ParsingError::UnexpectedToken(token.token).spanned_from_token(token)),
    }
}

fn parse_single_note(token_stream: &mut TokenStream) -> ParseResult<SingleNote> {
    let note_name = parse_note_name(token_stream)?;
    let accidental = parse_accidental(token_stream)?;
    let octave_number = parse_octave_number(token_stream)?;
    let note_length = parse_note_length(token_stream)?;

    Ok(SingleNote {
        note_name,
        octave_number,
        note_length,
        accidental,
    })
}

fn parse_list_chord(token_stream: &mut TokenStream) -> ParseResult<ListChord> {
    let mut notes = Vec::new();
    loop {
        let token = token_stream.peek()?;
        match token.token {
            Token::Whitespace => {
                token_stream.bump()?;
            }
            Token::CBracket => {
                token_stream.bump()?;
                break;
            }
            _ => {
                let note = parse_note_name(token_stream)?;
                let accidental = parse_accidental(token_stream)?;
                let octave_number = parse_octave_number(token_stream)?;
                notes.push(ChordNote {
                    note_name: note,
                    octave_number,
                    accidental,
                });
            }
        }
    }

    let note_length = parse_note_length(token_stream)?;

    Ok(ListChord { notes, note_length })
}

fn parse_chord_quality(token_stream: &mut TokenStream) -> ParseResult<ChordQuality> {
    let token = token_stream.expect(Token::ChordQuality)?;
    ChordQuality::from_str(token.slice()).map_err(|_| {
        ParsingError::InvalidChordQuality(token.slice().to_string()).spanned_from_token(&token)
    })
}

fn parse_chord_extensions(token_stream: &mut TokenStream) -> ParseResult<Vec<ChordExtension>> {
    let mut extensions = Vec::new();
    loop {
        let token = token_stream.peek()?;
        match token.token {
            Token::Number => {
                if token.slice() == "7" {
                    token_stream.bump()?;
                    extensions.push(ChordExtension::Seventh);
                } else {
                    return Err(
                        ParsingError::InvalidChordExtension(token.slice().to_string())
                            .spanned_from_token(token),
                    );
                }
            }
            Token::ChordExtension => {
                let extension = ChordExtension::from_str(token.slice()).map_err(|_| {
                    ParsingError::InvalidChordExtension(token.slice().to_string())
                        .spanned_from_token(token)
                })?;
                token_stream.bump()?;
                extensions.push(extension);
            }
            _ => {
                break;
            }
        }
    }
    Ok(extensions)
}

fn parse_chord_name(token_stream: &mut TokenStream) -> ParseResult<ChordName> {
    let root = parse_note_name(token_stream)?;
    let root_accidental = parse_accidental(token_stream)?;
    let root_octave_number = parse_octave_number(token_stream)?;
    let quality = parse_chord_quality(token_stream)?;
    let extensions = parse_chord_extensions(token_stream)?;

    Ok(ChordName {
        root,
        root_octave_number,
        root_accidental,
        quality,
        extensions,
    })
}

fn parse_named_chord(token_stream: &mut TokenStream) -> ParseResult<NamedChord> {
    let chord_name = parse_chord_name(token_stream)?;
    let note_length = parse_note_length(token_stream)?;

    Ok(NamedChord {
        chord_name,
        note_length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_one_bar() {
        let ast = parse("{ Bb4:4 D4:8 D4:8 E4:2 }");
        if let Err(e) = ast {
            panic!("{}", e);
        }
        // let ast = ast.unwrap();
        // dbg!(&ast);
    }

    #[test]
    fn test_parse_chord() {
        let ast = parse("{ [Bb4 Eb4]:2 }");
        if let Err(e) = ast {
            panic!("{}", e);
        }
        // let ast = ast.unwrap();
        // dbg!(&ast);
    }

    #[test]
    fn test_parse_bar_note_length() {
        let ast = parse("{ Bb4|1 D4|2 D4|1 E4|4 }");
        if let Err(e) = ast {
            panic!("{}", e);
        }
        // let ast = ast.unwrap();
        // dbg!(&ast);
    }

    #[test]
    fn test_parse_named_chord() {
        let ast = parse(r"{ \E4min7:4 \C4majadd9:8 }");
        if let Err(e) = ast {
            panic!("{}", e);
        }
        // let ast = ast.unwrap();
        // dbg!(&ast);
    }
}
