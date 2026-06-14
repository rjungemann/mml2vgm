//! Real-time live note playback via direct chip register writes.
//!
//! [`LivePlayer`] owns chip emulators, parses MML source to extract instrument
//! definitions and channel assignments, and exposes `note_on`/`note_off`
//! methods that write registers directly—bypassing the compile → render pipeline.
//!
//! Typical use:
//! ```no_run
//! use mml2vgm::live_player::LivePlayer;
//! let source = "{ PartYM2612 = A }";
//! let mut player = LivePlayer::from_source(source, 44100).unwrap();
//! player.note_on("A1", 60, 100); // C4 on channel A1
//! let mut buf = vec![0.0f32; 2048];
//! player.generate_samples(&mut buf, 44100); // called from the audio callback
//! player.note_off("A1");
//! ```

use crate::chips::SoundChipEmulator;
use crate::SoundChip;
use std::collections::HashMap;

// ── instrument parsing ───────────────────────────────────────────────────────

/// A parsed FM instrument definition extracted from `'@` lines in MML source.
#[derive(Debug, Clone)]
pub struct FmInstrumentDef {
    /// Instrument number (matches `@N` in channel commands, e.g. `@1`).
    pub number: u8,
    /// Flat parameter array: `[op0×11, op1×11, op2×11, op3×11, alg, fb]` = 46 values.
    ///
    /// Layout per operator (11 values):
    /// `[AR, DR, SR, RR, SL, TL, KS, ML, DT, AM, SSG-EG]`
    pub params: Vec<u32>,
}

/// Parse all FM instruments from MML source text.
///
/// Handles `'@ M NNN` / `'@ F NNN` declarations followed by four operator rows
/// then an alg/fb row.
pub fn parse_instruments(source: &str) -> Vec<FmInstrumentDef> {
    let mut instruments = Vec::new();
    let mut current_num: Option<u8> = None;
    let mut op_params: Vec<Vec<u32>> = Vec::new();
    let mut alg_fb: Option<(u32, u32)> = None;

    for line in source.lines() {
        let t = line.trim();
        if !t.starts_with("'@") {
            continue;
        }
        let rest = t[2..].trim();

        // New FM instrument declaration: '@ M NNN or '@ F NNN
        if rest.starts_with('M') || rest.starts_with('F') {
            flush_instrument(
                &mut instruments,
                &mut current_num,
                &mut op_params,
                &mut alg_fb,
            );
            let num_str = rest.split_whitespace().nth(1).unwrap_or("0");
            let num = num_str.parse::<u8>().unwrap_or(0);
            current_num = Some(num);
            op_params.clear();
            alg_fb = None;
            continue;
        }

        // PCM or other non-FM types — stop accumulating
        if rest.starts_with('P') || rest.starts_with('S') || rest.starts_with('D') {
            flush_instrument(
                &mut instruments,
                &mut current_num,
                &mut op_params,
                &mut alg_fb,
            );
            current_num = None;
            continue;
        }

        // Parameter row: '@ N1,N2,...
        if current_num.is_some() && rest.contains(',') {
            let values: Vec<u32> = rest
                .split(',')
                .map(|s| s.trim().parse::<u32>().unwrap_or(0))
                .collect();

            if values.len() >= 9 && op_params.len() < 4 {
                // Operator row (9–11 values): normalize to exactly 11 values
                let mut op = values[..values.len().min(11)].to_vec();
                while op.len() < 11 {
                    op.push(0);
                }
                op_params.push(op);
            } else if values.len() == 2 {
                // Alg/FB row — complete the instrument
                alg_fb = Some((values[0], values[1]));
                flush_instrument(
                    &mut instruments,
                    &mut current_num,
                    &mut op_params,
                    &mut alg_fb,
                );
            }
        }
    }

    // Flush any trailing instrument
    flush_instrument(
        &mut instruments,
        &mut current_num,
        &mut op_params,
        &mut alg_fb,
    );
    instruments
}

fn flush_instrument(
    instruments: &mut Vec<FmInstrumentDef>,
    current_num: &mut Option<u8>,
    op_params: &mut Vec<Vec<u32>>,
    alg_fb: &mut Option<(u32, u32)>,
) {
    if let (Some(num), Some((alg, fb))) = (*current_num, *alg_fb) {
        if op_params.len() == 4 {
            let mut params = Vec::with_capacity(46);
            for op in op_params.iter() {
                params.extend_from_slice(op);
            }
            params.push(alg);
            params.push(fb);
            instruments.push(FmInstrumentDef {
                number: num,
                params,
            });
        }
    }
    *current_num = None;
    *alg_fb = None;
    op_params.clear();
}

// ── channel mapping ──────────────────────────────────────────────────────────

/// Return a map from channel-letter to chip name, extracted from `Part...` metadata.
///
/// E.g. `PartYM2612 = A` in the header → `{ 'A': "YM2612" }`.
fn parse_chip_map(source: &str) -> HashMap<char, String> {
    let mut map = HashMap::new();
    let mut in_header = false;

    for line in source.lines() {
        let t = line.trim();
        if t.starts_with("'{") || t == "{" {
            in_header = true;
            continue;
        }
        if in_header && (t == "}" || t.starts_with("'}")) {
            in_header = false;
            continue;
        }
        if !in_header || !t.starts_with("Part") {
            continue;
        }
        if let Some(eq_pos) = t.find('=') {
            let key = t[..eq_pos].trim();
            let val = t[eq_pos + 1..].trim();
            let chip_name = if key.starts_with("PartYM2612") {
                "YM2612"
            } else if key.starts_with("PartSN76489") {
                "SN76489"
            } else if key.starts_with("PartYM2151") {
                "YM2151"
            } else if key.starts_with("PartYM2608") {
                "YM2608"
            } else {
                continue;
            };
            for ch in val.chars() {
                if ch.is_ascii_uppercase() {
                    map.insert(ch, chip_name.to_string());
                }
            }
        }
    }
    map
}

/// Collect unique channel IDs (e.g. `["A1", "A2", "B1"]`) in order of first appearance.
fn collect_channel_ids(source: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        let b = t.as_bytes();
        if b.len() >= 3 && b[0] == b'\'' && b[1].is_ascii_uppercase() && b[2].is_ascii_digit() {
            let id = format!("{}{}", b[1] as char, b[2] as char);
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

/// Scan channel command lines for the first `@N` instrument selection.
/// Returns `None` if no instrument assignment is found (e.g. for PSG channels).
fn find_channel_instrument(source: &str, channel_id: &str) -> Option<u8> {
    let prefix = format!("'{}", channel_id);
    for line in source.lines() {
        let t = line.trim();
        if !t.starts_with(&prefix) {
            continue;
        }
        let rest = &t[prefix.len()..];
        let bytes = rest.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'@' {
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i > start {
                    if let Ok(n) = rest[start..i].parse::<u8>() {
                        return Some(n);
                    }
                }
            } else {
                i += 1;
            }
        }
    }
    None
}

// ── chip kind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChipKind {
    YM2612,
    SN76489,
    Other,
}

impl ChipKind {
    fn from_name(name: &str) -> Self {
        match name {
            "YM2612" | "YM2612X" | "YM2612X2" => ChipKind::YM2612,
            "SN76489" | "SN76489X2" => ChipKind::SN76489,
            _ => ChipKind::Other,
        }
    }

    fn matches_sound_chip(self, sc: &SoundChip) -> bool {
        match self {
            ChipKind::YM2612 | ChipKind::Other => {
                matches!(
                    sc,
                    SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2
                )
            }
            ChipKind::SN76489 => {
                matches!(sc, SoundChip::SN76489 | SoundChip::SN76489X2)
            }
        }
    }
}

// ── channel state ────────────────────────────────────────────────────────────

struct LiveChannelState {
    id: String,
    chip_kind: ChipKind,
    /// YM2612 port (0 = channels 0-2, 1 = channels 3-5).
    port: u8,
    /// Hardware channel within the port (0-2).
    hw_ch: u8,
    /// FM instrument for this channel (None for PSG or unassigned channels).
    instr: Option<FmInstrumentDef>,
    /// True after operator parameters have been written to the chip once.
    init_done: bool,
    /// Currently sounding MIDI note, if any.
    active_note: Option<u8>,
}

// ── frequency helpers ────────────────────────────────────────────────────────

/// Compute the YM2612 (block, F-number) pair for a MIDI note.
///
/// Uses the same FNUM_TABLE as the VGM code generator (TYPE-C table from
/// `FNUM_YM2612.txt`).
pub fn midi_to_ym2612_freq(midi_note: u8) -> (u8, u16) {
    const FNUM_TABLE: [u16; 12] = [
        0x283, 0x2A8, 0x2D2, 0x2FD, 0x32A, 0x35B, 0x38E, 0x3C4, 0x3FE, 0x43B, 0x47B, 0x4BF,
    ];
    let note_index = (midi_note % 12) as usize;
    let octave = (midi_note / 12) as i32 - 1;
    let block = ((octave - 1).clamp(0, 7)) as u8;
    (block, FNUM_TABLE[note_index])
}

/// Compute the SN76489 10-bit tone divider for a MIDI note.
pub fn midi_to_sn76489_divider(midi_note: u8, clock: u32) -> u16 {
    let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
    let divider = (clock as f64 / (32.0 * freq)).round() as u16;
    divider.min(1023)
}

// ── register-write helpers ───────────────────────────────────────────────────

/// Write YM2612 F-type operator registers (all except TL) to `chip`.
///
/// TL is omitted here and must be written separately via [`write_ym2612_tl`]
/// so that volume (velocity) adjustment is applied to carrier operators.
fn write_ym2612_op_params(chip: &mut dyn SoundChipEmulator, port: u8, ch: u8, params: &[u32]) {
    let op_stride = if params.len() >= 46 { 11usize } else { 9usize };
    let alg_idx = op_stride * 4;
    let alg = params.get(alg_idx).copied().unwrap_or(7) as u8;
    let fb = params.get(alg_idx + 1).copied().unwrap_or(0) as u8;

    // MML operator order → hardware operator offset (matches VGM generator's mml_to_hw swap)
    let mml_to_hw: [u8; 4] = [0, 2, 1, 3];
    for (op_idx, &hw) in mml_to_hw.iter().enumerate() {
        let op_off = ch + hw * 4;
        let b = op_idx * op_stride;
        if params.len() > b + 8 {
            // Layout: [AR, DR, SR, RR, SL, TL, KS, ML, DT, AM?, SSG-EG?]
            //          b+0 b+1 b+2 b+3 b+4 b+5 b+6 b+7 b+8 b+9   b+10
            let ar = params[b] as u8;
            let dr = params[b + 1] as u8;
            let sr = params[b + 2] as u8;
            let rr = params[b + 3] as u8;
            let sl = params[b + 4] as u8;
            // b+5 = TL — skipped here, written by write_ym2612_tl
            let ks = params[b + 6] as u8;
            let ml = params[b + 7] as u8;
            let dt = params[b + 8] as u8;
            let am = if op_stride >= 11 {
                params.get(b + 9).copied().unwrap_or(0) as u8
            } else {
                0
            };
            let ssg = if op_stride >= 11 {
                params.get(b + 10).copied().unwrap_or(0) as u8
            } else {
                0
            };

            chip.write_port(port, 0x30 + op_off, ((dt & 0x7) << 4) | (ml & 0xF));
            chip.write_port(port, 0x50 + op_off, ((ks & 0x3) << 6) | (ar & 0x1F));
            chip.write_port(port, 0x60 + op_off, ((am & 0x1) << 7) | (dr & 0x1F));
            chip.write_port(port, 0x70 + op_off, sr & 0x1F);
            chip.write_port(port, 0x80 + op_off, ((sl & 0xF) << 4) | (rr & 0xF));
            chip.write_port(port, 0x90 + op_off, ssg & 0xF);
        }
    }
    chip.write_port(port, 0xB0 + ch, ((fb & 0x7) << 3) | (alg & 0x7));
}

/// Write YM2612 TL (total level) for all 4 operators, adjusting carrier TL by velocity.
///
/// Carrier TL = `voice_tl + (127 - velocity)` clamped to 127.
/// Modulator TL = `voice_tl` unchanged.
fn write_ym2612_tl(
    chip: &mut dyn SoundChipEmulator,
    port: u8,
    ch: u8,
    params: &[u32],
    velocity: u8,
) {
    let vol = velocity as u32;
    let op_stride = if params.len() >= 46 { 11usize } else { 9usize };
    let alg = params.get(op_stride * 4).copied().unwrap_or(7) as u8;

    let carrier: [bool; 4] = match alg {
        4 => [false, true, false, true],
        5 | 6 => [false, true, true, true],
        7 => [true, true, true, true],
        _ => [false, false, false, true], // alg 0–3
    };

    let mml_to_hw: [usize; 4] = [0, 2, 1, 3];
    for mml_op in 0..4usize {
        let hw_op = mml_to_hw[mml_op];
        let op_off = ch as usize + hw_op * 4;
        let voice_tl = params.get(mml_op * op_stride + 5).copied().unwrap_or(0);
        let tl = if carrier[mml_op] {
            (voice_tl + (127 - vol)).min(127) as u8
        } else {
            voice_tl as u8
        };
        chip.write_port(port, 0x40 + op_off as u8, tl & 0x7F);
    }
}

// ── LivePlayer ───────────────────────────────────────────────────────────────

/// Real-time sound chip player that writes registers directly into chip emulators.
///
/// Build with [`LivePlayer::from_source`] after a successful compile, then forward
/// `note_on`/`note_off` events from the GUI thread while [`generate_samples`] runs
/// continuously in the audio callback.
pub struct LivePlayer {
    chips: Vec<(SoundChip, Box<dyn SoundChipEmulator>)>,
    channel_states: Vec<LiveChannelState>,
    #[allow(dead_code)]
    sample_rate: u32,
}

// SAFETY: All concrete chip emulator types (YM2612, SN76489, …) hold only
// plain numeric data and contain no thread-local state or raw pointers, so
// they are safe to send across threads. The Mutex in LiveAudioEngine ensures
// only one thread accesses LivePlayer at a time.
unsafe impl Send for LivePlayer {}

impl LivePlayer {
    /// Build a `LivePlayer` from a MML source string.
    ///
    /// Parses instrument definitions and channel-chip assignments from the source,
    /// instantiates chip emulators, and performs global init writes.
    ///
    /// Returns `Err` if no channels are found in the source.
    pub fn from_source(source: &str, sample_rate: u32) -> Result<Self, String> {
        let instruments = parse_instruments(source);
        let chip_map = parse_chip_map(source);
        let channel_ids = collect_channel_ids(source);

        if channel_ids.is_empty() {
            return Err("no channels found in MML source".to_string());
        }

        let mut ym2612_ch_counter = 0u8;
        let mut sn76489_ch_counter = 0u8;
        let mut need_ym2612 = false;
        let mut need_sn76489 = false;

        let mut channel_states: Vec<LiveChannelState> = Vec::new();

        for id in &channel_ids {
            if id.len() < 2 {
                continue;
            }
            let letter = id.chars().next().unwrap();
            let chip_name = chip_map
                .get(&letter)
                .map(|s| s.as_str())
                .unwrap_or("YM2612");
            let chip_kind = ChipKind::from_name(chip_name);

            let state = match chip_kind {
                ChipKind::SN76489 => {
                    let hw_ch = sn76489_ch_counter.min(2);
                    sn76489_ch_counter += 1;
                    need_sn76489 = true;
                    LiveChannelState {
                        id: id.clone(),
                        chip_kind: ChipKind::SN76489,
                        port: 0,
                        hw_ch,
                        instr: None, // PSG channels don't use FM instruments
                        init_done: false,
                        active_note: None,
                    }
                }
                ChipKind::YM2612 | ChipKind::Other => {
                    let abs_ch = ym2612_ch_counter;
                    ym2612_ch_counter += 1;
                    let port = abs_ch / 3;
                    let hw_ch = abs_ch % 3;
                    need_ym2612 = true;
                    let instr_num = find_channel_instrument(source, id);
                    let instr =
                        instr_num.and_then(|n| instruments.iter().find(|i| i.number == n).cloned());
                    LiveChannelState {
                        id: id.clone(),
                        chip_kind: ChipKind::YM2612,
                        port,
                        hw_ch,
                        instr,
                        init_done: false,
                        active_note: None,
                    }
                }
            };
            channel_states.push(state);
        }

        let mut chips: Vec<(SoundChip, Box<dyn SoundChipEmulator>)> = Vec::new();

        if need_ym2612 {
            let mut ym2612 =
                Box::new(crate::chips::ym2612::YM2612::new()) as Box<dyn SoundChipEmulator>;
            // Global init: LFO off, timer off, DAC off
            ym2612.write_port(0, 0x22, 0x00);
            ym2612.write_port(0, 0x27, 0x00);
            ym2612.write_port(0, 0x2B, 0x00);
            // Key-off all channels, mute all operators, enable stereo output
            for abs_ch in 0u8..6 {
                let port = abs_ch / 3;
                let ch = abs_ch % 3;
                let key_byte = ((port & 0x1) << 2) | (ch & 0x3);
                ym2612.write_port(0, 0x28, key_byte); // key-off
                for &op_mul in &[0u8, 2, 1, 3] {
                    let op_off = ch + op_mul * 4;
                    ym2612.write_port(port, 0x40 + op_off, 0x7F); // TL = 127 (mute)
                }
                ym2612.write_port(port, 0xB4 + ch, 0xC0); // L+R stereo output
            }
            chips.push((SoundChip::YM2612, ym2612));
        }

        if need_sn76489 {
            let sn76489 =
                Box::new(crate::chips::sn76489::SN76489::new()) as Box<dyn SoundChipEmulator>;
            chips.push((SoundChip::SN76489, sn76489));
        }

        Ok(Self {
            chips,
            channel_states,
            sample_rate,
        })
    }

    /// Trigger note-on for the named channel (e.g. `"A1"`, `"B2"`).
    ///
    /// `velocity` is 0–127 and scales the carrier TL for FM channels.
    pub fn note_on(&mut self, channel: &str, midi_note: u8, velocity: u8) {
        let Some(idx) = self.channel_states.iter().position(|s| s.id == channel) else {
            return;
        };

        // Extract state before mutably borrowing chips
        let chip_kind = self.channel_states[idx].chip_kind;
        let port = self.channel_states[idx].port;
        let hw_ch = self.channel_states[idx].hw_ch;
        let init_done = self.channel_states[idx].init_done;
        let instr = self.channel_states[idx].instr.clone();
        let had_note = self.channel_states[idx].active_note.is_some();

        let Some(chip_idx) = self
            .chips
            .iter()
            .position(|(k, _)| chip_kind.matches_sound_chip(k))
        else {
            return;
        };

        match chip_kind {
            ChipKind::YM2612 | ChipKind::Other => {
                let chip = &mut *self.chips[chip_idx].1;

                // Write operator params (DT/ML, KS/AR, AM/DR, SR, SL/RR, SSG-EG, FB/ALG)
                // on the first note for this channel.
                if !init_done {
                    if let Some(ref p) = instr {
                        write_ym2612_op_params(chip, port, hw_ch, &p.params);
                    }
                    self.channel_states[idx].init_done = true;
                }

                // Write TL for all operators (volume-scaled carriers)
                if let Some(ref p) = instr {
                    write_ym2612_tl(chip, port, hw_ch, &p.params, velocity);
                }

                // Key-off the previous note if still sounding
                if had_note {
                    let key_byte = ((port & 0x1) << 2) | (hw_ch & 0x3);
                    chip.write_port(0, 0x28, key_byte);
                }

                // Write frequency: MSB (block + F-num high) first, then LSB per OPN2 spec
                let (block, f_num) = midi_to_ym2612_freq(midi_note);
                let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
                chip.write_port(port, 0xA4 + hw_ch, msb);
                chip.write_port(port, 0xA0 + hw_ch, (f_num & 0xFF) as u8);

                // Key-on: all 4 operator slots
                let key_byte = 0xF0u8 | ((port & 0x1) << 2) | (hw_ch & 0x3);
                chip.write_port(0, 0x28, key_byte);

                self.channel_states[idx].active_note = Some(midi_note);
            }
            ChipKind::SN76489 => {
                let psg_ch = hw_ch;
                let chip = &mut *self.chips[chip_idx].1;

                // Write tone frequency (LATCH byte then DATA byte)
                // SN76489 clock = 3,579,545 Hz (NTSC default)
                let divider = midi_to_sn76489_divider(midi_note, 3_579_545);
                // Latch: 1_CC_0_DDDD (CC=channel, 0=tone, DDDD=low 4 bits of divider)
                chip.write(0x80 | ((psg_ch & 3) << 5) | (divider as u8 & 0x0F), 0);
                // Data: 0_X_HHHHHH (HHHHHH = bits 9:4 of divider)
                chip.write((divider >> 4) as u8 & 0x3F, 0);

                // Volume: 1_CC_1_AAAA (CC=channel, 1=volume, AAAA=attenuation 0-15)
                // velocity 0-127 → attenuation 15-0 (0=loudest, 15=silent)
                let attenuation = 15u8.saturating_sub(velocity / 8);
                chip.write(0x90 | ((psg_ch & 3) << 5) | (attenuation & 0xF), 0);

                self.channel_states[idx].active_note = Some(midi_note);
            }
        }
    }

    /// Trigger note-off for the named channel.
    pub fn note_off(&mut self, channel: &str) {
        let Some(idx) = self.channel_states.iter().position(|s| s.id == channel) else {
            return;
        };

        let chip_kind = self.channel_states[idx].chip_kind;
        let port = self.channel_states[idx].port;
        let hw_ch = self.channel_states[idx].hw_ch;

        let Some(chip_idx) = self
            .chips
            .iter()
            .position(|(k, _)| chip_kind.matches_sound_chip(k))
        else {
            return;
        };

        match chip_kind {
            ChipKind::YM2612 | ChipKind::Other => {
                let chip = &mut *self.chips[chip_idx].1;
                let key_byte = ((port & 0x1) << 2) | (hw_ch & 0x3);
                chip.write_port(0, 0x28, key_byte);
            }
            ChipKind::SN76489 => {
                let psg_ch = hw_ch;
                let chip = &mut *self.chips[chip_idx].1;
                // Maximum attenuation = 0x0F → silence
                chip.write(0x90 | ((psg_ch & 3) << 5) | 0x0F, 0);
            }
        }

        self.channel_states[idx].active_note = None;
    }

    /// Fill `buffer` with interleaved stereo f32 samples.
    ///
    /// `buffer` must have an even length (`frames * 2`). Mixes output from all
    /// active chip emulators, then soft-clips to `[-1.0, 1.0]`.
    pub fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        buffer.fill(0.0);
        if self.chips.is_empty() {
            return;
        }
        let mut tmp = vec![0.0f32; buffer.len()];
        for (_, chip) in &mut self.chips {
            tmp.fill(0.0);
            chip.generate_samples(&mut tmp, sample_rate);
            for (b, t) in buffer.iter_mut().zip(tmp.iter()) {
                *b += t;
            }
        }
        for s in buffer.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }
    }
}
