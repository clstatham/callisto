use std::error::Error;

use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind, num::*,
};

use crate::syntax::*;

const TICKS_PER_BEAT: u32 = 96;

const MAJOR_SCALE: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
const MINOR_SCALE: [u8; 7] = [0, 2, 3, 5, 7, 8, 10];
const DIMINISHED_SCALE: [u8; 7] = [0, 2, 3, 5, 6, 8, 10];
const AUGMENTED_SCALE: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

pub fn midi_named_chord(chord: &ChordName) -> Vec<u8> {
    let ChordName {
        root,
        root_accidental,
        root_octave_number,
        quality,
        extensions,
    } = chord;
    let root = midi_note(*root, *root_accidental, *root_octave_number);
    let root = root.as_int();

    let scale = match quality {
        ChordQuality::Major => MAJOR_SCALE,
        ChordQuality::Minor => MINOR_SCALE,
        ChordQuality::Diminished => DIMINISHED_SCALE,
        ChordQuality::Augmented => AUGMENTED_SCALE,
    };

    let mut chord = vec![root, root + scale[2], root + scale[4]]; // root, third, fifth
    if extensions.contains(&ChordExtension::Sixth) {
        chord.push(root + scale[5]);
    }
    if extensions.contains(&ChordExtension::Seventh) {
        chord.push(root + scale[6]);
    }
    if extensions.contains(&ChordExtension::Ninth) {
        chord.push(root + scale[1] + 12);
    }
    if extensions.contains(&ChordExtension::Eleventh) {
        chord.push(root + scale[3] + 12);
    }
    if extensions.contains(&ChordExtension::Thirteenth) {
        chord.push(root + scale[5] + 12);
    }
    chord.sort();
    chord.dedup();
    chord
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
        Timing::Metrical(u15::new(TICKS_PER_BEAT as u16)),
    ));

    let mut track = Vec::new();

    let tempo = if let Some(Tempo { tempo }) = notes.tempo {
        tempo
    } else {
        120
    };

    // convert to microseconds per beat
    let tempo = 60_000_000 / tempo;

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(tempo))),
    });

    let time_signature = if let Some(time_signature) = notes.time_signature {
        time_signature
    } else {
        TimeSignature::new(4, 4)
    };

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
            time_signature.numerator.get(),
            time_signature.denominator.get(),
            36,
            8,
        )),
    });

    let mut ticks_since_last_event = 0;

    macro_rules! advance_ticks {
        ($delta:expr) => {{
            ticks_since_last_event += $delta;
        }};
    }

    macro_rules! event_tick {
        () => {{
            let delta = u28::new(ticks_since_last_event);
            ticks_since_last_event = 0;
            delta
        }};
    }

    for event in notes.notes.iter() {
        match event {
            SeqEvent::Rest(note_length) => {
                advance_ticks!(note_length.ticks(TICKS_PER_BEAT, time_signature));
                continue;
            }
            SeqEvent::Single(note) => {
                let &SingleNote {
                    note_name,
                    octave_number,
                    note_length,
                    accidental,
                } = note;

                let key = midi_note(note_name, accidental, octave_number);

                let first_message = MidiMessage::NoteOn {
                    key,
                    vel: u7::new(127),
                };
                let second_message = MidiMessage::NoteOff {
                    key,
                    vel: u7::new(0),
                };

                let first_event = TrackEvent {
                    delta: event_tick!(),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(0),
                        message: first_message,
                    },
                };

                advance_ticks!(note_length.ticks(TICKS_PER_BEAT, time_signature));

                let second_event = TrackEvent {
                    delta: event_tick!(),
                    kind: TrackEventKind::Midi {
                        channel: u4::new(0),
                        message: second_message,
                    },
                };

                track.push(first_event);
                track.push(second_event);
            }
            SeqEvent::ListChord(chord) => {
                let ListChord { notes, note_length } = chord;
                let mut chord_events = Vec::new();
                let mut stop_events = Vec::new();

                for (i, note) in notes.iter().enumerate() {
                    let &ChordNote {
                        note_name,
                        octave_number,
                        accidental,
                    } = note;

                    let key = midi_note(note_name, accidental, octave_number);

                    let first_message = MidiMessage::NoteOn {
                        key,
                        vel: u7::new(127),
                    };
                    let second_message = MidiMessage::NoteOff {
                        key,
                        vel: u7::new(0),
                    };

                    let first_event = TrackEvent {
                        delta: if i == 0 { event_tick!() } else { u28::new(0) },
                        kind: TrackEventKind::Midi {
                            channel: u4::new(0),
                            message: first_message,
                        },
                    };

                    if i == 0 {
                        advance_ticks!(note_length.ticks(TICKS_PER_BEAT, time_signature));
                    }

                    let second_event = TrackEvent {
                        delta: if i == 0 { event_tick!() } else { u28::new(0) },
                        kind: TrackEventKind::Midi {
                            channel: u4::new(0),
                            message: second_message,
                        },
                    };

                    chord_events.push(first_event);
                    stop_events.push(second_event);
                }

                track.extend(chord_events);
                track.extend(stop_events);
            }
            SeqEvent::NamedChord(named_chord) => {
                let NamedChord {
                    chord_name,
                    note_length,
                } = named_chord;
                let chord_notes = midi_named_chord(chord_name);

                let mut chord_events = Vec::new();
                let mut stop_events = Vec::new();

                for (i, note) in chord_notes.iter().enumerate() {
                    let key = u7::new(*note);

                    let first_message = MidiMessage::NoteOn {
                        key,
                        vel: u7::new(127),
                    };
                    let second_message = MidiMessage::NoteOff {
                        key,
                        vel: u7::new(0),
                    };

                    let first_event = TrackEvent {
                        delta: if i == 0 { event_tick!() } else { u28::new(0) },
                        kind: TrackEventKind::Midi {
                            channel: u4::new(0),
                            message: first_message,
                        },
                    };

                    if i == 0 {
                        advance_ticks!(note_length.ticks(TICKS_PER_BEAT, time_signature));
                    }

                    let second_event = TrackEvent {
                        delta: if i == 0 { event_tick!() } else { u28::new(0) },
                        kind: TrackEventKind::Midi {
                            channel: u4::new(0),
                            message: second_message,
                        },
                    };

                    chord_events.push(first_event);
                    stop_events.push(second_event);
                }

                track.extend(chord_events);
                track.extend(stop_events);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi() {
        let input = r"
{ 
    tempo 120
    time 4 4
    \E4min7|1
    z|1
    \C4majadd9|1
}";

        let ast = crate::parser::parse(input);
        let ast = match ast {
            Ok(ast) => ast,
            Err(e) => panic!("Error parsing: {}", e),
        };
        let midi = ast_to_midi(&ast).unwrap();

        midi.write_std(&mut std::fs::File::create("test.mid").unwrap())
            .unwrap();
    }
}
