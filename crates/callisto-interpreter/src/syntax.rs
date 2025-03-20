use std::str::FromStr;

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum Token {
    #[regex(r"[ \t\n\f]+")]
    Whitespace,
    #[regex(r"[0-9]+")]
    Number,
    #[token(r"#")]
    Sharp,
    #[regex(r"[A-Ga-g]", priority = 3)]
    NoteName,
    #[token(r"b", priority = 4)]
    Flat,
    #[token(r":")]
    Colon,
    #[token(r"|")]
    Bar,
    #[token(r"/")]
    Slash,
    #[token(r"\")]
    Backslash,
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
    pub notes: Vec<SeqEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeqEvent {
    Single(SingleNote),
    Chord(Chord),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chord {
    pub notes: Vec<ChordNote>,
    pub note_length: NoteLength,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChordNote {
    pub note_name: NoteName,
    pub octave_number: i32,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingleNote {
    pub note_name: NoteName,
    pub octave_number: i32,
    pub note_length: NoteLength,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Accidental {
    Sharp,
    Flat,
    Natural,
}

impl Accidental {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "#" => Some(Accidental::Sharp),
            "b" => Some(Accidental::Flat),
            "n" => Some(Accidental::Natural),
            _ => None,
        }
    }
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

impl FromStr for NoteName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(NoteName::A),
            "B" => Ok(NoteName::B),
            "C" => Ok(NoteName::C),
            "D" => Ok(NoteName::D),
            "E" => Ok(NoteName::E),
            "F" => Ok(NoteName::F),
            "G" => Ok(NoteName::G),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoteLength {
    SixtyFourth,
    ThirtySecond,
    Sixteenth,
    Eighth,
    Quarter,
    Half,
    #[default]
    Whole,
    Bars(u32),
}

impl FromStr for NoteLength {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "64" => Ok(NoteLength::SixtyFourth),
            "32" => Ok(NoteLength::ThirtySecond),
            "16" => Ok(NoteLength::Sixteenth),
            "8" => Ok(NoteLength::Eighth),
            "4" => Ok(NoteLength::Quarter),
            "2" => Ok(NoteLength::Half),
            "1" => Ok(NoteLength::Whole),
            _ => Err(()),
        }
    }
}
