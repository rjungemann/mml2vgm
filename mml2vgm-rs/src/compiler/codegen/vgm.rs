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
    /// Quantize/gate value
    quantize: u8,
    /// true = uppercase Q (proportional: note plays value/8 of duration)
    /// false = lowercase q (absolute: silence = value/48 of duration)
    quantize_proportional: bool,
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
            quantize: 0,
            quantize_proportional: false,
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
    /// When true, add_wait is a no-op (used during parallel part processing)
    suppress_waits: bool,
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
            suppress_waits: false,
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

    /// Write the standard YM2612 power-on reset sequence expected by every VGM.
    ///
    /// Sets LFO off, Timer off, DAC off, then for each of the 6 channels:
    /// key-off, mute all operators (TL=127), and (for active channels only)
    /// B4=0xC0 (stereo enable). B4 is interleaved per-channel, matching the
    /// C# reference ordering.
    fn ym2612_global_init(&mut self, num_active: u8) {
        let t = 0u64;
        // Global: LFO off, Timer off, DAC off
        self.ym2612_write_reg(0, 0x22, 0x00, t);
        self.ym2612_write_reg(0, 0x27, 0x00, t);
        self.ym2612_write_reg(0, 0x2B, 0x00, t);

        for abs_ch in 0u8..6 {
            let port = abs_ch / 3;
            let ch = abs_ch % 3;
            // Key-off
            let key_byte = ((port & 0x1) << 2) | (ch & 0x3);
            self.ym2612_write_reg(0, 0x28, key_byte, t);
            // Mute all 4 operators (TL=127) in slot write order S1,S2,S3,S4
            for &op_mul in &[0u8, 2, 1, 3] {
                let op_off = ch + op_mul * 4;
                self.ym2612_write_reg(port, 0x40 + op_off, 0x7F, t);
            }
            // Stereo enable immediately after TL mutes, only for active channels
            if abs_ch < num_active {
                self.ym2612_write_reg(port, 0xB4 + ch, 0xC0, t);
            }
        }
    }

    fn convert_ast_to_commands(&mut self, ast: &MmlAst) -> MmlResult<()> {
        // Count active YM2612 channels (parts that target YM2612, in sorted name order)
        let mut part_names: Vec<String> = ast.parts.keys().cloned().collect();
        part_names.sort();
        let num_ym2612_active = part_names
            .iter()
            .filter(|n| ast.parts[*n].chip.as_deref() == Some("YM2612"))
            .count() as u8;

        // Emit YM2612 global initialisation at time 0 if the song uses the chip
        if num_ym2612_active > 0 {
            self.ym2612_global_init(num_ym2612_active);
        }

        // Process global settings (tempo, etc.) — these don't emit chip writes
        let mut global_time: u64 = 0;
        for node in &ast.global_settings {
            self.process_node_global(node, &mut global_time)?;
        }

        // Process each part independently from time=0 (parallel/simultaneous playback).
        // During part processing, waits are suppressed — only write commands with
        // their absolute timestamps accumulate. After all parts are done, write
        // commands are sorted by time and waits are re-inserted between time-steps.
        let init_len = self.commands.len();
        let mut max_part_time: u64 = 0;

        self.suppress_waits = true;
        for name in &part_names {
            if let Some(part) = ast.parts.get(name) {
                let mut part_time: u64 = 0;
                self.process_part(part, &mut part_time)?;
                if part_time > max_part_time {
                    max_part_time = part_time;
                }
            }
        }
        self.suppress_waits = false;

        // Collect and sort write commands emitted by all parts
        let mut part_cmds: Vec<VgmCommand> = self.commands.drain(init_len..).collect();
        // Filter out any waits (shouldn't exist, but guard just in case)
        part_cmds.retain(|c| {
            !matches!(
                c.command_type,
                VgmCommandType::Wait | VgmCommandType::Wait1 | VgmCommandType::Wait2
            )
        });
        // Stable sort preserves relative order of simultaneous writes
        part_cmds.sort_by_key(|c| c.time);

        // Re-insert waits between time-steps
        let mut last_time: u64 = 0;
        for cmd in part_cmds {
            if cmd.time > last_time {
                self.add_wait((cmd.time - last_time) as u32, cmd.time);
                last_time = cmd.time;
            }
            self.commands.push(cmd);
        }

        // Add trailing wait from last register write to end of song
        if max_part_time > last_time {
            self.add_wait((max_part_time - last_time) as u32, max_part_time);
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
                    state.init_done = false;
                }
            }
            MmlNode::Quantize(q) => {
                state.quantize = q.value;
                state.quantize_proportional = q.proportional;
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
                    self.ym2612_write_init(state.ym2612_port, state.ym2612_ch, params.as_deref(), state.volume, *time);
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
                // Apply quantize/gate:
                // Q (proportional): note_on = floor(dur * value / 8), gap = dur - note_on
                // q (absolute):     note_on = floor(dur * (48-value) / 48), gap = floor(dur * value / 48)
                let (note_on_samples, gap) = if state.quantize == 0 {
                    (samples, 0u32)
                } else if state.quantize_proportional {
                    let note_on = (samples as u64 * state.quantize as u64 / 8) as u32;
                    (note_on, samples.saturating_sub(note_on))
                } else {
                    let gap = (samples as u64 * state.quantize as u64 / 48) as u32;
                    let note_on = (samples as u64 * (48 - state.quantize as u64) / 48) as u32;
                    (note_on, gap)
                };
                // Wait for note-on portion
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                // Key off at end of note-on
                self.ym2612_key_off(state, time);
                state.keyed_on = false;
                // Wait for gap (silence between notes)
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
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
    /// `params` is the flat parameter Vec from `FmInstrument.parameters`.
    /// M-type: 11 params/op → 46 total (alg at [44], fb at [45]).
    /// F-type:  9 params/op → 38 total (alg at [36], fb at [37]).
    fn ym2612_write_init(&mut self, port: u8, ch: u8, params: Option<&[u32]>, volume: u8, time: u64) {
        // Detect stride: 46+ params = M-type (stride 11, has AM+SSG-EG), else F-type (stride 9)
        let op_stride = params.map_or(11, |p| if p.len() >= 46 { 11 } else { 9 });
        let alg_idx = op_stride * 4;

        let (alg, fb) = if let Some(p) = params {
            let alg = p.get(alg_idx).copied().unwrap_or(7) as u8;
            let fb = p.get(alg_idx + 1).copied().unwrap_or(0) as u8;
            (alg, fb)
        } else {
            (7, 0)
        };

        // MML op index → hardware slot offset multiplier:
        //   MML op0 → S1 (offset 0), op1 → S2 (offset 8), op2 → S3 (offset 4), op3 → S4 (offset 12)
        let mml_to_hw: [u8; 4] = [0, 2, 1, 3];

        // Write 6 registers per op (DT/ML, KS/AR, AMS/DR, SR, SL/RR, SSG-EG) in MML op order.
        // TL is written separately below.
        for op_idx in 0..4usize {
            let op_off = ch + mml_to_hw[op_idx] * 4;
            let b = op_idx * op_stride;

            let (ar, dr, sr, rr, sl, ks, ml, dt, am, ssg_eg) = if let Some(p) = params {
                if p.len() > b + 8 {
                    let am  = if op_stride >= 11 { p.get(b + 9).copied().unwrap_or(0) as u8  } else { 0 };
                    let ssg = if op_stride >= 11 { p.get(b + 10).copied().unwrap_or(0) as u8 } else { 0 };
                    (p[b] as u8, p[b+1] as u8, p[b+2] as u8, p[b+3] as u8, p[b+4] as u8,
                     p[b+6] as u8, p[b+7] as u8, p[b+8] as u8, am, ssg)
                } else {
                    (31, 0, 0, 7, 0, 0, 1, 0, 0, 0)
                }
            } else {
                (31, 0, 0, 7, 0, 0, 1, 0, 0, 0)
            };

            self.ym2612_write_reg(port, 0x30 + op_off, ((dt & 0x7) << 4) | (ml & 0xF), time);
            self.ym2612_write_reg(port, 0x50 + op_off, ((ks & 0x3) << 6) | (ar & 0x1F), time);
            self.ym2612_write_reg(port, 0x60 + op_off, ((am & 0x1) << 7) | (dr & 0x1F), time);
            self.ym2612_write_reg(port, 0x70 + op_off, sr & 0x1F, time);
            self.ym2612_write_reg(port, 0x80 + op_off, ((sl & 0xF) << 4) | (rr & 0xF), time);
            self.ym2612_write_reg(port, 0x90 + op_off, ssg_eg & 0xF, time);
        }

        // 0xB0: feedback[5:3] | algorithm[2:0] — written after all op regs
        self.ym2612_write_reg(port, 0xB0 + ch, ((fb & 0x7) << 3) | (alg & 0x7), time);

        // TL writes in hardware register order (hw_mul 0,1,2,3 → offsets 0,4,8,12).
        // Map from hardware offset mul back to MML op: hw_mul→mml_op = [0,2,1,3]
        let hw_to_mml: [usize; 4] = [0, 2, 1, 3];
        for hw_mul in 0u8..4 {
            let op_off = ch + hw_mul * 4;
            let mml_op = hw_to_mml[hw_mul as usize];
            let b = mml_op * op_stride;
            let tl = params.and_then(|p| p.get(b + 5)).copied().unwrap_or(127) as u8;
            self.ym2612_write_reg(port, 0x40 + op_off, tl & 0x7F, time);
        }

        // Volume-adjusted TL for carrier operators only.
        // Carriers per algorithm (by hw_mul = slot-1, where S1=hw0,S2=hw2,S3=hw1,S4=hw3):
        //   ALG 0-3: S4 (hw_mul=3)
        //   ALG 4:   S2,S4 (hw_muls 2,3)
        //   ALG 5,6: S3,S2,S4 (hw_muls 1,2,3)
        //   ALG 7:   all (hw_muls 0,1,2,3)
        let carrier_hw_muls: &[u8] = match alg {
            0 | 1 | 2 | 3 => &[3],
            4             => &[2, 3],
            5 | 6         => &[1, 2, 3],
            _             => &[0, 1, 2, 3],
        };
        let vol_adjust = (127u16).saturating_sub(volume as u16);
        for &hw_mul in carrier_hw_muls {
            let op_off = ch + hw_mul * 4;
            let mml_op = hw_to_mml[hw_mul as usize];
            let b = mml_op * op_stride;
            let base_tl = params.and_then(|p| p.get(b + 5)).copied().unwrap_or(127) as u16;
            let adj_tl = (base_tl + vol_adjust).min(127) as u8;
            self.ym2612_write_reg(port, 0x40 + op_off, adj_tl, time);
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
    /// Uses the reference FNUM_YM2612.txt TYPE-C table, with block = octave - 1
    /// (matching the C# mml2vgm reference compiler exactly).
    fn midi_note_to_ym2612_freq(midi_note: u8) -> (u8, u16) {
        // From FNUM_YM2612.txt TYPE-C: C C# D D# E F F# G G# A A# B
        const FNUM_TABLE: [u16; 12] = [
            0x283, 0x2A8, 0x2D2, 0x2FD, 0x32A, 0x35B,
            0x38E, 0x3C4, 0x3FE, 0x43B, 0x47B, 0x4BF,
        ];
        let note_index = (midi_note % 12) as usize;
        // MIDI C4=60: octave = 60/12 - 1 = 4; block = octave - 1 = 3
        let octave = (midi_note / 12) as i32 - 1;
        let block = ((octave - 1).clamp(0, 7)) as u8;
        (block, FNUM_TABLE[note_index])
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
        if self.suppress_waits || samples == 0 {
            return;
        }
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
        // 0x34: VGM data offset (relative from 0x34).  Data starts at 0x100 → 0x100 − 0x34 = 0xCC.
        let data_offset_rel = self.header.data_offset.saturating_sub(0x34);
        output.extend_from_slice(&data_offset_rel.to_le_bytes());
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
