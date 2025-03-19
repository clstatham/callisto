use std::error::Error;

use midly::{Format, Header, MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind, num::*};

use crate::syntax::*;

const PPQ: u32 = 96;

pub trait ToMidi {
    fn to_midi_track_events(&self) -> Vec<TrackEvent>;
}

pub fn midi_note_length(note_length: NoteLength) -> u28 {
    let length = match note_length {
        NoteLength::SixtyFourth => PPQ / 16,
        NoteLength::ThirtySecond => PPQ / 8,
        NoteLength::Sixteenth => PPQ / 4,
        NoteLength::Eighth => PPQ / 2,
        NoteLength::Quarter => PPQ,
        NoteLength::Half => PPQ * 2,
        NoteLength::Whole => PPQ * 4,
    };
    u28::new(length)
}

pub fn midi_note(name: NoteName, accidental: Accidental, octave: i32) -> u7 {
    let note = match name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    };
    let note = match accidental {
        Accidental::Sharp => note + 1,
        Accidental::Flat => note - 1,
        Accidental::Natural => note,
    };
    let octave = octave + 2;
    let note = note + (octave * 12);
    if !(0..=127).contains(&note) {
        panic!("Note out of range");
    } else {
        u7::new(note as u8)
    }
}

pub fn ast_to_midi(ast: &Root) -> Result<Smf, Box<dyn Error>> {
    let seq = &ast.statements[0];

    let Statement::Sequence(notes) = seq;

    let mut midi = Smf::new(Header::new(
        Format::SingleTrack,
        midly::Timing::Metrical(u15::new(PPQ as u16)),
    ));

    let mut track = Vec::new();

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(500_000))),
    });

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 36, 8)),
    });

    for event in notes.notes.iter() {
        match event {
            SeqEvent::Single(note) => {
                let note_name = note.note_name;
                let octave_number = note.octave_number;
                let note_length = note.note_length;
                let accidental = note.accidental;

                add_note_to_track(
                    &mut track,
                    note_name,
                    accidental,
                    octave_number,
                    note_length,
                );
            }
            SeqEvent::Chord(chord) => {
                todo!("Handle chord");
            }
        }
    }

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    midi.tracks.push(track);

    Ok(midi)
}

fn add_note_to_track(
    track: &mut Vec<TrackEvent>,
    note_name: NoteName,
    accidental: Accidental,
    octave_number: i32,
    note_length: NoteLength,
) {
    let first_message = MidiMessage::NoteOn {
        key: midi_note(note_name, accidental, octave_number),
        vel: u7::new(100),
    };
    let second_message = MidiMessage::NoteOff {
        key: midi_note(note_name, accidental, octave_number),
        vel: u7::new(0),
    };

    let first_event = TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Midi {
            channel: u4::new(0),
            message: first_message,
        },
    };

    let second_event = TrackEvent {
        delta: midi_note_length(note_length),
        kind: TrackEventKind::Midi {
            channel: u4::new(0),
            message: second_message,
        },
    };

    track.push(first_event);
    track.push(second_event);
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_midi() {
        let input = r#"
            { C#4:2 Eb4:2 }
        "#;

        // let midi = Smf::parse(include_bytes!("../one_bar_ref.mid")).unwrap();
        // dbg!(midi);

        let ast = crate::parser::parse(input);
        let ast = match ast {
            Ok(ast) => ast,
            Err(e) => panic!("Error parsing: {}", e),
        };
        let midi = ast_to_midi(&ast).unwrap();

        midi.write_std(&mut File::create("test.mid").unwrap())
            .unwrap();
    }
}
