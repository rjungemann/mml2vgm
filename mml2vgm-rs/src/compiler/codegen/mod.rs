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

use crate::{MmlError, MmlResult, OutputFormat as LibOutputFormat, SoundChip};
use std::path::Path;

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
}

impl From<LibOutputFormat> for OutputFormat {
    fn from(format: LibOutputFormat) -> Self {
        match format {
            LibOutputFormat::VGM => OutputFormat::Vgm,
            LibOutputFormat::XGM => OutputFormat::Xgm,
            LibOutputFormat::XGM2 => OutputFormat::Xgm2,
            LibOutputFormat::ZGM => OutputFormat::Zgm,
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
            commands: vec![MmlNode::Note(note), MmlNode::Rest(Rest { duration: 60, dotted: false })],
        };

        ast.parts.insert("A1".to_string(), part);

        let generator = zgm::ZgmGenerator::from_ast(&ast, &CompileOptions::default()).unwrap();
        let result = generator.generate().unwrap();

        assert!(result.len() > 0x40);
        assert!(result.windows(2).any(|window| window == b"A1"));
        assert!(result.contains(&0x10));
    }
}
