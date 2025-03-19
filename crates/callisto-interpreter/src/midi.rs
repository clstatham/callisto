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

pub fn midi_note(name: NoteName, octave: i32) -> u7 {
    let note = match name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
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

    for &Note {
        note_name,
        octave_number,
        note_length,
    } in notes.notes.iter()
    {
        let first_message = MidiMessage::NoteOn {
            key: midi_note(note_name, octave_number),
            vel: u7::new(100),
        };
        let second_message = MidiMessage::NoteOff {
            key: midi_note(note_name, octave_number),
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

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    midi.tracks.push(track);

    Ok(midi)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_midi() {
        let input = r#"
            { c4:4 d4:8 d4:8 e4:2 }
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
