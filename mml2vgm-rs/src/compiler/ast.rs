//! Abstract Syntax Tree for MML
//!
//! This module contains the AST structures for representing MML source code.
//! The AST is built by the parser and used by the semantic analyzer and code generator.

use crate::{MmlError, MmlResult, Position, Span};
use std::collections::HashMap;
use std::path::PathBuf;

/// A musical note in MML
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// Note letter: C, D, E, F, G, A, B (case-insensitive, stored as uppercase)
    pub letter: char,
    /// Accidental: -1 = flat, 0 = natural, 1 = sharp
    pub accidental: i8,
    /// Octave (0-8, where 4 is middle octave)
    pub octave: u8,
    /// Duration in ticks (based on current length setting)
    pub duration: Option<u32>,
    /// Whether the note is dotted
    pub dotted: bool,
    /// Whether the note is tied to the next note
    pub tied: bool,
    /// Volume override (0-127)
    pub volume: Option<u8>,
    /// Instrument reference
    pub instrument: Option<usize>,
    /// Source location span
    pub span: Option<Span>,
}

impl Note {
    pub fn new(letter: char, accidental: i8, octave: u8) -> Self {
        Self {
            letter: letter.to_ascii_uppercase(),
            accidental,
            octave,
            duration: None,
            dotted: false,
            tied: false,
            volume: None,
            instrument: None,
            span: None,
        }
    }

    /// Calculate MIDI note number (0-127)
    /// MIDI note formula: 12 * (octave + 1) + note_index
    /// where C=0, C#=1, D=2, D#=3, E=4, F=5, F#=6, G=7, G#=8, A=9, A#=10, B=11
    pub fn midi_note(&self) -> u8 {
        let note_index = match self.letter {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => 0,
        };
        let midi = 12 * (self.octave as i32 + 1) + note_index + self.accidental as i32;
        midi.clamp(0, 127) as u8
    }
}

/// A rest (silence) in MML
#[derive(Debug, Clone, PartialEq)]
pub struct Rest {
    /// Duration in ticks
    pub duration: u32,
    /// Whether the rest is dotted
    pub dotted: bool,
    /// Source location span
    pub span: Option<Span>,
}

/// Tempo setting
#[derive(Debug, Clone, PartialEq)]
pub struct Tempo {
    /// BPM value (beats per minute)
    pub bpm: u32,
}

/// Volume setting
#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    /// Volume level (0-127)
    pub level: u8,
}

/// Default length setting
#[derive(Debug, Clone, PartialEq)]
pub struct Length {
    /// Length value (1, 2, 4, 8, 16, etc.)
    pub value: u32,
}

/// Octave setting
#[derive(Debug, Clone, PartialEq)]
pub struct Octave {
    /// Octave number (0-8)
    pub number: u8,
}

/// Octave shift (relative)
#[derive(Debug, Clone, PartialEq)]
pub enum OctaveShift {
    Up,
    Down,
}

/// Instrument selection
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentSelection {
    /// Instrument number
    pub number: usize,
    /// Source location span
    pub span: Option<Span>,
}

/// Quantize / gate time
/// - q (lowercase): absolute, silence = value/48 of note duration
/// - Q (uppercase): proportional, note sounds for value/8 of duration (Q8 = full note)
#[derive(Debug, Clone, PartialEq)]
pub struct Quantize {
    pub value: u8,
    /// true = uppercase Q (GatetimeDiv, proportional), false = lowercase q (Gatetime, absolute)
    pub proportional: bool,
}

/// Loop structure
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    /// Number of times to repeat
    pub count: usize,
    /// Body of the loop
    pub body: Vec<MmlNode>,
}

/// Part definition
#[derive(Debug, Clone, PartialEq)]
pub struct PartDefinition {
    /// Part name (e.g., "A1", "Y01")
    pub name: String,
    /// Sound chip target for this part
    pub chip: Option<String>,
    /// Tempo for this part
    pub tempo: Option<u32>,
    /// Commands in this part
    pub commands: Vec<MmlNode>,
}

/// Metadata entry (song info)
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub key: String,
    pub value: String,
}

/// Include directive
#[derive(Debug, Clone, PartialEq)]
pub struct Include {
    pub path: PathBuf,
}

/// Instrument definition for FM synthesis
#[derive(Debug, Clone, PartialEq)]
pub struct FmInstrument {
    pub number: u32,
    pub name: String,
    pub parameters: Vec<u32>,
}

/// Instrument definition for PCM
#[derive(Debug, Clone, PartialEq)]
pub struct PcmInstrument {
    pub number: u32,
    pub name: String,
    pub filename: PathBuf,
    pub frequency: u32,
    pub volume: u8,
    pub chip: String,
    pub option: Option<u32>,
}

/// OPX (YMF271) operator mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpxMode {
    /// 4-operator FM (4 operator rows)
    X4,
    /// 3-operator FM (3 operator rows)
    X3,
    /// 2-operator FM (2 operator rows)
    X2,
    /// 1-operator PCM-envelope (1 operator row)
    X1,
}

impl OpxMode {
    pub fn operator_count(&self) -> usize {
        match self {
            Self::X4 => 4,
            Self::X3 => 3,
            Self::X2 => 2,
            Self::X1 => 1,
        }
    }
}

/// OPX (YMF271) single-operator parameters (one row: AR DR SR RR SL TL KS ML DT WF ACC FB LFO AMS PMS)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OpxOperator {
    pub ar: u8,
    pub dr: u8,
    pub sr: u8,
    pub rr: u8,
    pub sl: u8,
    pub tl: u8,
    pub ks: u8,
    pub ml: u8,
    pub dt: u8,
    pub wf: u8,
    pub acc: u8,
    pub fb: u8,
    pub lfo: u8,
    pub ams: u8,
    pub pms: u8,
}

impl OpxOperator {
    /// Build from a flat parameter slice (15 values: AR DR SR RR SL TL KS ML DT WF ACC FB LFO AMS PMS).
    pub fn from_params(p: &[u32]) -> Self {
        let g = |i: usize| p.get(i).copied().unwrap_or(0) as u8;
        Self {
            ar: g(0), dr: g(1), sr: g(2), rr: g(3), sl: g(4),
            tl: g(5), ks: g(6), ml: g(7), dt: g(8), wf: g(9),
            acc: g(10), fb: g(11), lfo: g(12), ams: g(13), pms: g(14),
        }
    }
}

/// OPX (YMF271) instrument definition (modes X1–X4).
///
/// Stored separately from `FmInstrument` because the OPX operator field set
/// (15 params including DT, WF, ACC, FB, LFO, AMS, PMS) differs from the
/// OPN/OPM field set.
#[derive(Debug, Clone, PartialEq)]
pub struct OpxInstrument {
    pub number: u32,
    pub name: Option<String>,
    /// X1 / X2 / X3 / X4
    pub mode: OpxMode,
    /// One entry per operator in mode order (S1, S3, S2, S4 for X4).
    pub operators: Vec<OpxOperator>,
    /// Algorithm (CON) value — the `AL` row at the end of the definition.
    pub algorithm: u8,
}

/// Envelope definition
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub number: u32,
    pub parameters: Vec<u32>,
}

/// Arpeggio definition
#[derive(Debug, Clone, PartialEq)]
pub struct Arpeggio {
    pub number: u32,
    pub notes: Vec<Note>,
}

/// Alias definition
#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub name: String,
    pub expansion: String,
}

/// MIDI Control Change
#[derive(Debug, Clone, PartialEq)]
pub struct ControlChange {
    /// Controller number (0-127)
    pub controller: u8,
    /// Controller value (0-127)
    pub value: u8,
    /// Optional MIDI channel override (0-15)
    pub channel: Option<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI Program Change
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramChange {
    /// Program number (0-127)
    pub program: u8,
    /// Optional MIDI channel override (0-15)
    pub channel: Option<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI Pitch Bend
#[derive(Debug, Clone, PartialEq)]
pub struct PitchBend {
    /// Pitch bend value (-8192 to 8191, center = 0)
    pub value: i16,
    /// Optional MIDI channel override (0-15)
    pub channel: Option<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI Channel Aftertouch (Pressure)
#[derive(Debug, Clone, PartialEq)]
pub struct Aftertouch {
    /// Pressure value (0-127)
    pub value: u8,
    /// Optional MIDI channel override (0-15)
    pub channel: Option<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI Polyphonic Aftertouch (Note Pressure)
#[derive(Debug, Clone, PartialEq)]
pub struct PolyAftertouch {
    /// Note number (0-127)
    pub note: u8,
    /// Pressure value (0-127)
    pub value: u8,
    /// Optional MIDI channel override (0-15)
    pub channel: Option<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI System Exclusive message
#[derive(Debug, Clone, PartialEq)]
pub struct SysEx {
    /// System Exclusive data bytes
    pub data: Vec<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI Channel assignment for a part
#[derive(Debug, Clone, PartialEq)]
pub struct MidiChannel {
    /// MIDI channel (0-15)
    pub channel: u8,
    /// Source location span
    pub span: Option<Span>,
}

/// MIDI Program assignment for a part
#[derive(Debug, Clone, PartialEq)]
pub struct MidiProgram {
    /// Program number (0-127)
    pub program: u8,
    /// Optional bank MSB (0-127)
    pub bank_msb: Option<u8>,
    /// Optional bank LSB (0-127)
    pub bank_lsb: Option<u8>,
    /// Source location span
    pub span: Option<Span>,
}

/// MML AST node
#[derive(Debug, Clone, PartialEq)]
pub enum MmlNode {
    /// Note
    Note(Note),
    /// Rest
    Rest(Rest),
    /// Tempo change
    Tempo(Tempo),
    /// Volume change
    Volume(Volume),
    /// Length change
    Length(Length),
    /// Octave change
    Octave(Octave),
    /// Octave shift
    OctaveShift(OctaveShift),
    /// Instrument selection
    InstrumentSelection(InstrumentSelection),
    /// Quantize / gate time
    Quantize(Quantize),
    /// Loop
    Loop(Loop),
    /// Bar line
    Bar,
    /// Part definition
    PartDefinition(PartDefinition),
    /// Metadata
    Metadata(Metadata),
    /// Include directive
    Include(Include),
    /// FM instrument definition
    FmInstrument(FmInstrument),
    /// OPX (YMF271) instrument definition
    OpxInstrument(OpxInstrument),
    /// PCM instrument definition
    PcmInstrument(PcmInstrument),
    /// Envelope definition
    Envelope(Envelope),
    /// Arpeggio definition
    Arpeggio(Arpeggio),
    /// Alias definition
    Alias(Alias),
    /// Comment
    Comment(String),
    /// Chip-specific command
    ChipCommand {
        chip: String,
        command: String,
        args: Vec<u32>,
    },
    /// MIDI Control Change
    MidiControlChange(ControlChange),
    /// MIDI Program Change
    MidiProgramChange(ProgramChange),
    /// MIDI Pitch Bend
    MidiPitchBend(PitchBend),
    /// MIDI Channel Aftertouch
    MidiAftertouch(Aftertouch),
    /// MIDI Polyphonic Aftertouch
    MidiPolyAftertouch(PolyAftertouch),
    /// MIDI System Exclusive
    MidiSysEx(SysEx),
    /// MIDI Channel assignment
    MidiChannel(MidiChannel),
    /// MIDI Program assignment
    MidiProgram(MidiProgram),
}

/// MML AST root
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MmlAst {
    /// Metadata (song info)
    pub metadata: HashMap<String, String>,
    /// Global settings
    pub global_settings: Vec<MmlNode>,
    /// FM instruments
    pub fm_instruments: HashMap<u32, FmInstrument>,
    /// OPX (YMF271) instruments
    pub opx_instruments: HashMap<u32, OpxInstrument>,
    /// PCM instruments
    pub pcm_instruments: HashMap<u32, PcmInstrument>,
    /// Envelopes
    pub envelopes: HashMap<u32, Envelope>,
    /// Arpeggios
    pub arpeggios: HashMap<u32, Arpeggio>,
    /// Aliases
    pub aliases: HashMap<String, String>,
    /// Part definitions
    pub parts: HashMap<String, PartDefinition>,
    /// Include files
    pub includes: Vec<PathBuf>,
}

impl MmlAst {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a part by name
    pub fn get_part(&self, name: &str) -> Option<&PartDefinition> {
        self.parts.get(name)
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Get FM instrument by number
    pub fn get_fm_instrument(&self, number: u32) -> Option<&FmInstrument> {
        self.fm_instruments.get(&number)
    }

    /// Get OPX instrument by number
    pub fn get_opx_instrument(&self, number: u32) -> Option<&OpxInstrument> {
        self.opx_instruments.get(&number)
    }

    /// Get PCM instrument by number
    pub fn get_pcm_instrument(&self, number: u32) -> Option<&PcmInstrument> {
        self.pcm_instruments.get(&number)
    }
}

/// Error context for parsing errors
#[derive(Debug, Clone)]
pub struct ParseError {
    pub position: Position,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<ParseError> for MmlError {
    fn from(err: ParseError) -> Self {
        MmlError::Parse {
            line: err.position.line,
            column: err.position.column,
            message: err.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_midi_conversion() {
        let note = Note::new('C', 0, 4);
        assert_eq!(note.midi_note(), 60); // C4 = MIDI 60

        let note = Note::new('A', 0, 4);
        assert_eq!(note.midi_note(), 69); // A4 = MIDI 69

        let note = Note::new('C', 1, 4);
        assert_eq!(note.midi_note(), 61); // C#4 = MIDI 61

        let note = Note::new('C', -1, 4);
        assert_eq!(note.midi_note(), 59); // B3 (Cb4) = MIDI 59
    }

    #[test]
    fn test_mml_ast_new() {
        let ast = MmlAst::new();
        assert!(ast.metadata.is_empty());
        assert!(ast.parts.is_empty());
        assert!(ast.fm_instruments.is_empty());
    }

    #[test]
    fn test_part_definition() {
        let part = PartDefinition {
            name: "A1".to_string(),
            chip: Some("YM2612".to_string()),
            tempo: Some(120),
            commands: vec![
                MmlNode::Note(Note::new('C', 0, 4)),
                MmlNode::Rest(Rest { duration: 4, dotted: false, span: None }),
            ],
        };

        assert_eq!(part.name, "A1");
        assert_eq!(part.commands.len(), 2);
    }
}
