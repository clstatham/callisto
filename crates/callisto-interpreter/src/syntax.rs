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
    #[token(r"n", priority = 4)]
    Natural,
    #[token(r":")]
    Colon,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chord {
    pub notes: Vec<SingleNote>,
    pub note_length: NoteLength,
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

impl NoteName {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "A" => Some(NoteName::A),
            "B" => Some(NoteName::B),
            "C" => Some(NoteName::C),
            "D" => Some(NoteName::D),
            "E" => Some(NoteName::E),
            "F" => Some(NoteName::F),
            "G" => Some(NoteName::G),
            _ => None,
        }
    }
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

impl NoteLength {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "64" => Some(NoteLength::SixtyFourth),
            "32" => Some(NoteLength::ThirtySecond),
            "16" => Some(NoteLength::Sixteenth),
            "8" => Some(NoteLength::Eighth),
            "4" => Some(NoteLength::Quarter),
            "2" => Some(NoteLength::Half),
            "1" => Some(NoteLength::Whole),
            _ => None,
        }
    }
}
