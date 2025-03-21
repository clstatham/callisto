use std::{num::NonZeroU8, str::FromStr};

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
    #[regex(r"maj|min|dim|aug")]
    ChordQuality,
    #[regex(r"add9|add11|add13")]
    ChordExtension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tempo {
    pub tempo: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSignature {
    /// The number of beats in a measure.
    /// For example, 4 beats in 4/4 time.
    pub numerator: NonZeroU8,
    /// The type of note that gets one beat.
    /// For example, a quarter note gets one beat in 4/4 time.
    pub denominator: NonZeroU8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature {
            numerator: NonZeroU8::new(4).unwrap(),
            denominator: NonZeroU8::new(4).unwrap(),
        }
    }
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        TimeSignature {
            numerator: NonZeroU8::new(numerator).unwrap(),
            denominator: NonZeroU8::new(denominator).unwrap(),
        }
    }

    pub fn ticks_per_measure(&self, ticks_per_quarter: u32) -> u32 {
        (self.numerator.get() as u32 * ticks_per_quarter) / self.denominator.get() as u32
    }

    pub fn ticks_per_beat(&self, ticks_per_quarter: u32) -> u32 {
        ticks_per_quarter / self.denominator.get() as u32
    }
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
    pub tempo: Option<Tempo>,
    pub time_signature: Option<TimeSignature>,
    pub notes: Vec<SeqEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeqEvent {
    Single(SingleNote),
    ListChord(ListChord),
    NamedChord(NamedChord),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListChord {
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
pub struct ChordName {
    pub root: NoteName,
    pub root_accidental: Accidental,
    pub root_octave_number: i32,
    pub quality: ChordQuality,
    pub extensions: Vec<ChordExtension>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamedChord {
    pub chord_name: ChordName,
    pub note_length: NoteLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
}

impl FromStr for ChordQuality {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "maj" => Ok(ChordQuality::Major),
            "min" => Ok(ChordQuality::Minor),
            "dim" => Ok(ChordQuality::Diminished),
            "aug" => Ok(ChordQuality::Augmented),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChordExtension {
    Sixth,
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
}

impl FromStr for ChordExtension {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add6" => Ok(ChordExtension::Sixth),
            "add9" => Ok(ChordExtension::Ninth),
            "add11" => Ok(ChordExtension::Eleventh),
            "add13" => Ok(ChordExtension::Thirteenth),
            _ => Err(()),
        }
    }
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

impl NoteLength {
    pub fn ticks(self, ticks_per_beat: u32, time_signature: TimeSignature) -> u32 {
        match self {
            NoteLength::SixtyFourth => ticks_per_beat / 16,
            NoteLength::ThirtySecond => ticks_per_beat / 8,
            NoteLength::Sixteenth => ticks_per_beat / 4,
            NoteLength::Eighth => ticks_per_beat / 2,
            NoteLength::Quarter => ticks_per_beat,
            NoteLength::Half => ticks_per_beat * 2,
            NoteLength::Whole => ticks_per_beat * 4,
            NoteLength::Bars(bars) => {
                let denominator_ticks = match time_signature.denominator.get() {
                    1 => ticks_per_beat * 4,
                    2 => ticks_per_beat * 2,
                    4 => ticks_per_beat,
                    8 => ticks_per_beat / 2,
                    16 => ticks_per_beat / 4,
                    32 => ticks_per_beat / 8,
                    64 => ticks_per_beat / 16,
                    _ => panic!("Unsupported denominator"),
                };
                denominator_ticks * bars * time_signature.numerator.get() as u32
            }
        }
    }
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
