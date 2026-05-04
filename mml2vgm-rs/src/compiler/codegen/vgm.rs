//! VGM Format Generator
//!
//! This module generates VGM (Video Game Music) format files from MML AST.
//!
//! # VGM Format Overview
//!
//! VGM is a binary format that stores music data for various sound chips.
//! It includes a header with metadata, followed by a stream of commands
//! that write to sound chip registers at specific times.

use super::{CodeGenerator, OutputFormat, VgmHeader};
use crate::compiler::ast::{MmlAst, MmlNode};
use crate::{CompileOptions, MmlError, MmlResult, SoundChip};

/// VGM command types
/// Note: Values match the VGM specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VgmCommandType {
    /// Write to SN76489 PSG
    Sn76489Write = 0x50,
    /// Write to YM2413 OPLL
    Ym2413Write = 0x51,
    /// Write to YM2612 OPN2 port 0
    Ym2612WritePort0 = 0x52,
    /// Write to YM2612 OPN2 port 1
    Ym2612WritePort1 = 0x53,
    /// Write to YM2151 OPM
    Ym2151Write = 0x54,
    /// Write to YM2608 OPNA port 0
    Ym2608WritePort0 = 0x55,
    /// Write to YM2608 OPNA port 1
    Ym2608WritePort1 = 0x56,
    /// Write to YM2610 OPNB port 0
    Ym2610WritePort0 = 0x57,
    /// Write to YM2610 OPNB port 1
    Ym2610WritePort1 = 0x58,
    /// Write to YM3812 OPL2
    Ym3812Write = 0x5A,
    /// Write to YMF262 OPL3 port 0
    Ymf262WritePort0 = 0x5B,
    /// Write to YMF262 OPL3 port 1
    Ymf262WritePort1 = 0x5C,
    /// Write to RF5C164 PCM
    Rf5c164Write = 0x68,
    /// Wait n samples
    Wait = 0x61,
    /// Wait 1 sample (short form)
    Wait1 = 0x62,
    /// Wait 2 samples (short form)
    Wait2 = 0x63,
    /// End of sound data
    End = 0x66,
    /// Data block (0x67 is also used for PCM data with type byte)
    DataBlock = 0x67,
}

/// A single VGM command
#[derive(Debug, Clone)]
pub struct VgmCommand {
    pub command_type: VgmCommandType,
    pub data: Vec<u8>,
    pub time: u64, // Sample count from start
}

/// VGM file generator
pub struct VgmGenerator {
    header: VgmHeader,
    commands: Vec<VgmCommand>,
    chips: Vec<SoundChip>,
    pcm_data: Vec<PcmData>,
    gd3_tag: Option<Gd3Tag>,
}

/// PCM data for embedding in VGM
#[derive(Debug, Clone)]
pub struct PcmData {
    pub data: Vec<u8>,
    pub start_offset: u32,
}

/// GD3 tag for metadata (Game Gear/Sega Master System)
#[derive(Debug, Clone)]
pub struct Gd3Tag {
    pub track_name_en: String,
    pub track_name_jp: String,
    pub game_name_en: String,
    pub game_name_jp: String,
    pub system_name_en: String,
    pub system_name_jp: String,
    pub author_en: String,
    pub author_jp: String,
    pub release_date: String,
    pub converter: String,
    pub notes: String,
}

impl Default for Gd3Tag {
    fn default() -> Self {
        Self {
            track_name_en: String::new(),
            track_name_jp: String::new(),
            game_name_en: String::new(),
            game_name_jp: String::new(),
            system_name_en: String::new(),
            system_name_jp: String::new(),
            author_en: String::new(),
            author_jp: String::new(),
            release_date: String::new(),
            converter: String::new(),
            notes: String::new(),
        }
    }
}

impl VgmGenerator {
    /// Create a new VGM generator from an AST
    pub fn from_ast(ast: &MmlAst, options: &CompileOptions) -> MmlResult<Self> {
        let mut generator = Self {
            header: VgmHeader::default(),
            commands: Vec::new(),
            chips: Vec::new(),
            pcm_data: Vec::new(),
            gd3_tag: None,
        };

        // Extract metadata from AST
        generator.extract_metadata(ast);

        // Extract chip information from AST
        generator.extract_chips(ast, options);

        // Convert AST nodes to VGM commands
        generator.convert_ast_to_commands(ast)?;

        // Build GD3 tag if metadata exists
        generator.build_gd3_tag(ast);

        // Calculate total samples and data offset
        generator.calculate_header();

        Ok(generator)
    }

    /// Extract metadata from AST and update header
    fn extract_metadata(&mut self, ast: &MmlAst) {
        // Set rate from AST metadata if available
        // For now, use default rate
        // Version 1.71 supports most chips
        self.header.version = 0x00000171;
    }

    /// Extract chip information from AST and options
    fn extract_chips(&mut self, ast: &MmlAst, options: &CompileOptions) {
        // Get chips from options
        if let Some(ref chips) = options.target_chips {
            for chip in chips {
                if !self.chips.contains(chip) {
                    self.chips.push(*chip);
                }
            }
        }

        // Get chips from AST parts
        for part in ast.parts.values() {
            if let Some(ref chip_str) = part.chip {
                // Convert string to SoundChip
                // This is a simplified mapping
                let chip = match chip_str.to_uppercase().as_str() {
                    "YM2612" => SoundChip::YM2612,
                    "SN76489" => SoundChip::SN76489,
                    "YM2151" => SoundChip::YM2151,
                    "YM2413" => SoundChip::YM2413,
                    _ => continue,
                };
                if !self.chips.contains(&chip) {
                    self.chips.push(chip);
                }
            }
        }

        // If no chips specified, default to YM2612 + SN76489 (Mega Drive)
        if self.chips.is_empty() {
            self.chips = vec![SoundChip::YM2612, SoundChip::SN76489];
        }

        // Update header clocks based on chips
        for chip in &self.chips {
            match chip {
                SoundChip::SN76489 | SoundChip::SN76489X2 => {
                    self.header.sn76489_clock = chip.clock_rate();
                }
                SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2 => {
                    self.header.ym2612_clock = chip.clock_rate();
                }
                SoundChip::YM2151 => {
                    self.header.ym2151_clock = chip.clock_rate();
                }
                SoundChip::YM2413 => {
                    self.header.ym2413_clock = chip.clock_rate();
                }
                _ => {}
            }
        }
    }

    /// Convert AST nodes to VGM commands
    fn convert_ast_to_commands(&mut self, ast: &MmlAst) -> MmlResult<()> {
        let mut current_time: u64 = 0;

        // Process global settings
        for node in &ast.global_settings {
            self.process_node(node, &mut current_time)?;
        }

        // Process parts
        for part in ast.parts.values() {
            self.process_part(part, &mut current_time)?;
        }

        Ok(())
    }

    /// Process a part definition
    fn process_part(&mut self, part: &crate::compiler::ast::PartDefinition, time: &mut u64) -> MmlResult<()> {
        // Parts are processed based on their target chip
        // For now, just process all commands sequentially
        for node in &part.commands {
            self.process_node(node, time)?;
        }
        Ok(())
    }

    /// Process a single AST node into VGM commands
    fn process_node(&mut self, node: &MmlNode, time: &mut u64) -> MmlResult<()> {
        match node {
            MmlNode::Note(note) => {
                // Convert note to appropriate chip register writes
                // For now, this is a placeholder - actual implementation
                // will depend on the target chip
                self.process_note(note, time)?;
            }
            MmlNode::Rest(rest) => {
                // Rest = wait for duration
                let samples = self.duration_to_samples(rest.duration);
                *time += samples as u64;
                self.commands.push(VgmCommand {
                    command_type: VgmCommandType::Wait,
                    data: samples.to_le_bytes().to_vec(),
                    time: *time,
                });
            }
            MmlNode::Tempo(tempo) => {
                // Tempo change affects timing calculations
                // VGM doesn't have a tempo command - tempo is handled by wait times
            }
            MmlNode::Volume(vol) => {
                // Volume changes are chip-specific
            }
            MmlNode::Length(len) => {
                // Default length for subsequent notes
            }
            MmlNode::Octave(oct) => {
                // Default octave for subsequent notes
            }
            MmlNode::Loop(loop_node) => {
                // Handle loops
                for _ in 0..loop_node.count {
                    for node in &loop_node.body {
                        self.process_node(node, time)?;
                    }
                }
            }
            MmlNode::Metadata(metadata) => {
                // Metadata is stored in GD3 tag
            }
            MmlNode::Include(include) => {
                // Includes should be resolved during parsing
            }
            _ => {
                // Other node types (FM instruments, PCM instruments, etc.)
                // will be handled in future implementations
            }
        }
        Ok(())
    }

    /// Process a note into chip-specific register writes
    fn process_note(&mut self, note: &crate::compiler::ast::Note, time: &mut u64) -> MmlResult<()> {
        // Get MIDI note number
        let midi_note = note.midi_note();

        // For now, generate a simple wait command
        // Actual implementation will write to chip registers
        let duration = note.duration.unwrap_or(1);
        let samples = self.duration_to_samples(duration);

        // Placeholder: Write to YM2612 (most common Mega Drive chip)
        // This would be replaced with actual register calculations
        if self.chips.contains(&SoundChip::YM2612) {
            // YM2612 register writes would go here
            // For now, just add a wait
        }

        // Handle PSG (SN76489) if present
        if self.chips.contains(&SoundChip::SN76489) {
            // SN76489 tone generation would go here
            // Frequency = block * 1024 + tone
            let (block, tone) = self.midi_note_to_psg_freq(midi_note);
            
            // Write tone register ( channels 0-2 are tone, 3 is noise)
            // For channel 0:
            self.commands.push(VgmCommand {
                command_type: VgmCommandType::Sn76489Write,
                data: vec![0x80 | (block & 0x07) as u8], // Channel 0 tone high 4 bits
                time: *time,
            });
            self.commands.push(VgmCommand {
                command_type: VgmCommandType::Sn76489Write,
                data: vec![(tone >> 4) as u8], // Channel 0 tone low 4 bits
                time: *time,
            });
            
            // Volume (attenuation) - lower = louder
            let volume = 0x0F; // Max volume for now
            self.commands.push(VgmCommand {
                command_type: VgmCommandType::Sn76489Write,
                data: vec![0x90 | (volume & 0x0F) as u8], // Channel 0 volume
                time: *time,
            });
        }

        *time += samples as u64;
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Wait,
            data: samples.to_le_bytes().to_vec(),
            time: *time,
        });

        Ok(())
    }

    /// Convert MIDI note to PSG frequency (SN76489)
    fn midi_note_to_psg_freq(&self, midi_note: u8) -> (u8, u16) {
        // PSG frequency formula:
        // f = clock / (32 * divider)
        // divider = block * 1024 + tone
        // For SN76489 clock = 3,579,545 Hz
        
        // MIDI note to frequency
        let freq = 440.0 * (2.0_f64).powf((midi_note as i32 - 69) as f64 / 12.0);
        
        // Calculate divider
        let clock = self.header.sn76489_clock as f64;
        let divider = clock / (32.0 * freq);
        
        // divider = block * 1024 + tone, where block is 0-7, tone is 0-1023
        let divider_int = divider.round() as u32;
        
        if divider_int == 0 {
            (0, 0)
        } else {
            // Calculate block and tone
            // The PSG has 10-bit tone values split across registers
            // tone_high = tone >> 4 (4 bits)
            // tone_low = tone & 0x0F (4 bits)
            // block is determined by the tone value
            
            // Simplified: use 10-bit tone value
            let tone_val = divider_int.min(1023);
            let block = (divider_int / 1024).min(7) as u8;
            let tone = (divider_int % 1024) as u16;
            
            (block, tone)
        }
    }

    /// Convert duration to samples
    fn duration_to_samples(&self, duration: u32) -> u32 {
        // Default: 44100 samples per second
        // Quarter note = 1 beat, duration is in ticks
        // For now, assume 1 tick = 1 sample (placeholder)
        duration
    }

    /// Build GD3 tag from metadata
    fn build_gd3_tag(&mut self, ast: &MmlAst) {
        let mut tag = Gd3Tag::default();

        // Extract metadata from AST
        for (key, value) in &ast.metadata {
            match key.to_lowercase().as_str() {
                "title" | "name" => {
                    tag.track_name_en = value.clone();
                }
                "author" | "composer" => {
                    tag.author_en = value.clone();
                }
                "game" => {
                    tag.game_name_en = value.clone();
                }
                "system" => {
                    tag.system_name_en = value.clone();
                }
                "date" => {
                    tag.release_date = value.clone();
                }
                "converter" => {
                    tag.converter = value.clone();
                }
                "notes" | "comment" => {
                    tag.notes = value.clone();
                }
                _ => {}
            }
        }

        self.gd3_tag = Some(tag);
    }

    /// Calculate header fields
    fn calculate_header(&mut self) {
        // Calculate total samples from commands
        let mut total_samples: u32 = 0;
        for cmd in &self.commands {
            match cmd.command_type {
                VgmCommandType::Wait => {
                    if cmd.data.len() >= 4 {
                        let samples = u32::from_le_bytes([cmd.data[0], cmd.data[1], cmd.data[2], cmd.data[3]]);
                        total_samples += samples;
                    }
                }
                VgmCommandType::Wait1 => {
                    total_samples += 1;
                }
                VgmCommandType::Wait2 => {
                    total_samples += 2;
                }
                _ => {}
            }
        }
        self.header.total_samples = total_samples;

        // Calculate data offset (header size + chip-specific header extensions)
        // Base header is 0x100 bytes for VGM 1.71
        self.header.data_offset = 0x100;
    }

    /// Generate the VGM file binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header
        self.write_header(&mut output)?;

        // Write commands
        self.write_commands(&mut output)?;

        // Write GD3 tag if present
        if let Some(ref tag) = self.gd3_tag {
            self.write_gd3_tag(tag, &mut output)?;
        }

        // Write PCM data if present
        for pcm in &self.pcm_data {
            self.write_pcm_data_block(pcm, &mut output)?;
        }

        Ok(output)
    }

    /// Write VGM header
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        // Write ident
        output.extend_from_slice(&self.header.ident);

        // Write version
        output.extend_from_slice(&self.header.version.to_le_bytes());

        // Write SN76489 clock
        output.extend_from_slice(&self.header.sn76489_clock.to_le_bytes());

        // Write YM2413 clock
        output.extend_from_slice(&self.header.ym2413_clock.to_le_bytes());

        // Write GD3 offset
        output.extend_from_slice(&self.header.gd3_offset.to_le_bytes());

        // Write total samples
        output.extend_from_slice(&self.header.total_samples.to_le_bytes());

        // Write loop offset
        output.extend_from_slice(&self.header.loop_offset.to_le_bytes());

        // Write loop samples
        output.extend_from_slice(&self.header.loop_samples.to_le_bytes());

        // Write rate
        output.extend_from_slice(&self.header.rate.to_le_bytes());

        // Write SN76489 feedback
        output.extend_from_slice(&self.header.sn76489_feedback.to_le_bytes());

        // Write SN76489 shift register width
        output.push(self.header.sn76489_shift_register_width);

        // Write SN76489 flags
        output.push(self.header.sn76489_flags);

        // Reserved bytes (0x100 - current position)
        while output.len() < 0x100 {
            output.push(0);
        }

        Ok(())
    }

    /// Write VGM commands
    fn write_commands(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        for cmd in &self.commands {
            // Write command byte
            output.push(cmd.command_type as u8);

            // Write data bytes
            output.extend_from_slice(&cmd.data);
        }

        // Write end of sound data
        output.push(VgmCommandType::End as u8);

        Ok(())
    }

    /// Write GD3 tag
    fn write_gd3_tag(&self, tag: &Gd3Tag, output: &mut Vec<u8>) -> MmlResult<()> {
        let gd3_offset = output.len() as u32;

        // GD3 tag starts with "Gd3 " identifier
        let mut gd3_data = vec![b'G', b'd', b'3', b' '];

        // Write version (0x00)
        gd3_data.push(0x00);

        // Write all fields as UTF-16LE with null terminators
        fn write_gd3_string(data: &mut Vec<u8>, s: &str) {
            for c in s.encode_utf16() {
                data.extend_from_slice(&c.to_le_bytes());
            }
            data.extend_from_slice(&0u16.to_le_bytes()); // Null terminator
        }

        write_gd3_string(&mut gd3_data, &tag.track_name_en);
        write_gd3_string(&mut gd3_data, &tag.track_name_jp);
        write_gd3_string(&mut gd3_data, &tag.game_name_en);
        write_gd3_string(&mut gd3_data, &tag.game_name_jp);
        write_gd3_string(&mut gd3_data, &tag.system_name_en);
        write_gd3_string(&mut gd3_data, &tag.system_name_jp);
        write_gd3_string(&mut gd3_data, &tag.author_en);
        write_gd3_string(&mut gd3_data, &tag.author_jp);
        write_gd3_string(&mut gd3_data, &tag.release_date);
        write_gd3_string(&mut gd3_data, &tag.converter);
        write_gd3_string(&mut gd3_data, &tag.notes);

        // Write GD3 data
        output.extend_from_slice(&gd3_data);

        Ok(())
    }

    /// Write PCM data block
    fn write_pcm_data_block(&self, pcm: &PcmData, output: &mut Vec<u8>) -> MmlResult<()> {
        // Write data block header
        output.push(VgmCommandType::DataBlock as u8);
        output.push(0x00); // Block type: PCM data
        output.extend_from_slice(&pcm.data.len().to_le_bytes());

        // Write PCM data
        output.extend_from_slice(&pcm.data);

        Ok(())
    }
}

impl CodeGenerator for VgmGenerator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Vgm
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}
