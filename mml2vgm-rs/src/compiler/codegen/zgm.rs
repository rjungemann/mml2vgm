//! ZGM Format Generator
//!
//! This module generates ZGM format files from MML AST.
//!
//! # ZGM Format Overview
//!
//! ZGM is a format developed for the Sega Mega Drive/Genesis that supports
//! extended features beyond standard VGM. It includes Define and Track divisions
//! for more flexible music composition.

use super::{CodeGenerator, OutputFormat, VgmHeader};
use crate::compiler::ast::{MmlAst, MmlNode};
use crate::{CompileOptions, MmlResult, SoundChip};

/// ZGM generator
pub struct ZgmGenerator {
    header: VgmHeader,
    chips: Vec<SoundChip>,
    /// Define division data
    define_division: Vec<u8>,
    /// Track division data
    track_division: Vec<u8>,
    define_count: u16,
    track_count: u16,
}

struct ZgmChipDefinition {
    ident: u32,
    command_no: u16,
    clock: u32,
}

impl ZgmGenerator {
    /// Create a new ZGM generator from an AST
    pub fn from_ast(ast: &MmlAst, options: &CompileOptions) -> MmlResult<Self> {
        let mut generator = Self {
            header: VgmHeader::default(),
            chips: Vec::new(),
            define_division: Vec::new(),
            track_division: Vec::new(),
            define_count: 0,
            track_count: 0,
        };

        generator.chips = Self::extract_chips(ast, options);

        // Update header for ZGM
        generator.header.ident = [b'Z', b'G', b'M', b' '];
        generator.header.version = 0x00000100; // ZGM version 1.00

        let (define_division, define_count) = generator.build_define_division();
        generator.define_division = define_division;
        generator.define_count = define_count;
        generator.track_division = Self::build_track_division(ast);
        generator.track_count = u16::from(!generator.track_division.is_empty());

        Ok(generator)
    }

    /// Generate ZGM binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header
        self.write_header(&mut output)?;

        // Write Define division
        self.write_define_division(&mut output)?;

        // Write Track division
        self.write_track_division(&mut output)?;

        let eof_offset = output.len().saturating_sub(1) as u32;
        let define_offset = if self.define_division.is_empty() { 0 } else { 0x40u32 };
        let track_offset = if self.track_division.is_empty() {
            0
        } else {
            define_offset + self.define_division.len() as u32
        };

        output[0x04..0x08].copy_from_slice(&eof_offset.to_le_bytes());
        output[0x1c..0x20].copy_from_slice(&define_offset.to_le_bytes());
        output[0x20..0x24].copy_from_slice(&track_offset.to_le_bytes());
        output[0x24..0x26].copy_from_slice(&self.define_count.to_le_bytes());
        output[0x26..0x28].copy_from_slice(&self.track_count.to_le_bytes());

        Ok(output)
    }

    /// Write ZGM header
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.header.ident);
        output.extend_from_slice(&0u32.to_le_bytes()); // EOF offset
        output.extend_from_slice(&10u32.to_le_bytes()); // Version, following legacy builder
        output.extend_from_slice(&0u32.to_le_bytes()); // Total samples
        output.extend_from_slice(&0u32.to_le_bytes()); // Loop samples
        output.extend_from_slice(&0u32.to_le_bytes()); // Loop offset
        output.extend_from_slice(&0u32.to_le_bytes()); // GD3 offset
        output.extend_from_slice(&0u32.to_le_bytes()); // Define offset
        output.extend_from_slice(&0u32.to_le_bytes()); // Track 1 offset
        output.extend_from_slice(&0u16.to_le_bytes()); // Define count
        output.extend_from_slice(&0u16.to_le_bytes()); // Track count
        output.extend_from_slice(&0u32.to_le_bytes()); // Extra hdr offset
        while output.len() < 0x40 {
            output.push(0);
        }

        Ok(())
    }

    /// Write Define division
    /// 
    /// The Define division contains instrument definitions, PCM data,
    /// and other setup information.
    fn write_define_division(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.define_division);

        Ok(())
    }

    /// Write Track division
    ///
    /// The Track division contains the actual music data organized by tracks.
    fn write_track_division(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.track_division);

        Ok(())
    }

    fn extract_chips(ast: &MmlAst, options: &CompileOptions) -> Vec<SoundChip> {
        let mut chips = options.target_chips.clone().unwrap_or_default();

        for part in ast.parts.values() {
            if let Some(chip_name) = &part.chip {
                if let Ok(chip) = chip_name.parse::<SoundChip>() {
                    if !chips.contains(&chip) {
                        chips.push(chip);
                    }
                }
            }
        }

        if chips.is_empty() {
            chips.push(SoundChip::YM2612);
            chips.push(SoundChip::SN76489);
        }

        chips
    }

    fn build_define_division(&self) -> (Vec<u8>, u16) {
        let mut division = Vec::new();
        let mut next_command_no = 0x80u16;
        let mut count = 0u16;

        for chip in &self.chips {
            let Some(ident) = Self::chip_ident(chip) else {
                continue;
            };
            let definition = ZgmChipDefinition {
                ident,
                command_no: next_command_no,
                clock: chip.clock_rate(),
            };
            next_command_no = next_command_no.saturating_add(Self::chip_port_count(chip));

            division.extend_from_slice(b"Def");
            division.push(14);
            division.extend_from_slice(&definition.ident.to_le_bytes());
            division.extend_from_slice(&definition.command_no.to_le_bytes());
            division.extend_from_slice(&definition.clock.to_le_bytes());
            count = count.saturating_add(1);
        }

        (division, count)
    }

    fn build_track_division(ast: &MmlAst) -> Vec<u8> {
        let mut division = Vec::new();
        division.extend_from_slice(b"Trk");

        let mut payload = Vec::new();
        payload.extend_from_slice(&u32::MAX.to_le_bytes());

        for part in ast.parts.values() {
            payload.push(part.name.len().min(u8::MAX as usize) as u8);
            payload.extend_from_slice(part.name.as_bytes());

            for node in &part.commands {
                Self::push_track_node(node, &mut payload);
            }
            payload.push(0xfe);
        }

        payload.push(0xff);
        let division_len = (3 + 4 + payload.len()) as u32;
        division.extend_from_slice(&division_len.to_le_bytes());
        division.extend_from_slice(&payload);
        division
    }

    fn push_track_node(node: &MmlNode, output: &mut Vec<u8>) {
        match node {
            MmlNode::Note(note) => {
                output.push(0x10);
                output.push(note.midi_note());
                output.extend_from_slice(&note.duration.unwrap_or(1).to_le_bytes());
            }
            MmlNode::Rest(rest) => {
                output.push(0x11);
                output.extend_from_slice(&rest.duration.to_le_bytes());
            }
            MmlNode::Tempo(tempo) => {
                output.push(0x12);
                output.extend_from_slice(&tempo.bpm.to_le_bytes());
            }
            MmlNode::Loop(loop_node) => {
                for _ in 0..loop_node.count {
                    for nested in &loop_node.body {
                        Self::push_track_node(nested, output);
                    }
                }
            }
            _ => {}
        }
    }

    fn chip_ident(chip: &SoundChip) -> Option<u32> {
        match chip {
            SoundChip::SN76489 => Some(0x0000_000c),
            SoundChip::YM2413 => Some(0x0000_0010),
            SoundChip::YM2612 => Some(0x0000_002c),
            SoundChip::YM2151 => Some(0x0000_0030),
            SoundChip::SegaPCM => Some(0x0000_0038),
            SoundChip::YM2203 => Some(0x0000_0044),
            SoundChip::YM2608 => Some(0x0000_0048),
            SoundChip::YM2610B => Some(0x0000_004c),
            SoundChip::YM3812 => Some(0x0000_0050),
            SoundChip::YM3526 => Some(0x0000_0054),
            SoundChip::Y8950 => Some(0x0000_0058),
            SoundChip::YMF262 => Some(0x0000_005c),
            SoundChip::RF5C164 => Some(0x0000_006c),
            SoundChip::AY8910 => Some(0x0000_0074),
            SoundChip::DMG => Some(0x0000_0080),
            SoundChip::NES => Some(0x0000_0084),
            SoundChip::K051649 => Some(0x0000_009c),
            SoundChip::K054539 => Some(0x0000_00a0),
            SoundChip::HuC6280 => Some(0x0000_00a4),
            SoundChip::C140 => Some(0x0000_00a8),
            SoundChip::K053260 => Some(0x0000_00ac),
            SoundChip::POKEY => Some(0x0000_00b0),
            SoundChip::QSound => Some(0x0000_00b4),
            SoundChip::C352 => Some(0x0000_00dc),
            SoundChip::CONDUCTOR => Some(0x0001_0000),
            SoundChip::VRC6 => Some(0x0001_0004),
            SoundChip::YM2609 => Some(0x0002_0001),
            SoundChip::MIDI => Some(0x0005_0000),
            _ => None,
        }
    }

    fn chip_port_count(chip: &SoundChip) -> u16 {
        match chip {
            SoundChip::YM2612 | SoundChip::YM2608 | SoundChip::YM2610B => 2,
            SoundChip::YM2609 => 4,
            SoundChip::SN76489 => 2,
            _ => 1,
        }
    }
}

impl CodeGenerator for ZgmGenerator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Zgm
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}
