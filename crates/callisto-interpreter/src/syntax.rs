use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum Token {
    #[regex(r"[ \t\n\f]+")]
    Whitespace,
    #[regex(r"[a-zA-Z]+")]
    Text,
    #[regex(r"[0-9]+")]
    Number,
    #[token(r":")]
    Colon,
    #[token(r"/")]
    Slash,
    #[token(r"{")]
    OBrace,
    #[token(r"}")]
    CBrace,
    #[token(r"[")]
    OBracket,
    #[token(r"]")]
    CBracket,
    #[token(r"tempo")]
    Tempo,
    #[token(r"time")]
    Time,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Root {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statement {
    Sequence(Sequence),
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence {
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note {
    pub note_name: NoteName,
    pub octave_number: i32,
    pub note_length: NoteLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoteName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoteLength {
    SixtyFourth,
    ThirtySecond,
    Sixteenth,
    Eighth,
    Quarter,
    Half,
    Whole,
}
