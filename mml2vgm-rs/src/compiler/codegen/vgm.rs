//! VGM Format Generator
//!
//! This module generates VGM (Video Game Music) format files from MML AST.

use super::{CodeGenerator, OutputFormat, VgmHeader};
use crate::compiler::ast::{MmlAst, MmlNode, OctaveShift};
use crate::{CompileOptions, MmlError, MmlResult, SoundChip};
use std::collections::HashMap;

/// VGM command types (values match the VGM specification)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VgmCommandType {
    Sn76489Write = 0x50,
    Ym2413Write = 0x51,
    Ym2612WritePort0 = 0x52,
    Ym2612WritePort1 = 0x53,
    Ym2151Write = 0x54,
    Ym2608WritePort0 = 0x55,
    Ym2608WritePort1 = 0x56,
    Ym2610WritePort0 = 0x57,
    Ym2610WritePort1 = 0x58,
    Ym3812Write = 0x5A,
    Ymf262WritePort0 = 0x5B,
    Ymf262WritePort1 = 0x5C,
    Rf5c164Write = 0x68,
    Wait = 0x61,
    Wait1 = 0x62,
    Wait2 = 0x63,
    End = 0x66,
    DataBlock = 0x67,
}

/// A single VGM command
#[derive(Debug, Clone)]
pub struct VgmCommand {
    pub command_type: VgmCommandType,
    pub data: Vec<u8>,
    pub time: u64,
}

/// PCM data for embedding in VGM
#[derive(Debug, Clone)]
pub struct PcmData {
    pub data: Vec<u8>,
    pub start_offset: u32,
}

/// GD3 tag for metadata
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

/// Per-part state during VGM code generation
struct PartCodegenState {
    /// Chip name for this part (e.g. "YM2612", "SN76489")
    chip: Option<String>,
    /// YM2612 port (0 = channels 0-2, 1 = channels 3-5)
    ym2612_port: u8,
    /// YM2612 channel within the port (0-2)
    ym2612_ch: u8,
    /// Tempo in BPM
    tempo: u32,
    /// Current octave (0-8)
    octave: u8,
    /// Current default note length denominator (4 = quarter note)
    length: u32,
    /// Current volume (0-127)
    volume: u8,
    /// Selected FM instrument number
    instrument_num: Option<u32>,
    /// Whether the operator registers have been written for this channel
    init_done: bool,
    /// Whether a key-on is in effect
    keyed_on: bool,
}

impl PartCodegenState {
    fn new(chip: Option<String>, ym2612_port: u8, ym2612_ch: u8) -> Self {
        Self {
            chip,
            ym2612_port,
            ym2612_ch,
            tempo: 120,
            octave: 4,
            length: 4,
            volume: 127,
            instrument_num: None,
            init_done: false,
            keyed_on: false,
        }
    }
}

/// VGM file generator
pub struct VgmGenerator {
    header: VgmHeader,
    commands: Vec<VgmCommand>,
    chips: Vec<SoundChip>,
    pcm_data: Vec<PcmData>,
    gd3_tag: Option<Gd3Tag>,
    /// FM instrument flat-parameter tables indexed by instrument number
    fm_instruments: HashMap<u32, Vec<u32>>,
    /// Next YM2612 absolute channel to allocate (0-5)
    next_ym2612_channel: u8,
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
            fm_instruments: HashMap::new(),
            next_ym2612_channel: 0,
        };

        generator.header.version = 0x00000171;
        generator.extract_chips(ast, options);

        // Store FM instrument parameters from the AST
        for (num, inst) in &ast.fm_instruments {
            generator.fm_instruments.insert(*num, inst.parameters.clone());
        }

        generator.convert_ast_to_commands(ast)?;
        generator.build_gd3_tag(ast);
        generator.calculate_header();

        Ok(generator)
    }

    fn extract_chips(&mut self, ast: &MmlAst, options: &CompileOptions) {
        if let Some(ref chips) = options.target_chips {
            for chip in chips {
                if !self.chips.contains(chip) {
                    self.chips.push(*chip);
                }
            }
        }

        for part in ast.parts.values() {
            if let Some(ref chip_str) = part.chip {
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

        if self.chips.is_empty() {
            self.chips = vec![SoundChip::YM2612, SoundChip::SN76489];
        }

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

    fn convert_ast_to_commands(&mut self, ast: &MmlAst) -> MmlResult<()> {
        let mut current_time: u64 = 0;

        for node in &ast.global_settings {
            self.process_node_global(node, &mut current_time)?;
        }

        // Sort parts by name for deterministic output order
        let mut part_names: Vec<String> = ast.parts.keys().cloned().collect();
        part_names.sort();

        for name in &part_names {
            if let Some(part) = ast.parts.get(name) {
                self.process_part(part, &mut current_time)?;
            }
        }

        Ok(())
    }

    fn process_part(
        &mut self,
        part: &crate::compiler::ast::PartDefinition,
        time: &mut u64,
    ) -> MmlResult<()> {
        let chip = part.chip.clone();

        // Allocate YM2612 channel
        let (ym2612_port, ym2612_ch) = if chip.as_deref() == Some("YM2612") {
            let ch = self.next_ym2612_channel.min(5);
            self.next_ym2612_channel = self.next_ym2612_channel.saturating_add(1);
            (ch / 3, ch % 3)
        } else {
            (0, 0)
        };

        let mut state = PartCodegenState::new(chip, ym2612_port, ym2612_ch);

        for node in &part.commands {
            self.process_node_with_state(node, &mut state, time)?;
        }

        // Key off any note still ringing at end of part
        if state.keyed_on {
            self.ym2612_key_off(&state, time);
        }

        Ok(())
    }

    /// Process MML nodes that appear in global context (outside any part)
    fn process_node_global(&mut self, node: &MmlNode, time: &mut u64) -> MmlResult<()> {
        match node {
            MmlNode::Rest(rest) => {
                let samples = self.note_duration_to_samples(rest.duration, rest.dotted, 120, 4);
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            MmlNode::Note(note) => {
                // Global notes (no chip assigned) emit SN76489 writes using the default channel
                let mut state = PartCodegenState::new(None, 0, 0);
                state.octave = note.octave;
                self.process_psg_note(note, &state, time);
                let dur = note.duration.unwrap_or(4);
                let samples = self.note_duration_to_samples(dur, note.dotted, 120, 4);
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            _ => {}
        }
        Ok(())
    }

    fn process_node_with_state(
        &mut self,
        node: &MmlNode,
        state: &mut PartCodegenState,
        time: &mut u64,
    ) -> MmlResult<()> {
        match node {
            MmlNode::Tempo(t) => {
                state.tempo = t.bpm;
            }
            MmlNode::Octave(o) => {
                state.octave = o.number;
            }
            MmlNode::OctaveShift(shift) => match shift {
                OctaveShift::Up => state.octave = (state.octave + 1).min(8),
                OctaveShift::Down => state.octave = state.octave.saturating_sub(1),
            },
            MmlNode::Length(l) => {
                state.length = l.value.max(1);
            }
            MmlNode::Volume(v) => {
                state.volume = v.level;
            }
            MmlNode::InstrumentSelection(sel) => {
                let new_num = sel.number as u32;
                if state.instrument_num != Some(new_num) {
                    state.instrument_num = Some(new_num);
                    state.init_done = false; // re-init when instrument changes
                }
            }
            MmlNode::Note(note) => {
                self.process_chip_note(note, state, time)?;
            }
            MmlNode::Rest(rest) => {
                if state.keyed_on && state.chip.as_deref() == Some("YM2612") {
                    self.ym2612_key_off(state, time);
                    state.keyed_on = false;
                }
                let samples =
                    self.note_duration_to_samples(rest.duration, rest.dotted, state.tempo, state.length);
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            MmlNode::Loop(loop_node) => {
                for _ in 0..loop_node.count {
                    for inner in &loop_node.body {
                        self.process_node_with_state(inner, state, time)?;
                    }
                }
            }
            MmlNode::Bar => {}
            _ => {}
        }
        Ok(())
    }

    fn process_chip_note(
        &mut self,
        note: &crate::compiler::ast::Note,
        state: &mut PartCodegenState,
        time: &mut u64,
    ) -> MmlResult<()> {
        let midi = note.midi_note();
        let dur = note.duration.unwrap_or(state.length);
        let dotted = note.dotted;
        let samples = self.note_duration_to_samples(dur, dotted, state.tempo, state.length);

        match state.chip.as_deref() {
            Some("YM2612") => {
                // Write operator registers if needed
                if !state.init_done {
                    let params = state.instrument_num
                        .and_then(|n| self.fm_instruments.get(&n).cloned());
                    self.ym2612_write_init(state.ym2612_port, state.ym2612_ch, params.as_deref(), *time);
                    state.init_done = true;
                }
                // Key off any previous note
                if state.keyed_on {
                    self.ym2612_key_off(state, time);
                    state.keyed_on = false;
                }
                // Frequency
                let (block, f_num) = Self::midi_note_to_ym2612_freq(midi);
                self.ym2612_write_freq(state.ym2612_port, state.ym2612_ch, block, f_num, *time);
                // Key on
                self.ym2612_key_on(state, time);
                state.keyed_on = true;
                // Wait for note duration
                *time += samples as u64;
                self.add_wait(samples, *time);
                // Key off after the note
                self.ym2612_key_off(state, time);
                state.keyed_on = false;
            }
            Some("SN76489") | None => {
                self.process_psg_note(note, state, time);
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            _ => {
                // Unknown chip: just advance time
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
        }
        Ok(())
    }

    fn process_psg_note(
        &mut self,
        note: &crate::compiler::ast::Note,
        state: &PartCodegenState,
        time: &u64,
    ) {
        let midi = note.midi_note();
        let (_, tone) = self.midi_note_to_psg_freq(midi);

        // Write tone register for channel 0 (simplified)
        let tone_low = (tone & 0x0F) as u8;
        let tone_high = ((tone >> 4) & 0x3F) as u8;
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Sn76489Write,
            data: vec![0x80 | tone_low],
            time: *time,
        });
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Sn76489Write,
            data: vec![tone_high],
            time: *time,
        });

        // Volume: map 0-127 → PSG attenuation 15-0 (inverted, 0=loud 15=silent)
        let atten = (15u8).saturating_sub((state.volume >> 3) & 0x0F);
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Sn76489Write,
            data: vec![0x90 | (atten & 0x0F)],
            time: *time,
        });
    }

    // ── YM2612 helpers ──────────────────────────────────────────────────────────

    /// Write one YM2612 register (port 0 or 1)
    fn ym2612_write_reg(&mut self, port: u8, reg: u8, val: u8, time: u64) {
        let cmd_type = if port == 0 {
            VgmCommandType::Ym2612WritePort0
        } else {
            VgmCommandType::Ym2612WritePort1
        };
        self.commands.push(VgmCommand {
            command_type: cmd_type,
            data: vec![reg, val],
            time,
        });
    }

    /// Write all YM2612 operator and channel setup registers.
    ///
    /// `params` is the flat parameter Vec from `FmInstrument.parameters`:
    ///   ops 0-3 at indices [op*11 .. op*11+11], then ALG at [44], FB at [45].
    fn ym2612_write_init(&mut self, port: u8, ch: u8, params: Option<&[u32]>, time: u64) {
        // Unpack ALG/FB
        let (alg, fb) = if let Some(p) = params {
            let alg = p.get(44).copied().unwrap_or(7) as u8;
            let fb = p.get(45).copied().unwrap_or(0) as u8;
            (alg, fb)
        } else {
            (7, 0)
        };

        // 0xB0+ch: feedback [5:3] | algorithm [2:0]
        self.ym2612_write_reg(port, 0xB0 + ch, ((fb & 0x7) << 3) | (alg & 0x7), time);
        // 0xB4+ch: LR=both (0xC0), AMS=0, FMS=0
        self.ym2612_write_reg(port, 0xB4 + ch, 0xC0, time);

        // Operator registers: 4 operators, each at ch + op*4 offset within the register bank
        for op in 0..4u8 {
            let op_off = ch + op * 4;

            let (ar, dr, sr, rr, sl, tl, ks, ml, dt, am, ssg_eg) = if let Some(p) = params {
                let b = (op as usize) * 11;
                if p.len() > b + 10 {
                    (
                        p[b] as u8,
                        p[b + 1] as u8,
                        p[b + 2] as u8,
                        p[b + 3] as u8,
                        p[b + 4] as u8,
                        p[b + 5] as u8,
                        p[b + 6] as u8,
                        p[b + 7] as u8,
                        p[b + 8] as u8,
                        p[b + 9] as u8,
                        p[b + 10] as u8,
                    )
                } else {
                    (31, 0, 0, 7, 0, 0, 0, 1, 0, 0, 0)
                }
            } else {
                (31, 0, 0, 7, 0, 0, 0, 1, 0, 0, 0)
            };

            // 0x30: DT[6:4] | ML[3:0]
            self.ym2612_write_reg(port, 0x30 + op_off, ((dt & 0x7) << 4) | (ml & 0xF), time);
            // 0x40: TL[6:0]
            self.ym2612_write_reg(port, 0x40 + op_off, tl & 0x7F, time);
            // 0x50: KS[7:6] | AR[4:0]
            self.ym2612_write_reg(port, 0x50 + op_off, ((ks & 0x3) << 6) | (ar & 0x1F), time);
            // 0x60: AM[7] | DR[4:0]
            self.ym2612_write_reg(port, 0x60 + op_off, ((am & 0x1) << 7) | (dr & 0x1F), time);
            // 0x70: SR[4:0]
            self.ym2612_write_reg(port, 0x70 + op_off, sr & 0x1F, time);
            // 0x80: SL[7:4] | RR[3:0]
            self.ym2612_write_reg(port, 0x80 + op_off, ((sl & 0xF) << 4) | (rr & 0xF), time);
            // 0x90: SSG-EG[3:0]
            self.ym2612_write_reg(port, 0x90 + op_off, ssg_eg & 0xF, time);
        }
    }

    /// Write YM2612 F-number and block for a channel
    fn ym2612_write_freq(&mut self, port: u8, ch: u8, block: u8, f_num: u16, time: u64) {
        // 0xA4+ch: block[5:3] | F-num MSB [2:0]  (write FIRST per spec)
        let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
        self.ym2612_write_reg(port, 0xA4 + ch, msb, time);
        // 0xA0+ch: F-num LSB [7:0]
        self.ym2612_write_reg(port, 0xA0 + ch, (f_num & 0xFF) as u8, time);
    }

    fn ym2612_key_on(&mut self, state: &PartCodegenState, time: &u64) {
        // Register 0x28, port 0: key-on byte = (all-ops 0xF0) | (port<<2) | ch
        let key_byte = 0xF0u8 | ((state.ym2612_port & 0x1) << 2) | (state.ym2612_ch & 0x3);
        self.ym2612_write_reg(0, 0x28, key_byte, *time);
    }

    fn ym2612_key_off(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = 0x00u8 | ((state.ym2612_port & 0x1) << 2) | (state.ym2612_ch & 0x3);
        self.ym2612_write_reg(0, 0x28, key_byte, *time);
    }

    /// Compute YM2612 block and F-number from a MIDI note number.
    ///
    /// Formula: F-number = freq × 2^(20 − block) × 144 / ym2612_clock
    /// Block is chosen to keep F-number in [0, 2047].
    fn midi_note_to_ym2612_freq(midi_note: u8) -> (u8, u16) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        // YM2612 master clock from VgmHeader default = 7,670,453 Hz → /144 ≈ 53,267 Hz
        let f_clk = 7_670_453.0_f64 / 144.0;

        let mut block = 0u8;
        let mut f_num = (freq * ((1u32 << 20) as f64) / f_clk).round() as u32;

        while f_num > 2047 && block < 7 {
            block += 1;
            let shift = 20u32.saturating_sub(block as u32);
            f_num = (freq * ((1u32 << shift) as f64) / f_clk).round() as u32;
        }

        (block, f_num.min(2047) as u16)
    }

    /// Convert a note duration to 44100 Hz sample count.
    ///
    /// `duration` is the MML length denominator (1=whole, 2=half, 4=quarter …).
    fn note_duration_to_samples(&self, duration: u32, dotted: bool, bpm: u32, _default: u32) -> u32 {
        let bpm = bpm.max(1);
        let duration = duration.max(1);
        // Samples for one whole note at this BPM
        let whole_note = 44100u64 * 4 * 60 / bpm as u64;
        let base = (whole_note / duration as u64) as u32;
        if dotted { base + base / 2 } else { base }.max(1)
    }

    /// Emit a Wait command with the correct 16-bit LE format, splitting if > 65535.
    fn add_wait(&mut self, mut samples: u32, time: u64) {
        while samples > 0 {
            let chunk = samples.min(65535) as u16;
            self.commands.push(VgmCommand {
                command_type: VgmCommandType::Wait,
                data: chunk.to_le_bytes().to_vec(),
                time,
            });
            samples -= chunk as u32;
        }
    }

    /// Convert MIDI note to SN76489 tone divider
    fn midi_note_to_psg_freq(&self, midi_note: u8) -> (u8, u16) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let clock = self.header.sn76489_clock as f64;
        let divider = (clock / (32.0 * freq)).round() as u32;
        let tone_val = divider.min(1023) as u16;
        let block = (divider / 1024).min(7) as u8;
        (block, tone_val)
    }

    fn build_gd3_tag(&mut self, ast: &MmlAst) {
        let mut tag = Gd3Tag::default();
        for (key, value) in &ast.metadata {
            match key.to_lowercase().as_str() {
                "title" | "name" | "titlename" => tag.track_name_en = value.clone(),
                "author" | "composer" => tag.author_en = value.clone(),
                "game" => tag.game_name_en = value.clone(),
                "system" | "systemname" => tag.system_name_en = value.clone(),
                "date" => tag.release_date = value.clone(),
                "converter" => tag.converter = value.clone(),
                "notes" | "comment" => tag.notes = value.clone(),
                _ => {}
            }
        }
        self.gd3_tag = Some(tag);
    }

    fn calculate_header(&mut self) {
        let mut total_samples: u32 = 0;
        for cmd in &self.commands {
            match cmd.command_type {
                VgmCommandType::Wait => {
                    if cmd.data.len() >= 2 {
                        let s = u16::from_le_bytes([cmd.data[0], cmd.data[1]]) as u32;
                        total_samples += s;
                    }
                }
                VgmCommandType::Wait1 => total_samples += 735,
                VgmCommandType::Wait2 => total_samples += 882,
                _ => {}
            }
        }
        self.header.total_samples = total_samples;
        self.header.data_offset = 0x100;
    }

    /// Generate the VGM file binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();
        self.write_header(&mut output)?;
        self.write_commands(&mut output)?;
        if let Some(ref tag) = self.gd3_tag {
            self.write_gd3_tag(tag, &mut output)?;
        }
        for pcm in &self.pcm_data {
            self.write_pcm_data_block(pcm, &mut output)?;
        }
        // Patch EOF offset at bytes 4-7 (relative from offset 4)
        let eof_offset = output.len().saturating_sub(4) as u32;
        if output.len() >= 8 {
            output[4..8].copy_from_slice(&eof_offset.to_le_bytes());
        }
        Ok(output)
    }

    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        output.extend_from_slice(&self.header.ident);
        output.extend_from_slice(&0u32.to_le_bytes()); // EOF offset placeholder
        output.extend_from_slice(&self.header.version.to_le_bytes());
        output.extend_from_slice(&self.header.sn76489_clock.to_le_bytes());
        output.extend_from_slice(&self.header.ym2413_clock.to_le_bytes());
        output.extend_from_slice(&self.header.gd3_offset.to_le_bytes());
        output.extend_from_slice(&self.header.total_samples.to_le_bytes());
        output.extend_from_slice(&self.header.loop_offset.to_le_bytes());
        output.extend_from_slice(&self.header.loop_samples.to_le_bytes());
        output.extend_from_slice(&self.header.rate.to_le_bytes());
        output.extend_from_slice(&self.header.sn76489_feedback.to_le_bytes());
        output.push(self.header.sn76489_shift_register_width);
        output.push(self.header.sn76489_flags);
        output.extend_from_slice(&self.header.ym2612_clock.to_le_bytes());
        output.extend_from_slice(&self.header.ym2151_clock.to_le_bytes());
        while output.len() < 0x100 {
            output.push(0);
        }
        Ok(())
    }

    fn write_commands(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        for cmd in &self.commands {
            output.push(cmd.command_type as u8);
            output.extend_from_slice(&cmd.data);
        }
        output.push(VgmCommandType::End as u8);
        Ok(())
    }

    fn write_gd3_tag(&self, tag: &Gd3Tag, output: &mut Vec<u8>) -> MmlResult<()> {
        let mut gd3_data = vec![b'G', b'd', b'3', b' '];
        gd3_data.push(0x00);

        fn write_gd3_string(data: &mut Vec<u8>, s: &str) {
            for c in s.encode_utf16() {
                data.extend_from_slice(&c.to_le_bytes());
            }
            data.extend_from_slice(&0u16.to_le_bytes());
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

        output.extend_from_slice(&gd3_data);
        Ok(())
    }

    fn write_pcm_data_block(&self, pcm: &PcmData, output: &mut Vec<u8>) -> MmlResult<()> {
        output.push(VgmCommandType::DataBlock as u8);
        output.push(0x00);
        output.extend_from_slice(&pcm.data.len().to_le_bytes());
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
