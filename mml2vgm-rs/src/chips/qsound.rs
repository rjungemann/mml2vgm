//! QSound (Capcom DL-1425) 16-voice stereo PCM chip emulation.
//!
//! ## Phases implemented
//! - Phase 1: Correctness — block type 0x8F, Q4.12 pitch, documented register layout, 24-bit banking
//! - Phase 2: Pan lookup — 33-entry sqrt equal-power table
//! - Phase 3: Echo/reverb — circular delay buffer, per-voice send levels, global feedback
//! - Phase 4: ADPCM — 3 one-shot Capcom IMA-ADPCM channels at ~8012 Hz
//!
//! ## VGM wiring
//! Opcode 0xC4: three-byte payload `[data_hi, addr, data_lo]`.
//! The full 16-bit word `(data_hi << 8) | data_lo` is written to DSP register `addr`.
//!
//! ## ROM
//! 16-bit signed samples.  Loaded via VGM data block type **0x8F** (not 0x88).
//! Effective address: `(bank << 16) | addr`.  ROM up to 8 MB (word-addressed).
//!
//! ## References
//! - MAME `qsound.cpp` / `qsoundhle.cpp`
//! - ValleyBell HLE: https://github.com/ValleyBell/qsound-hle
//! - Register gist: https://gist.github.com/superctr/fa2491fcf48b070459db30814eb7330f

use super::SoundChipEmulator;

// ── Constants ─────────────────────────────────────────────────────────────────

const NUM_PCM: usize = 16;
const NUM_ADPCM: usize = 3;
/// Byte size of the 24-bit-addressed 16-bit ROM (8 MB).
const ROM_BYTES: usize = 8 * 1024 * 1024;
/// Native output rate of the QSound DSP.
const NATIVE_RATE: f64 = 24038.0;
/// ADPCM playback rate (NATIVE_RATE / 3).
const ADPCM_RATE: f64 = NATIVE_RATE / 3.0;
/// Echo delay buffer: max length in stereo pairs (0x0FFF).
const ECHO_BUF_MAX: usize = 0x1000 * 2; // interleaved L+R
/// Default echo delay (mid-range).
const ECHO_DELAY_DEFAULT: usize = 0x0D00;

// ── Pan table ─────────────────────────────────────────────────────────────────

/// Pre-computed 33-entry sqrt equal-power pan table.
/// `table[i] = round((256 / sqrt(32)) * sqrt(i))`.
/// Index 0 = silent, 32 = maximum.
fn build_pan_table() -> [u8; 33] {
    let mut t = [0u8; 33];
    let scale = 256.0_f64 / 32.0_f64.sqrt();
    for (i, slot) in t.iter_mut().enumerate() {
        *slot = (scale * (i as f64).sqrt()).round().min(255.0) as u8;
    }
    t
}

/// Decode a QSound pan word to (left_gain, right_gain) in 0.0..1.0.
///
/// Pan word encoding:
/// - `0x0110..0x011F`: left half (n=0 hard-left, n=16 centre)
/// - `0x0120`        : exact centre
/// - `0x0130..0x013F`: right half (n=0 centre, n=16 hard-right)
/// - `0x0150+`       : linear mode — treat as centre for now
fn pan_gains(pan_word: u16, tbl: &[u8; 33]) -> (f32, f32) {
    let base = pan_word & 0xFFF0;
    let n = (pan_word & 0x000F) as usize; // 0..15
                                          // s = pan step: 0 = hard left, 16 = centre, 32 = hard right
    let s: usize = match base {
        0x0110 => n,      // left half: n=0 → s=0 (hard left), n=16 → s=16 (centre)
        0x0120 => 16,     // exact centre
        0x0130 => 16 + n, // right half: n=0 → s=16 (centre), n=16 → s=32 (hard right)
        _ => 16,          // fallback centre
    };
    let s = s.min(32);
    let l = tbl[32 - s] as f32 / 256.0;
    let r = tbl[s] as f32 / 256.0;
    (l, r)
}

// ── IMA-ADPCM tables ─────────────────────────────────────────────────────────

#[rustfmt::skip]
const IMA_STEP_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31,
    34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130,
    143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411,
    1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026,
    4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487,
    12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086,
    29794, 32767,
];
const IMA_INDEX_TABLE: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

/// Decode a 4-bit IMA-ADPCM nibble, updating predictor and step_index in place.
fn ima_decode(nibble: u8, predictor: &mut i32, step_index: &mut i32) -> i16 {
    let step = IMA_STEP_TABLE[*step_index as usize];
    let sign = nibble & 0x08;
    let magnitude = nibble & 0x07;
    let delta = step * (magnitude as i32 * 2 + 1) / 8;
    if sign != 0 {
        *predictor = (*predictor - delta).clamp(-32768, 32767);
    } else {
        *predictor = (*predictor + delta).clamp(-32768, 32767);
    }
    *step_index = (*step_index + IMA_INDEX_TABLE[nibble as usize]).clamp(0, 88);
    *predictor as i16
}

// ── Voice structs ─────────────────────────────────────────────────────────────

/// One PCM voice.
#[derive(Debug, Clone)]
struct QSoundVoice {
    /// Upper address bits 16–23.  Applied one tick after write (pending_bank).
    bank: u8,
    pending_bank: u8,
    /// Current 16-bit playback position (wrapping).
    addr: u16,
    /// Pitch in Q4.12 fixed-point.  0x1000 = 1× native speed.
    rate: u16,
    /// 12-bit fractional phase accumulator.  Initialise to 0x8000 (lower 12 bits = 0x000).
    phase: u16,
    /// Loop length: subtracted from addr when addr >= end_addr.
    loop_len: u16,
    /// End boundary.
    end_addr: u16,
    /// Linear amplitude 0x0000–0x7FFF.
    volume: u16,
    /// Pan word (decoded lazily via pan_gains).
    pan: u16,
    /// Whether this voice is producing audio.
    active: bool,
}

impl Default for QSoundVoice {
    fn default() -> Self {
        Self {
            bank: 0,
            pending_bank: 0,
            addr: 0,
            rate: 0,
            phase: 0x8000,
            loop_len: 0,
            end_addr: 0,
            volume: 0,
            pan: 0x0120, // default centre
            active: false,
        }
    }
}

/// One ADPCM voice (one-shot, ~8012 Hz).
#[derive(Debug, Clone, Default)]
struct AdpcmVoice {
    start: u16,
    end: u16,
    bank: u8,
    volume: u16,
    active: bool,
    /// Current byte offset from start, in bytes within the ROM bank.
    byte_pos: u32,
    /// Which nibble of the current byte to read next (false = low, true = high).
    nibble_hi: bool,
    predictor: i32,
    step_index: i32,
    /// Fractional accumulator for rate conversion from 8012 Hz to NATIVE_RATE.
    phase_acc: f64,
    /// Last decoded sample (held between native-rate ticks).
    last_sample: i16,
}

// ── Echo unit ─────────────────────────────────────────────────────────────────

struct EchoUnit {
    /// Circular stereo-interleaved delay buffer.  Capacity = ECHO_BUF_MAX.
    buffer: Vec<f32>,
    /// Write head (index into buffer, always even).
    write_pos: usize,
    /// Active delay length in stereo pairs.
    delay_len: usize,
    /// Recirculation coefficient, −1.0..1.0.
    feedback: f32,
    /// Per-voice send level, signed −1.0..1.0.
    send: [f32; NUM_PCM],
    /// One-pole low-pass state for the feedback path.
    last: f32,
}

impl Default for EchoUnit {
    fn default() -> Self {
        Self {
            buffer: vec![0.0f32; ECHO_BUF_MAX],
            write_pos: 0,
            delay_len: ECHO_DELAY_DEFAULT,
            feedback: 0.0,
            send: [0.0f32; NUM_PCM],
            last: 0.0,
        }
    }
}

impl EchoUnit {
    /// Process one native-rate tick.
    ///
    /// `voice_samples`: the dry mono sample for each PCM voice.
    /// Returns (echo_l, echo_r) to be added to dry output.
    fn process(&mut self, voice_samples: &[f32; NUM_PCM]) -> (f32, f32) {
        let buf_len = self.buffer.len();
        // Compute read position (delay_len stereo pairs behind write)
        let read_offset = 2 * self.delay_len;
        let read_pos = (self.write_pos + buf_len - read_offset) % buf_len;
        let echo_l = self.buffer[read_pos];
        let echo_r = self.buffer[(read_pos + 1) % buf_len];

        // One-pole low-pass on the feedback signal
        let avg = (echo_l + echo_r) * 0.5;
        let filtered = (avg + self.last) * 0.5;
        self.last = avg;
        let feedback_l = (echo_l + filtered * self.feedback * 4.0).clamp(-1.0, 1.0);
        let feedback_r = (echo_r + filtered * self.feedback * 4.0).clamp(-1.0, 1.0);

        // Echo send: sum of per-voice dry samples × send level
        let mut send_l = 0.0f32;
        let mut send_r = 0.0f32;
        for (v, &s) in voice_samples.iter().enumerate() {
            send_l += s * self.send[v];
            send_r += s * self.send[v];
        }

        // Write new buffer entry
        self.buffer[self.write_pos] = (send_l + feedback_l).clamp(-1.0, 1.0);
        self.buffer[(self.write_pos + 1) % buf_len] = (send_r + feedback_r).clamp(-1.0, 1.0);
        self.write_pos = (self.write_pos + 2) % buf_len;

        (echo_l, echo_r)
    }

    /// Update delay length from register 0xD9.  Clamped to valid hardware range.
    fn set_delay(&mut self, samples: u16) {
        self.delay_len = (samples as usize)
            .clamp(0x055A, 0x0FFF)
            .min(ECHO_BUF_MAX / 2 - 1);
    }
}

// ── QSound chip ───────────────────────────────────────────────────────────────

/// Q Sound.
pub struct QSound {
    clock_rate: u32,
    voices: [QSoundVoice; NUM_PCM],
    adpcm: [AdpcmVoice; NUM_ADPCM],
    rom: Vec<u8>,
    echo: EchoUnit,
    pan_table: [u8; 33],
    /// Resampling accumulator: fractional native ticks per output sample.
    resample_acc: f64,
}

impl QSound {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(4_000_000)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            voices: std::array::from_fn(|_| QSoundVoice::default()),
            adpcm: std::array::from_fn(|_| AdpcmVoice::default()),
            rom: vec![0u8; ROM_BYTES],
            echo: EchoUnit::default(),
            pan_table: build_pan_table(),
            resample_acc: 0.0,
        }
    }

    // ── ROM access ────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    fn read_word(&self, bank: u8, addr: u16) -> i16 {
        let byte_off = (bank as usize * 65536 + addr as usize) * 2;
        if byte_off + 1 < self.rom.len() {
            i16::from_le_bytes([self.rom[byte_off], self.rom[byte_off + 1]])
        } else {
            0
        }
    }

    #[allow(dead_code)]
    fn read_byte(&self, bank: u8, addr: u32) -> u8 {
        let byte_off = bank as usize * 65536 + addr as usize;
        if byte_off < self.rom.len() {
            self.rom[byte_off]
        } else {
            0
        }
    }

    // ── Register write ────────────────────────────────────────────────────────

    fn write_reg(&mut self, addr: u8, data: u16) {
        let addr = addr as usize;

        // PCM voice registers: 0x00–0x6F (16 voices × 7 regs, with 8 words allocated per voice)
        if addr < NUM_PCM * 8 {
            let ch = addr / 8;
            let reg = addr % 8;
            let v = &mut self.voices[ch];
            match reg {
                0 => v.pending_bank = (data >> 8) as u8, // upper byte = bank bits
                1 => {
                    v.addr = data;
                    if data != 0 {
                        v.active = true;
                    }
                }
                2 => v.rate = data,
                3 => v.phase = data,
                4 => v.loop_len = data,
                5 => {
                    v.end_addr = data;
                    // Writing 0 to end_addr deactivates
                    if data == 0 {
                        v.active = false;
                    }
                }
                6 => v.volume = data & 0x7FFF,
                _ => {}
            }
            return;
        }

        // Pan registers: 0x80–0x8F
        if (0x80..=0x8F).contains(&addr) {
            self.voices[addr - 0x80].pan = data;
            return;
        }

        // ADPCM pan: 0x90–0x92 (ignored for now — ADPCM always centre)
        if (0x90..=0x92).contains(&addr) {
            return;
        }

        // Echo feedback: 0x93
        if addr == 0x93 {
            self.echo.feedback = (data as i16) as f32 / 32768.0;
            return;
        }

        // ADPCM registers: 0xCA–0xD8
        if (0xCA..=0xD8).contains(&addr) {
            let offset = addr - 0xCA;
            if offset < 12 {
                // 3 channels × 4 regs each: start, end, bank, volume
                let ch = offset / 4;
                let reg = offset % 4;
                if ch < NUM_ADPCM {
                    match reg {
                        0 => self.adpcm[ch].start = data,
                        1 => self.adpcm[ch].end = data,
                        2 => self.adpcm[ch].bank = (data & 0xFF) as u8,
                        3 => self.adpcm[ch].volume = data & 0x7FFF,
                        _ => {}
                    }
                }
            } else {
                // offsets 12–14 (0xD6–0xD8): key-on for channels 0–2
                let ch = offset - 12;
                if ch < NUM_ADPCM && data != 0 {
                    let av = &mut self.adpcm[ch];
                    av.active = true;
                    av.byte_pos = 0;
                    av.nibble_hi = false;
                    av.predictor = 0;
                    av.step_index = 0;
                    av.phase_acc = 0.0;
                    av.last_sample = 0;
                }
            }
            return;
        }

        // Echo level per voice: 0xBA–0xC9
        if (0xBA..=0xC9).contains(&addr) {
            let idx = addr - 0xBA;
            if idx < NUM_PCM {
                self.echo.send[idx] = (data as i16) as f32 / 32768.0;
            }
            return;
        }

        // Echo delay: 0xD9
        if addr == 0xD9 {
            self.echo.set_delay(data);
        }
    }

    // ── One native-rate tick ──────────────────────────────────────────────────

    /// Clock all voices one native-rate tick.  Returns (left, right) output sample.
    fn clock_native(&mut self) -> (f32, f32) {
        let mut dry_l = 0.0f32;
        let mut dry_r = 0.0f32;
        let mut voice_samples = [0.0f32; NUM_PCM];
        let tbl = &self.pan_table;

        for (i, v) in self.voices.iter_mut().enumerate() {
            if !v.active {
                continue;
            }

            // Apply pending bank (one-tick latency)
            v.bank = v.pending_bank;

            // Read current sample (standalone fn avoids borrow conflict with self.voices iter_mut)
            let raw = read_word_static(v.bank, v.addr, &self.rom);

            let vol = v.volume.min(0x7FFF) as f32 / 0x7FFF_u32 as f32;
            let mono = raw as f32 / 32768.0 * vol;
            voice_samples[i] = mono;

            let (l_gain, r_gain) = pan_gains(v.pan, tbl);
            dry_l += mono * l_gain;
            dry_r += mono * r_gain;

            // Q4.12 pitch advance
            let inc = v.rate as u32;
            let frac_inc = inc & 0xFFF;
            let int_inc = inc >> 12;
            let new_frac = (v.phase as u32 & 0xFFF) + frac_inc;
            let carry = new_frac >> 12;
            v.phase = (v.phase & 0xF000) | ((new_frac & 0xFFF) as u16);
            v.addr = v.addr.wrapping_add((int_inc + carry) as u16);

            // End / loop check
            if v.end_addr > 0 && v.addr >= v.end_addr {
                if v.loop_len > 0 {
                    v.addr = v.addr.wrapping_sub(v.loop_len);
                } else {
                    v.active = false;
                }
            }
        }

        // ADPCM mix (always centre pan)
        for av in &mut self.adpcm {
            if !av.active {
                continue;
            }
            av.phase_acc += 1.0; // one native tick
                                 // Advance ADPCM decoder at ~8012 Hz (every NATIVE_RATE/ADPCM_RATE ticks)
            let ticks_per_adpcm = NATIVE_RATE / ADPCM_RATE; // ≈ 3.0
            if av.phase_acc >= ticks_per_adpcm {
                av.phase_acc -= ticks_per_adpcm;
                // Compute absolute byte offset in ROM
                let start_byte = av.start as u32;
                let end_byte = av.end as u32;
                if av.byte_pos >= end_byte.saturating_sub(start_byte) {
                    av.active = false;
                } else {
                    let rom_byte_off =
                        (av.bank as usize * 65536 + av.start as usize) + av.byte_pos as usize;
                    let byte_val = if rom_byte_off < self.rom.len() {
                        self.rom[rom_byte_off]
                    } else {
                        0
                    };
                    let nibble = if av.nibble_hi {
                        (byte_val >> 4) & 0x0F
                    } else {
                        byte_val & 0x0F
                    };
                    av.nibble_hi = !av.nibble_hi;
                    if !av.nibble_hi {
                        av.byte_pos += 1;
                    }
                    av.last_sample = ima_decode(nibble, &mut av.predictor, &mut av.step_index);
                }
            }

            let vol = av.volume.min(0x7FFF) as f32 / 0x7FFF_u32 as f32;
            let s = av.last_sample as f32 / 32768.0 * vol;
            dry_l += s * 0.5;
            dry_r += s * 0.5;
        }

        // Echo/reverb
        let (echo_l, echo_r) = self.echo.process(&voice_samples);

        let out_l = (dry_l + echo_l).clamp(-1.0, 1.0);
        let out_r = (dry_r + echo_r).clamp(-1.0, 1.0);
        (out_l, out_r)
    }
}

/// Free function for ROM word read (avoids borrow conflicts inside the loop).
#[inline(always)]
fn read_word_static(bank: u8, addr: u16, rom: &[u8]) -> i16 {
    let byte_off = (bank as usize * 65536 + addr as usize) * 2;
    if byte_off + 1 < rom.len() {
        i16::from_le_bytes([rom[byte_off], rom[byte_off + 1]])
    } else {
        0
    }
}

// ── SoundChipEmulator trait ───────────────────────────────────────────────────

impl SoundChipEmulator for QSound {
    fn name(&self) -> &'static str {
        "QSound"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        for v in &mut self.voices {
            *v = QSoundVoice::default();
        }
        for av in &mut self.adpcm {
            *av = AdpcmVoice::default();
        }
        self.echo = EchoUnit::default();
        self.resample_acc = 0.0;
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.write_reg(addr, data as u16);
    }

    /// VGM 0xC4: port=data_hi, addr=register_addr, data=data_lo → 16-bit write.
    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let word = ((port as u16) << 8) | (data as u16);
        self.write_reg(addr, word);
    }

    fn clock(&mut self) {}

    /// VGM data block type **0x8F** loads the PCM ROM.
    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x8F {
            let copy_len = data.len().min(self.rom.len());
            self.rom[..copy_len].copy_from_slice(&data[..copy_len]);
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        let out_rate = sample_rate as f64;
        let ticks_per_sample = NATIVE_RATE / out_rate;

        for frame in buffer.chunks_mut(2) {
            self.resample_acc += ticks_per_sample;
            let ticks = self.resample_acc as u32;
            let mut acc_l = 0.0f32;
            let mut acc_r = 0.0f32;
            for _ in 0..ticks {
                let (l, r) = self.clock_native();
                acc_l += l;
                acc_r += r;
            }
            if ticks > 0 {
                acc_l /= ticks as f32;
                acc_r /= ticks as f32;
            }
            self.resample_acc -= ticks as f64;

            let scale = 1.0 / NUM_PCM as f32;
            if frame.len() >= 2 {
                frame[0] = (acc_l * scale).clamp(-1.0, 1.0);
                frame[1] = (acc_r * scale).clamp(-1.0, 1.0);
            }
        }
    }
}

impl Default for QSound {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chip() -> QSound {
        QSound::new()
    }

    // ── Phase 1 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_qsound_new() {
        let chip = make_chip();
        assert_eq!(chip.name(), "QSound");
        assert_eq!(chip.clock_rate(), 4_000_000);
        assert!(chip.is_initialized());
    }

    #[test]
    fn test_qsound_silence_when_no_voices_active() {
        let mut chip = make_chip();
        let mut buf = [0.0f32; 16];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().all(|&s| s == 0.0),
            "idle chip must produce silence"
        );
    }

    #[test]
    fn test_qsound_load_pcm_data_accepts_0x8f() {
        let mut chip = make_chip();
        let data: Vec<u8> = (0u8..=255).collect();
        chip.load_pcm_data(0x8F, &data);
        assert_eq!(chip.rom[0], 0);
        assert_eq!(chip.rom[255], 255);
    }

    #[test]
    fn test_qsound_load_pcm_data_rejects_0x88() {
        let mut chip = make_chip();
        let data: Vec<u8> = vec![0xAB; 256];
        chip.load_pcm_data(0x88, &data); // must NOT overwrite
        assert_ne!(
            chip.rom[0], 0xAB,
            "block type 0x88 (Y8950) must not overwrite QSound ROM"
        );
    }

    #[test]
    fn test_qsound_load_pcm_data_rejects_wrong_type() {
        let mut chip = make_chip();
        let data = vec![0xFFu8; 256];
        chip.load_pcm_data(0x00, &data);
        assert_eq!(chip.rom[0], 0, "wrong block type must not overwrite ROM");
    }

    #[test]
    fn test_qsound_register_layout_voice0() {
        let mut chip = make_chip();
        // reg 0: bank (upper byte of data)
        chip.write_reg(0, 0x0300); // bank = 3
        assert_eq!(chip.voices[0].pending_bank, 3);
        // reg 2: rate
        chip.write_reg(2, 0x1000);
        assert_eq!(chip.voices[0].rate, 0x1000);
        // reg 4: loop_len
        chip.write_reg(4, 0x0100);
        assert_eq!(chip.voices[0].loop_len, 0x0100);
        // reg 5: end_addr
        chip.write_reg(5, 0x0200);
        assert_eq!(chip.voices[0].end_addr, 0x0200);
        // reg 6: volume
        chip.write_reg(6, 0x4000);
        assert_eq!(chip.voices[0].volume, 0x4000);
    }

    #[test]
    fn test_qsound_addr_write_activates_voice() {
        let mut chip = make_chip();
        assert!(!chip.voices[0].active);
        chip.write_reg(1, 0x0010); // write non-zero addr
        assert!(chip.voices[0].active);
    }

    #[test]
    fn test_qsound_pan_register_written() {
        let mut chip = make_chip();
        chip.write_reg(0x80, 0x0120); // centre
        assert_eq!(chip.voices[0].pan, 0x0120);
        chip.write_reg(0x81, 0x0110); // hard left
        assert_eq!(chip.voices[1].pan, 0x0110);
    }

    #[test]
    fn test_qsound_pitch_unity_rate() {
        // rate = 0x1000 → advance addr by 1 per native tick
        let mut v = QSoundVoice {
            active: true,
            rate: 0x1000,
            addr: 100,
            phase: 0x8000, // lower 12 bits = 0
            end_addr: 0x8000,
            volume: 0x7FFF,
            ..Default::default()
        };
        let inc = v.rate as u32;
        let frac_inc = inc & 0xFFF;
        let int_inc = inc >> 12;
        let new_frac = (v.phase as u32 & 0xFFF) + frac_inc;
        let carry = new_frac >> 12;
        v.phase = (v.phase & 0xF000) | ((new_frac & 0xFFF) as u16);
        v.addr = v.addr.wrapping_add((int_inc + carry) as u16);
        assert_eq!(
            v.addr, 101,
            "unity rate must advance addr by exactly 1 per tick"
        );
    }

    #[test]
    fn test_qsound_pitch_half_rate() {
        // rate = 0x0800 → advance addr by 0.5/tick → need 2 ticks for +1
        let mut v = QSoundVoice {
            active: true,
            rate: 0x0800,
            addr: 100,
            phase: 0x8000, // lower 12 bits = 0
            end_addr: 0x8000,
            volume: 0x7FFF,
            ..Default::default()
        };
        let advance_one_tick = |v: &mut QSoundVoice| {
            let inc = v.rate as u32;
            let frac_inc = inc & 0xFFF;
            let int_inc = inc >> 12;
            let new_frac = (v.phase as u32 & 0xFFF) + frac_inc;
            let carry = new_frac >> 12;
            v.phase = (v.phase & 0xF000) | ((new_frac & 0xFFF) as u16);
            v.addr = v.addr.wrapping_add((int_inc + carry) as u16);
        };
        advance_one_tick(&mut v);
        assert_eq!(v.addr, 100, "half rate must NOT advance addr after 1 tick");
        advance_one_tick(&mut v);
        assert_eq!(
            v.addr, 101,
            "half rate must advance addr by 1 after 2 ticks"
        );
    }

    #[test]
    fn test_qsound_key_on_produces_sound() {
        let mut chip = make_chip();

        // Fill ROM words 0..127 with a non-zero value
        for i in 0..128usize {
            let val: i16 = 0x4000;
            let idx = i * 2;
            chip.rom[idx] = val as u8;
            chip.rom[idx + 1] = (val >> 8) as u8;
        }

        // Voice 0 setup via write_port (VGM 0xC4 encoding: port=data_hi, addr=reg, data=data_lo)
        chip.write_port(0x00, 0x00, 0x00); // bank pending = 0
        chip.write_port(0x00, 0x01, 0x00); // addr = 0 (then we set it non-zero below)
        chip.write_port(0x10, 0x02, 0x00); // rate = 0x1000
        chip.write_port(0x80, 0x03, 0x00); // phase = 0x8000
        chip.write_port(0x00, 0x04, 0x40); // loop_len = 0x0040
        chip.write_port(0x00, 0x05, 0x80); // end_addr = 0x0080
        chip.write_port(0x7F, 0x06, 0xFF); // volume = 0x7FFF
        chip.write_port(0x01, 0x07, 0x20); // pan centre = 0x0120
                                           // Activate by writing addr
        chip.write_port(0x00, 0x01, 0x01); // addr = 1, triggers active=true

        let mut buf = vec![0.0f32; 256];
        chip.generate_samples(&mut buf, 44100);

        let any_nonzero = buf.iter().any(|&s| s != 0.0);
        assert!(
            any_nonzero,
            "key-on voice with ROM data must produce non-silent output"
        );
    }

    #[test]
    fn test_qsound_reset_silences() {
        let mut chip = make_chip();
        chip.voices[0].active = true;
        chip.voices[0].volume = 0x7FFF;
        chip.reset();
        assert!(!chip.voices[0].active);
        assert_eq!(chip.voices[0].volume, 0);
    }

    #[test]
    fn test_qsound_write_port_decodes_16bit() {
        let mut chip = make_chip();
        // Write 0xABCD to voice 0 rate register (addr 2)
        chip.write_port(0xAB, 0x02, 0xCD);
        assert_eq!(chip.voices[0].rate, 0xABCD);
    }

    // ── Phase 2 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_qsound_pan_table_endpoints() {
        let tbl = build_pan_table();
        assert_eq!(tbl[0], 0, "pan_table[0] must be 0 (silent)");
        // pan_table[32] = round(256/sqrt(32) * sqrt(32)) = round(256) = 256 → clamps to 255
        assert!(tbl[32] >= 254, "pan_table[32] must be near 255 (maximum)");
        // Monotonically increasing
        for i in 1..=32 {
            assert!(tbl[i] >= tbl[i - 1], "pan_table must be non-decreasing");
        }
    }

    #[test]
    fn test_qsound_pan_centre_equal_power() {
        let tbl = build_pan_table();
        let (l, r) = pan_gains(0x0120, &tbl); // exact centre
        assert!(
            (l - r).abs() < 1e-4,
            "centre pan must produce equal L/R gains, got l={l}, r={r}"
        );
        assert!(l > 0.0, "centre pan gains must be positive");
    }

    #[test]
    fn test_qsound_pan_hard_left() {
        let tbl = build_pan_table();
        // 0x0110 + 0 = hard left (n=0 → s=0)
        let (l, r) = pan_gains(0x0110, &tbl);
        assert!(l > 0.9, "hard-left left gain must be near 1.0, got {l}");
        assert!(r < 1e-4, "hard-left right gain must be near 0.0, got {r}");
    }

    #[test]
    fn test_qsound_pan_hard_right() {
        let tbl = build_pan_table();
        // 0x0130 + 16 = hard right (n=16 → s=32)
        let (l, r) = pan_gains(0x0130 | 0x000F, &tbl); // max n = 15
        assert!(
            r > l,
            "right half pan must produce more right gain than left"
        );
    }

    // ── Phase 3 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_qsound_echo_default_state() {
        let echo = EchoUnit::default();
        assert_eq!(echo.feedback, 0.0);
        assert!(echo.delay_len >= 0x055A && echo.delay_len <= 0x0FFF);
        assert!(echo.send.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_qsound_echo_delay_register() {
        let mut chip = make_chip();
        chip.write_port(0x07, 0xD9, 0x00); // reg 0xD9, value 0x0700
                                           // 0x0700 = 1792, within valid range 0x055A..0x0FFF
        assert_eq!(chip.echo.delay_len, 0x0700);
    }

    #[test]
    fn test_qsound_echo_feedback_register() {
        let mut chip = make_chip();
        chip.write_port(0x40, 0x93, 0x00); // echo_feedback = 0x4000 → 0.5
        let expected = 0x4000_u16 as i16 as f32 / 32768.0;
        assert!((chip.echo.feedback - expected).abs() < 1e-5);
    }

    #[test]
    fn test_qsound_echo_send_register() {
        let mut chip = make_chip();
        chip.write_port(0x20, 0xBA, 0x00); // echo_level[0] = 0x2000
        let expected = 0x2000_u16 as i16 as f32 / 32768.0;
        assert!((chip.echo.send[0] - expected).abs() < 1e-5);
    }

    #[test]
    fn test_qsound_echo_decay_produces_tail() {
        let mut chip = make_chip();

        // Set up a high-amplitude echo feedback so the tail is audible.
        // Use a very short delay (32 native samples) so the echo round-trip completes
        // well within the small render window (~139 native ticks at 44100 Hz output).
        chip.echo.feedback = 0.8;
        chip.echo.delay_len = 32; // bypass hardware minimum for testability
        chip.echo.send[0] = 0.5;

        // Write a single loud tone into ROM and trigger voice 0 briefly
        for i in 0..64usize {
            let val: i16 = 0x4000;
            let idx = i * 2;
            chip.rom[idx] = val as u8;
            chip.rom[idx + 1] = (val >> 8) as u8;
        }
        chip.write_port(0x10, 0x02, 0x00); // rate = unity
        chip.write_port(0x80, 0x03, 0x00); // phase
        chip.write_port(0x00, 0x05, 0x40); // end_addr = 64
        chip.write_port(0x7F, 0x06, 0xFF); // volume max
        chip.write_port(0x01, 0x07, 0x20); // pan centre (reg 7 = voice 0 pan? NO)
        chip.write_port(0x01, 0x20, 0x20); // pan: addr 0x80 + 0 = voice 0 pan centre
        chip.write_port(0x00, 0x01, 0x01); // activate voice 0

        // Render until voice stops playing
        let mut pre_buf = vec![0.0f32; 512];
        chip.generate_samples(&mut pre_buf, 44100);

        // Deactivate voice and render more
        chip.voices[0].active = false;
        let mut post_buf = vec![0.0f32; 512];
        chip.generate_samples(&mut post_buf, 44100);

        // With feedback=0.8, echo should still be ringing
        let any_nonzero = post_buf.iter().any(|&s| s.abs() > 1e-6);
        assert!(
            any_nonzero,
            "echo feedback=0.8 should produce a decaying tail after voice stops"
        );
    }

    #[test]
    fn test_qsound_echo_no_leakage_when_zeroed() {
        let mut chip = make_chip();
        // Default: feedback=0, send=[0; 16] → echo must not amplify anything
        let mut buf = vec![0.0f32; 256];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().all(|&s| s == 0.0),
            "zero feedback + zero send → echo must produce no output"
        );
    }

    // ── Phase 4 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_qsound_adpcm_key_on_activates() {
        let mut chip = make_chip();
        assert!(!chip.adpcm[0].active);
        chip.write_port(0x00, 0xD6, 0x01); // key-on ch0
        assert!(chip.adpcm[0].active);
    }

    #[test]
    fn test_qsound_adpcm_silence_on_empty_rom() {
        let mut chip = make_chip();
        // ROM is all zeros — IMA-ADPCM of silence produces silence
        chip.write_port(0x00, 0xCA, 0x00); // start = 0
        chip.write_port(0x00, 0xCB, 0x80); // end = 0x0080
        chip.write_port(0x7F, 0xCD, 0xFF); // volume max = 0x7FFF
        chip.write_port(0x00, 0xD6, 0x01); // key-on ch0
        let mut buf = vec![0.0f32; 256];
        chip.generate_samples(&mut buf, 44100);
        // All-zero ROM with IMA-ADPCM = all-zero nibbles = zero output
        assert!(
            buf.iter().all(|&s| s == 0.0),
            "ADPCM with zero ROM must produce silence"
        );
    }

    #[test]
    fn test_qsound_adpcm_registers_written() {
        let mut chip = make_chip();
        chip.write_port(0x01, 0xCA, 0x00); // ch0 start = 0x0100
        chip.write_port(0x02, 0xCB, 0x00); // ch0 end   = 0x0200
        chip.write_port(0x00, 0xCC, 0x01); // ch0 bank  = 1
        chip.write_port(0x40, 0xCD, 0x00); // ch0 vol   = 0x4000
        assert_eq!(chip.adpcm[0].start, 0x0100);
        assert_eq!(chip.adpcm[0].end, 0x0200);
        assert_eq!(chip.adpcm[0].bank, 0x01);
        assert_eq!(chip.adpcm[0].volume, 0x4000);
    }
}
