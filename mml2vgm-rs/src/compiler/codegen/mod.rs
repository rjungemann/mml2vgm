//! Code Generation Module
//!
//! This module handles generating VGM, XGM, and ZGM format files from the parsed MML AST.
//!
//! # Architecture
//!
//! The code generation process:
//! 1. Semantic analysis validates and resolves the AST
//! 2. A CodeGenerator is created with the appropriate format
//! 3. The generator produces binary output with proper headers and command streams

pub mod vgm;
pub mod xgm;
pub mod zgm;
pub mod midi;
pub mod midi_controller;

use crate::{MmlError, MmlResult, OutputFormat as LibOutputFormat, SoundChip};
use std::path::Path;

/// A single note event with timing and source location information
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoteEvent {
    /// Sample offset at note-on (44100 Hz)
    pub sample_start: u64,
    /// Sample offset at note-off (i.e. start + quantized duration)
    pub sample_end: u64,
    /// Part name (e.g. "A1", "Y01")
    pub part: String,
    /// MIDI note number 0–127
    pub note_midi: u8,
    /// Instrument number (from last `@NNN` in this part), or 0 if none
    pub instrument: u32,
    /// Source line (1-indexed)
    pub line: usize,
    /// Source column start (1-indexed)
    pub col_start: usize,
    /// Source column end (1-indexed, inclusive)
    pub col_end: usize,
}

/// Source map: collection of note events with timing and source positions
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct SourceMap {
    /// All note events with timing and source information
    pub events: Vec<NoteEvent>,
}

/// Output format for code generation (internal to codegen module)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// VGM format (Video Game Music)
    Vgm,
    /// XGM format (Extended Game Music)
    Xgm,
    /// XGM2 format (Extended Game Music v2)
    Xgm2,
    /// ZGM format (ZGM Music)
    Zgm,
    /// MIDI format (Standard MIDI File)
    Midi,
}

impl From<LibOutputFormat> for OutputFormat {
    fn from(format: LibOutputFormat) -> Self {
        match format {
            LibOutputFormat::VGM => OutputFormat::Vgm,
            LibOutputFormat::XGM => OutputFormat::Xgm,
            LibOutputFormat::XGM2 => OutputFormat::Xgm2,
            LibOutputFormat::ZGM => OutputFormat::Zgm,
            LibOutputFormat::MID => OutputFormat::Midi,
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Vgm => write!(f, "vgm"),
            OutputFormat::Xgm => write!(f, "xgm"),
            OutputFormat::Xgm2 => write!(f, "xgm2"),
            OutputFormat::Zgm => write!(f, "zgm"),
            OutputFormat::Midi => write!(f, "mid"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = MmlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vgm" => Ok(OutputFormat::Vgm),
            "xgm" => Ok(OutputFormat::Xgm),
            "xgm2" => Ok(OutputFormat::Xgm2),
            "zgm" => Ok(OutputFormat::Zgm),
            "mid" => Ok(OutputFormat::Midi),
            _ => Err(MmlError::InvalidOutputFormat(s.to_string())),
        }
    }
}

/// Trait for code generators
pub trait CodeGenerator {
    /// Generate binary output from the compiled data
    fn generate(&self) -> MmlResult<Vec<u8>>;

    /// Get the output format
    fn format(&self) -> OutputFormat;

    /// Get the target sound chips
    fn chips(&self) -> &[SoundChip];
}

/// Generate code for the specified format
pub fn generate_code(
    ast: &crate::compiler::ast::MmlAst,
    options: &crate::CompileOptions,
) -> MmlResult<Vec<u8>> {
    let format = OutputFormat::from(options.output_format());

    let generator: Box<dyn CodeGenerator> = match format {
        OutputFormat::Vgm => Box::new(vgm::VgmGenerator::from_ast(ast, options)?),
        OutputFormat::Xgm => Box::new(xgm::XgmGenerator::from_ast(ast, options)?),
        OutputFormat::Xgm2 => Box::new(xgm::Xgm2Generator::from_ast(ast, options)?),
        OutputFormat::Zgm => Box::new(zgm::ZgmGenerator::from_ast(ast, options)?),
        OutputFormat::Midi => Box::new(midi::MidiGenerator::from_ast(ast, options)?),
    };

    generator.generate()
}

/// Common VGM header structure used across formats
#[derive(Debug, Clone)]
pub struct VgmHeader {
    /// Identifies the file as a VGM file ("Vgm ")
    pub ident: [u8; 4],
    /// Version number
    pub version: u32,
    /// SN76489 clock rate
    pub sn76489_clock: u32,
    /// YM2413 clock rate
    pub ym2413_clock: u32,
    /// GD3 offset
    pub gd3_offset: u32,
    /// Total number of samples
    pub total_samples: u32,
    /// Loop offset (0 if no loop)
    pub loop_offset: u32,
    /// Loop number of samples (0 if no loop)
    pub loop_samples: u32,
    /// Rate of the GD3 tag
    pub rate: u32,
    /// SN76489 feedback mask
    pub sn76489_feedback: u16,
    /// SN76489 shift register width
    pub sn76489_shift_register_width: u8,
    /// SN76489 noise flags
    pub sn76489_flags: u8,
    /// YM2612 clock rate
    pub ym2612_clock: u32,
    /// YM2151 clock rate
    pub ym2151_clock: u32,
    /// Data offset
    pub data_offset: u32,
    /// YM2203 clock rate (VGM header offset 0x44)
    pub ym2203_clock: u32,
    /// YM2608 clock rate (VGM header offset 0x48)
    pub ym2608_clock: u32,
    /// YM2610/YM2610B clock rate (VGM header offset 0x4C)
    pub ym2610b_clock: u32,
    /// YM3812 clock rate (VGM header offset 0x50)
    pub ym3812_clock: u32,
    /// YM3526 clock rate (VGM header offset 0x54)
    pub ym3526_clock: u32,
    /// Y8950 clock rate (VGM header offset 0x58)
    pub y8950_clock: u32,
    /// YMF262 clock rate (VGM header offset 0x5C)
    pub ymf262_clock: u32,
    /// DMG (Game Boy APU) clock rate (VGM header offset 0x80)
    pub dmg_clock: u32,
    /// NES APU clock rate (VGM header offset 0x84)
    pub nes_apu_clock: u32,
    /// OKIM6295/K051649 flags (VGM header offset 0x94)
    /// bit 31: K051649 present, bit 30: K052539 present, bits 0-1: OKIM6295 clock divider
    pub k051649_flags: u32,
    /// K051649/K052539 clock rate (VGM header offset 0x9C)
    pub k051649_clock: u32,
    /// YMF271 clock rate (VGM header offset 0x64)
    pub ymf271_clock: u32,
    /// RF5C164 clock rate (VGM header offset 0x6C)
    pub rf5c164_clock: u32,
    /// AY8910 clock rate (VGM header offset 0x74)
    pub ay8910_clock: u32,
    /// K054539 clock rate (VGM header offset 0xA0)
    pub k054539_clock: u32,
    /// HuC6280 clock rate (VGM header offset 0xA4)
    pub huc6280_clock: u32,
    /// C140 clock rate (VGM header offset 0xA8)
    pub c140_clock: u32,
    /// K053260 clock rate (VGM header offset 0xAC)
    pub k053260_clock: u32,
    /// Pokey clock rate (VGM header offset 0xB0)
    pub pokey_clock: u32,
    /// QSound clock rate (VGM header offset 0xB4)
    pub qsound_clock: u32,
    /// C352 clock rate (VGM header offset 0xDC)
    pub c352_clock: u32,
    /// VRC6 clock rate (VGM header extension)
    pub vrc6_clock: u32,
    /// SegaPCM clock rate (VGM header offset 0x38)
    pub segapcm_clock: u32,
    /// YM2610 clock rate (unused in current header layout)
    pub ym2610_clock: u32,
    /// YM2413 extended clock (unused)
    pub ym2413_clock_ext: u32,
    /// YM2610B extended clock (unused)
    pub ym2610b_clock_ext: u32,
}

impl Default for VgmHeader {
    fn default() -> Self {
        Self {
            ident: [b'V', b'g', b'm', b' '],
            version: 0x00000171, // Version 1.71
            sn76489_clock: 3_579_545,
            ym2413_clock: 3_579_545,
            gd3_offset: 0,
            total_samples: 0,
            loop_offset: 0,
            loop_samples: 0,
            rate: 44100,
            sn76489_feedback: 0x0009,
            sn76489_shift_register_width: 16,
            sn76489_flags: 0,
            ym2612_clock: 7_670_453,
            ym2151_clock: 3_579_545,
            data_offset: 0,
            ym2203_clock: 0,
            ym2608_clock: 0,
            ym2610b_clock: 0,
            ym3812_clock: 0,
            ym3526_clock: 0,
            y8950_clock: 0,
            ymf262_clock: 0,
            dmg_clock: 0,
            nes_apu_clock: 0,
            k051649_flags: 0,
            k051649_clock: 0,
            ym2610_clock: 0,
            segapcm_clock: 0,
            rf5c164_clock: 0,
            ym2413_clock_ext: 0,
            ym2610b_clock_ext: 0,
            ymf271_clock: 0,
            ay8910_clock: 0,
            huc6280_clock: 0,
            c140_clock: 0,
            k053260_clock: 0,
            k054539_clock: 0,
            qsound_clock: 0,
            c352_clock: 0,
            pokey_clock: 0,
            vrc6_clock: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::ast::{MmlAst, Note, PartDefinition, Rest, MmlNode};
    use crate::CompileOptions;

    #[test]
    fn test_vgm_header_default() {
        let header = VgmHeader::default();
        assert_eq!(header.ident, [b'V', b'g', b'm', b' ']);
        assert_eq!(header.version, 0x00000171);
        assert_eq!(header.sn76489_clock, 3_579_545);
        assert_eq!(header.rate, 44100);
        // New console chip fields should default to 0
        assert_eq!(header.dmg_clock, 0);
        assert_eq!(header.nes_apu_clock, 0);
        assert_eq!(header.k051649_flags, 0);
        assert_eq!(header.k051649_clock, 0);
    }

    #[test]
    fn test_vgm_header_dmg_clock_offset() {
        let mut ast = MmlAst::new();
        let mut part = PartDefinition {
            name: "DMG1".to_string(),
            chip: Some("DMG".to_string()),
            tempo: Some(120),
            commands: vec![],
        };
        ast.parts.insert("DMG1".to_string(), part);
        
        let options = CompileOptions::default();
        let generator = vgm::VgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // Check DMG clock at offset 0x80
        let dmg_clock_bytes = &result[0x80..0x84];
        assert_eq!(dmg_clock_bytes, &4_194_304u32.to_le_bytes());
    }

    #[test]
    fn test_vgm_header_nes_apu_clock_offset() {
        let mut ast = MmlAst::new();
        let mut part = PartDefinition {
            name: "NES1".to_string(),
            chip: Some("NES".to_string()),
            tempo: Some(120),
            commands: vec![],
        };
        ast.parts.insert("NES1".to_string(), part);
        
        let options = CompileOptions::default();
        let generator = vgm::VgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // Check NES APU clock at offset 0x84
        let nes_clock_bytes = &result[0x84..0x88];
        assert_eq!(nes_clock_bytes, &1_789_772u32.to_le_bytes());
    }

    #[test]
    fn test_vgm_header_k051649_clock_offset() {
        let mut ast = MmlAst::new();
        let mut part = PartDefinition {
            name: "SCC1".to_string(),
            chip: Some("K051649".to_string()),
            tempo: Some(120),
            commands: vec![],
        };
        ast.parts.insert("SCC1".to_string(), part);
        
        let options = CompileOptions::default();
        let generator = vgm::VgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // Check K051649 clock at offset 0x9C
        let k051649_clock_bytes = &result[0x9C..0xA0];
        assert_eq!(k051649_clock_bytes, &1_789_772u32.to_le_bytes());
        
        // Check K051649 flags at offset 0x94 - bit 31 should be set
        let k051649_flags_bytes = &result[0x94..0x98];
        let flags = u32::from_le_bytes([k051649_flags_bytes[0], k051649_flags_bytes[1], k051649_flags_bytes[2], k051649_flags_bytes[3]]);
        assert!(flags & 0x80000000 != 0, "K051649 flag bit 31 should be set");
    }

    #[test]
    fn test_output_format_conversion() {
        use crate::OutputFormat as LibFormat;
        
        assert_eq!(
            OutputFormat::from(LibFormat::VGM),
            OutputFormat::Vgm
        );
        assert_eq!(
            OutputFormat::from(LibFormat::XGM),
            OutputFormat::Xgm
        );
        assert_eq!(
            OutputFormat::from(LibFormat::XGM2),
            OutputFormat::Xgm2
        );
        assert_eq!(
            OutputFormat::from(LibFormat::ZGM),
            OutputFormat::Zgm
        );
        assert_eq!(
            OutputFormat::from(LibFormat::MID),
            OutputFormat::Midi
        );
    }

    #[test]
    fn test_vgm_generator_basic() {
        let ast = MmlAst::new();
        let options = CompileOptions::default();
        
        let generator = vgm::VgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // VGM header is 0x100 bytes
        assert!(result.len() >= 0x100);
        
        // Check ident
        assert_eq!(&result[0..4], [b'V', b'g', b'm', b' ']);
        
        // Check end of sound data marker (0x66)
        assert!(result.contains(&0x66));
    }

    #[test]
    fn test_vgm_generator_with_note() {
        let mut ast = MmlAst::new();
        
        // Create a simple note
        let mut note = Note::new('C', 0, 4);
        note.duration = Some(480); // Quarter note at 120 BPM
        
        // Create a part with the note
        let mut part = PartDefinition {
            name: "FM1".to_string(),
            chip: Some("YM2612".to_string()),
            tempo: Some(120),
            commands: vec![MmlNode::Note(note)],
        };
        
        ast.parts.insert("FM1".to_string(), part);
        
        let options = CompileOptions::default();
        let generator = vgm::VgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // Should generate some commands
        assert!(result.len() > 0x100);
    }

    #[test]
    fn test_xgm_generator_basic() {
        let ast = MmlAst::new();
        let options = CompileOptions::default();
        
        let generator = xgm::XgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // XGM header
        assert!(result.len() >= 0x20);
        assert_eq!(&result[0..4], [b'X', b'G', b'M', b' ']);
        assert_eq!(&result[result.len() - 4..], &[0x0f, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn test_xgm_generator_with_note_payload() {
        let mut ast = MmlAst::new();
        let mut note = Note::new('C', 0, 4);
        note.duration = Some(240);

        let part = PartDefinition {
            name: "FM1".to_string(),
            chip: Some("YM2612".to_string()),
            tempo: Some(120),
            commands: vec![MmlNode::Tempo(crate::compiler::ast::Tempo { bpm: 120 }), MmlNode::Note(note)],
        };

        ast.parts.insert("FM1".to_string(), part);

        let generator = xgm::XgmGenerator::from_ast(&ast, &CompileOptions::default()).unwrap();
        let result = generator.generate().unwrap();

        assert!(result.len() > 0x20);
        assert!(result[0x20..].iter().any(|byte| (byte & 0xf0) == 0x30));
        assert!(result[0x20..].iter().any(|byte| (byte & 0xf0) == 0x40));
        assert!(result[0x20..].iter().any(|byte| *byte <= 0x0f));
    }

    #[test]
    fn test_xgm2_generator_with_note_payload() {
        let mut ast = MmlAst::new();
        let mut note = Note::new('G', 0, 4);
        note.duration = Some(180);

        let part = PartDefinition {
            name: "PSG1".to_string(),
            chip: Some("SN76489".to_string()),
            tempo: Some(120),
            commands: vec![MmlNode::Note(note)],
        };

        ast.parts.insert("PSG1".to_string(), part);

        let generator = xgm::Xgm2Generator::from_ast(&ast, &CompileOptions::default()).unwrap();
        let result = generator.generate().unwrap();

        assert!(result.len() > 0x20);
        assert_eq!(&result[0..4], [b'X', b'G', b'M', b'2']);
        assert!(result[0x20..].iter().any(|byte| (byte & 0xf0) == 0x10));
        assert!(result[0x20..].iter().any(|byte| (byte & 0xf0) == 0x20));
        assert_eq!(&result[result.len() - 4..], &[0x0f, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn test_zgm_generator_basic() {
        let ast = MmlAst::new();
        let options = CompileOptions::default();
        
        let generator = zgm::ZgmGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();
        
        // ZGM header
        assert!(result.len() >= 0x40);
        assert_eq!(&result[0..4], [b'Z', b'G', b'M', b' ']);
        assert!(result.windows(3).any(|window| window == b"Def"));
        assert!(result.windows(3).any(|window| window == b"Trk"));
    }

    #[test]
    fn test_zgm_generator_with_note_payload() {
        let mut ast = MmlAst::new();
        let mut note = Note::new('E', 0, 4);
        note.duration = Some(120);

        let part = PartDefinition {
            name: "A1".to_string(),
            chip: Some("SN76489".to_string()),
            tempo: Some(150),
            commands: vec![MmlNode::Note(note), MmlNode::Rest(Rest { duration: 60, dotted: false, span: None })],
        };

        ast.parts.insert("A1".to_string(), part);

        let generator = zgm::ZgmGenerator::from_ast(&ast, &CompileOptions::default()).unwrap();
        let result = generator.generate().unwrap();

        assert!(result.len() > 0x40);
        assert!(result.windows(2).any(|window| window == b"A1"));
        assert!(result.contains(&0x10));
    }
}
