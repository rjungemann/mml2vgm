//! XGM Format Generator
//!
//! This module generates XGM (Extended Game Music) format files from MML AST.
//!
//! # XGM Format Overview
//!
//! XGM is an extended version of VGM that supports the Sega Mega Drive/Genesis
//! sound chip configuration (YM2612 + SN76489). It uses a different command
//! structure optimized for this specific chip combination.

use super::{CodeGenerator, OutputFormat, VgmHeader};
use crate::compiler::ast::{MmlAst, MmlNode, Note};
use crate::{CompileOptions, MmlResult, SoundChip};

/// XGM generator for standard Mega Drive configuration
pub struct XgmGenerator {
    header: VgmHeader,
    chips: Vec<SoundChip>,
    fm_data: Vec<u8>,
    psg_data: Vec<u8>,
    /// XGM-specific: PCM channel overlay data
    pcm_overlay: Vec<u8>,
}

/// XGM2 generator with extended PCM frequency support
pub struct Xgm2Generator {
    header: VgmHeader,
    chips: Vec<SoundChip>,
    fm_data: Vec<u8>,
    psg_data: Vec<u8>,
    /// XGM2-specific: PCM frequency data
    pcm_frequency: Vec<u8>,
}

const XGM_FM_PART: u8 = 0xe0;
const XGM_FM_TEMPO: u8 = 0xe1;
const XGM_PSG_PART: u8 = 0xd0;
const XGM_PSG_TEMPO: u8 = 0xd1;
const XGM_FM_FREQ_WRITE_BASE: u8 = 0x30;
const XGM_FM_KEY_ON_BASE: u8 = 0x40;
const XGM_PSG_TONE_LOW_BASE: u8 = 0x10;
const XGM_PSG_TONE_HIGH_BASE: u8 = 0x20;
const XGM_FM_BLOCK_END: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const XGM_PSG_BLOCK_END: [u8; 4] = [0x0f, 0xff, 0xff, 0xff];
const YM2612_FNUM_TABLE: [u16; 12] = [
    617, 654, 693, 734, 778, 824, 873, 925, 980, 1038, 1100, 1165,
];

impl XgmGenerator {
    /// Create a new XGM generator from an AST
    pub fn from_ast(_ast: &MmlAst, _options: &CompileOptions) -> MmlResult<Self> {
        let (fm_data, psg_data) = Self::serialize_commands(_ast);
        let mut generator = Self {
            header: VgmHeader::default(),
            chips: Vec::new(),
            fm_data,
            psg_data,
            pcm_overlay: Vec::new(),
        };

        // XGM always targets YM2612 + SN76489
        generator.chips = vec![SoundChip::YM2612, SoundChip::SN76489];

        // Update header for XGM
        generator.header.ident = [b'X', b'G', b'M', b' '];
        generator.header.version = 0x00000100; // XGM version 1.00

        Ok(generator)
    }

    /// Generate XGM binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header
        self.write_header(&mut output)?;

        // Write command data
        // XGM uses a compact command format
        self.write_commands(&mut output)?;

        // Write PCM overlay if present
        if !self.pcm_overlay.is_empty() {
            output.extend_from_slice(&self.pcm_overlay);
        }

        Ok(output)
    }

    /// Write XGM header
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.header.ident);
        output.push(0x10); // Version
        output.push(0x00); // Flags
        output.extend_from_slice(&(0u16).to_le_bytes());
        output.extend_from_slice(&Self::block_units(self.fm_data.len()).to_le_bytes());
        output.extend_from_slice(&Self::block_units(self.psg_data.len()).to_le_bytes());
        while output.len() < 0x20 {
            output.push(0);
        }

        Ok(())
    }

    /// Write XGM commands
    fn write_commands(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.fm_data);
        output.extend_from_slice(&self.psg_data);
        Ok(())
    }

    fn serialize_commands(ast: &MmlAst) -> (Vec<u8>, Vec<u8>) {
        let mut fm_output = Vec::new();
        let mut psg_output = Vec::new();

        for part in ast.parts.values() {
            let is_psg = matches!(
                part.chip
                    .as_deref()
                    .and_then(|chip| chip.parse::<SoundChip>().ok()),
                Some(SoundChip::SN76489) | Some(SoundChip::SN76489X2)
            );
            let channel = Self::part_channel(&part.name, is_psg);

            let target = if is_psg {
                &mut psg_output
            } else {
                &mut fm_output
            };
            target.push(if is_psg { XGM_PSG_PART } else { XGM_FM_PART });
            target.push(part.name.len().min(u8::MAX as usize) as u8);
            target.extend_from_slice(part.name.as_bytes());

            for node in &part.commands {
                Self::push_node(node, target, is_psg, channel);
            }
        }

        fm_output.extend_from_slice(&XGM_FM_BLOCK_END);
        psg_output.extend_from_slice(&XGM_PSG_BLOCK_END);
        (fm_output, psg_output)
    }

    fn push_node(node: &MmlNode, output: &mut Vec<u8>, is_psg: bool, channel: u8) {
        match node {
            MmlNode::Note(note) => {
                if is_psg {
                    Self::push_psg_note(note, output, channel);
                } else {
                    Self::push_fm_note(note, output, channel);
                }
                Self::push_wait(
                    output,
                    Self::duration_to_frames(note.duration.unwrap_or(1)),
                    is_psg,
                );
            }
            MmlNode::Rest(rest) => {
                Self::push_wait(output, Self::duration_to_frames(rest.duration), is_psg);
            }
            MmlNode::Tempo(tempo) => {
                output.push(if is_psg { XGM_PSG_TEMPO } else { XGM_FM_TEMPO });
                output.extend_from_slice(&tempo.bpm.to_le_bytes());
            }
            MmlNode::Loop(loop_node) => {
                for _ in 0..loop_node.count {
                    for nested in &loop_node.body {
                        Self::push_node(nested, output, is_psg, channel);
                    }
                }
            }
            _ => {}
        }
    }

    fn push_fm_note(note: &Note, output: &mut Vec<u8>, channel: u8) {
        let midi = note.midi_note();
        let block = midi.saturating_div(12).saturating_sub(1).min(7);
        let fnum = Self::ym2612_fnum(midi);

        output.push(XGM_FM_FREQ_WRITE_BASE | (channel & 0x0f));
        output.push((fnum & 0x00ff) as u8);
        output.push((((fnum >> 8) as u8) & 0x07) | ((block & 0x07) << 3));
        output.push(XGM_FM_KEY_ON_BASE | (channel & 0x0f));
    }

    fn push_psg_note(note: &Note, output: &mut Vec<u8>, channel: u8) {
        let tone = Self::sn76489_tone(note.midi_note());

        output.push(XGM_PSG_TONE_LOW_BASE | (channel & 0x03));
        output.push((tone & 0x00ff) as u8);
        output.push(XGM_PSG_TONE_HIGH_BASE | (channel & 0x03));
        output.push(((tone >> 8) as u8) & 0x0f);
    }

    fn part_channel(name: &str, is_psg: bool) -> u8 {
        let parsed = name
            .chars()
            .rev()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<Vec<_>>();

        if parsed.is_empty() {
            return 0;
        }

        let number = parsed
            .into_iter()
            .rev()
            .collect::<String>()
            .parse::<u8>()
            .ok()
            .and_then(|value| value.checked_sub(1))
            .unwrap_or(0);

        if is_psg {
            number.min(3)
        } else {
            number.min(5)
        }
    }

    fn ym2612_fnum(midi_note: u8) -> u16 {
        YM2612_FNUM_TABLE[(midi_note % 12) as usize]
    }

    fn sn76489_tone(midi_note: u8) -> u16 {
        let frequency = 440.0 * 2f64.powf((midi_note as f64 - 69.0) / 12.0);
        let tone = (3_579_545.0 / (32.0 * frequency)).round();
        tone.clamp(1.0, 0x03ff as f64) as u16
    }

    fn block_units(len: usize) -> u16 {
        len.div_ceil(0x100) as u16
    }

    fn duration_to_frames(duration: u32) -> u16 {
        let frames = duration.max(1).div_ceil(16);
        frames.min(u16::MAX as u32) as u16
    }

    fn push_wait(output: &mut Vec<u8>, mut frames: u16, is_psg: bool) {
        while frames > 0 {
            if is_psg {
                if frames < 15 {
                    output.push((frames - 1) as u8);
                    break;
                }

                output.push(0x0e);
                let extra = frames.saturating_sub(15).min(255);
                output.push(extra as u8);
                frames = frames.saturating_sub(15 + extra);
            } else {
                if frames < 16 {
                    output.push((frames - 1) as u8);
                    break;
                }

                output.push(0x0f);
                let extra = frames.saturating_sub(16).min(255);
                output.push(extra as u8);
                frames = frames.saturating_sub(16 + extra);
            }
        }
    }
}

impl Xgm2Generator {
    /// Create a new XGM2 generator from an AST
    pub fn from_ast(_ast: &MmlAst, _options: &CompileOptions) -> MmlResult<Self> {
        let (fm_data, psg_data) = XgmGenerator::serialize_commands(_ast);
        let mut generator = Self {
            header: VgmHeader::default(),
            chips: Vec::new(),
            fm_data,
            psg_data,
            pcm_frequency: Vec::new(),
        };

        // XGM2 always targets YM2612 + SN76489
        generator.chips = vec![SoundChip::YM2612, SoundChip::SN76489];

        // Update header for XGM2
        generator.header.ident = [b'X', b'G', b'M', b'2'];
        generator.header.version = 0x00000100; // XGM2 version 1.00

        Ok(generator)
    }

    /// Generate XGM2 binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header
        self.write_header(&mut output)?;

        // Write command data
        self.write_commands(&mut output)?;

        // Write PCM frequency data
        if !self.pcm_frequency.is_empty() {
            output.extend_from_slice(&self.pcm_frequency);
        }

        Ok(output)
    }

    /// Write XGM2 header
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.header.ident);
        output.push(0x10); // Version
        output.push(0x00); // Flags
        output.extend_from_slice(&(0u16).to_le_bytes());
        output.extend_from_slice(&XgmGenerator::block_units(self.fm_data.len()).to_le_bytes());
        output.extend_from_slice(&XgmGenerator::block_units(self.psg_data.len()).to_le_bytes());
        while output.len() < 0x20 {
            output.push(0);
        }

        Ok(())
    }

    /// Write XGM2 commands
    fn write_commands(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.fm_data);
        output.extend_from_slice(&self.psg_data);
        Ok(())
    }
}

impl CodeGenerator for XgmGenerator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xgm
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}

impl CodeGenerator for Xgm2Generator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xgm2
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}
