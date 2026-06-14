//! YM2413 (OPLL) sound chip emulation
//!
//! The YM2413 is a simplified FM synthesis chip providing 9 channels (or 6 melodic +
//! 5 rhythm in rhythm mode). Each channel uses 2 operators (modulator + carrier) with
//! built-in patch ROM containing 15 melodic instruments.
//!
//! VGM opcode 0x51: two-byte payload [addr, data].
//!
//! Register map:
//! - 0x00-0x07: Custom instrument definition (8 bytes)
//! - 0x0E: Rhythm mode (bit5=enable, bits[4:0]=key-on for rhythm voices)
//! - 0x10-0x18: Channel F-number LSB (ch 1-9, offset by 0x10)
//! - 0x20-0x28: Channel sustain/key-on/block/F-number MSB
//! - 0x30-0x38: Channel instrument number (high nibble) + volume (low nibble)

use super::SoundChipEmulator;

/// Built-in OPLL instrument ROM (instruments 0-35, 8 bytes each).
/// Format per entry: [MOD_AM_VIB_EG_KSR_MULT, CAR_AM_VIB_EG_KSR_MULT,
///                    MOD_KSL_TL, CAR_KSL_WF_FB, MOD_AR_DR, CAR_AR_DR, MOD_SL_RR, CAR_SL_RR]
const OPLL_ROM: [[u8; 8]; 16] = [
    // 0: Custom (placeholder — loaded from registers 0x00-0x07)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 1: Bell
    [0x71, 0x61, 0x1E, 0x17, 0xD1, 0x78, 0x00, 0x17],
    // 2: Guitar
    [0x13, 0x41, 0x16, 0x0E, 0xFD, 0xF4, 0x23, 0x23],
    // 3: Piano
    [0x33, 0x01, 0x96, 0x04, 0xFD, 0xFF, 0x23, 0x13],
    // 4: Flute
    [0x01, 0x61, 0x21, 0x19, 0xF1, 0xF0, 0x05, 0x17],
    // 5: Clarinet
    [0x22, 0x21, 0x1E, 0x06, 0xF0, 0x76, 0x08, 0x28],
    // 6: Oboe
    [0x31, 0x22, 0x16, 0x05, 0x90, 0x71, 0x00, 0x13],
    // 7: Trumpet
    [0x21, 0x61, 0x1D, 0x07, 0x82, 0x80, 0x10, 0x17],
    // 8: Organ
    [0x02, 0x10, 0x00, 0x06, 0xA3, 0xE2, 0x00, 0x01],
    // 9: Horn
    [0x20, 0x61, 0x1C, 0x08, 0xA2, 0xF4, 0x20, 0x08],
    // 10: Synthesizer
    [0x20, 0x01, 0x9F, 0x04, 0xC0, 0xFF, 0x01, 0x00],
    // 11: Harpsichord
    [0x20, 0x20, 0x10, 0x06, 0xD1, 0xE4, 0x00, 0x00],
    // 12: Vibraphone
    [0x01, 0x01, 0x1F, 0x00, 0xD1, 0xFF, 0x03, 0x07],
    // 13: Synthesizer Bass
    [0x01, 0x61, 0x14, 0x07, 0xD1, 0xF6, 0x01, 0x18],
    // 14: Acoustic Bass
    [0x22, 0x21, 0x08, 0x06, 0xF2, 0xF4, 0x20, 0x0B],
    // 15: Electric Guitar
    [0x02, 0x01, 0x06, 0x00, 0xA1, 0xFF, 0x00, 0x10],
];

/// Frequency multiplier table (MULT register 0-15 → ratio × 2 to avoid 0.5)
const FREQ_MULT2: [u32; 16] = [1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 20, 24, 24, 30, 30];

/// ADSR envelope state
#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvState {
    Attack,
    Decay,
    Sustain,
    Release,
    Off,
}

/// Single FM operator
#[derive(Debug, Clone)]
struct OpllOperator {
    phase_acc: u32,
    env_state: EnvState,
    env_level: u32, // 0 = max volume, 511 = silence (9-bit)
    feedback_acc: i32,
}

impl Default for OpllOperator {
    fn default() -> Self {
        Self {
            phase_acc: 0,
            env_state: EnvState::Off,
            env_level: 511,
            feedback_acc: 0,
        }
    }
}

/// Single OPLL channel (2 operators)
#[derive(Debug, Clone)]
struct OpllChannel {
    f_num: u16, // 9-bit F-number
    block: u8,  // 3-bit block (octave)
    key_on: bool,
    sustain: bool,
    instrument: u8, // 0-15 (0 = custom)
    volume: u8,     // 0-15 (carrier output attenuation)
    ops: [OpllOperator; 2],
}

impl Default for OpllChannel {
    fn default() -> Self {
        Self {
            f_num: 0,
            block: 0,
            key_on: false,
            sustain: false,
            instrument: 0,
            volume: 15,
            ops: [OpllOperator::default(), OpllOperator::default()],
        }
    }
}

/// YM2413 (OPLL) chip emulator
pub struct YM2413 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 0x40],
    custom_patch: [u8; 8],
    channels: [OpllChannel; 9],
    rhythm_mode: bool,
    rhythm_keys: u8,
    #[allow(dead_code)]
    accumulated_cycles: f32,
}

impl YM2413 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x40],
            custom_patch: [0; 8],
            channels: std::array::from_fn(|_| OpllChannel::default()),
            rhythm_mode: false,
            rhythm_keys: 0,
            accumulated_cycles: 0.0,
        }
    }

    /// Load instrument patch bytes for the given instrument number (0 = custom)
    fn get_patch(&self, inst: u8) -> [u8; 8] {
        if inst == 0 {
            self.custom_patch
        } else {
            OPLL_ROM[inst.min(15) as usize]
        }
    }

    /// Compute the phase increment for an operator given channel f_num/block and MULT index
    /// Phase accumulator is 18-bit (0..=2^18-1 = full cycle)
    /// Increment = f_num * 2^block * FREQ_MULT2[mult] / (clock_rate / sample_rate * 2^18)
    fn phase_inc(&self, f_num: u16, block: u8, mult_idx: u8) -> u32 {
        let f = f_num as u32;
        let b = block as u32;
        let m2 = FREQ_MULT2[mult_idx as usize & 0xF];
        // phase_inc = f_num * 2^(block-1) * mult / (clk / sample_rate * 2^18)
        // Simplified: (f_num * m2 * sample_rate * 2^(block+17)) / (clk * 2)
        // But to avoid overflow, compute in u64:
        let sr = self.sample_rate as u64;
        let clk = self.clock_rate as u64;
        // inc = f * 2^block * m2 * sr / (clk * 2) — but FREQ_MULT2 is already *2
        // So: f * 2^block * m2 * sr / (clk * 2) with m2 = real_mult * 2
        //   = f * 2^block * real_mult * sr / clk
        // Phase space 2^18:
        let inc = (f as u64 * (1u64 << b) * m2 as u64 * sr * (1u64 << 17)) / (clk * 262144);
        inc.min(0x3FFFF) as u32
    }

    /// Convert 9-bit envelope level (0=loud, 511=silent) to linear amplitude [0.0..1.0]
    fn env_to_amp(level: u32) -> f32 {
        if level >= 511 {
            return 0.0;
        }
        // Use exponential mapping: 6dB per 64 steps (3 bits per octave)
        let db = level as f32 * (48.0 / 511.0);
        10f32.powf(-db / 20.0)
    }

    /// Sine lookup with 18-bit phase (0..=2^18)
    fn sine(phase: u32) -> f32 {
        let t = (phase as f32) / (1u32 << 18) as f32;
        (t * std::f32::consts::TAU).sin()
    }

    /// Advance envelopes and phases, return stereo sample pair
    fn channel_sample(&mut self, ch: usize) -> f32 {
        let chan = &self.channels[ch];
        if !chan.key_on {
            return 0.0;
        }

        let patch = self.get_patch(chan.instrument);
        // Modulator parameters
        let mod_mult = patch[0] & 0x0F;
        let mod_tl_raw = patch[2] & 0x3F; // 6-bit TL (0=max, 63=min)
        let mod_fb = patch[3] & 0x07;
        // Carrier parameters
        let car_mult = patch[1] & 0x0F;
        // Carrier volume comes from channel volume register (4-bit, 0=loud, 15=quiet)
        // combined with carrier's AR/DR/SL/RR from patch

        let f_num = chan.f_num;
        let block = chan.block;

        let mod_inc = self.phase_inc(f_num, block, mod_mult);
        let car_inc = self.phase_inc(f_num, block, car_mult);

        // Advance phases
        self.channels[ch].ops[0].phase_acc =
            self.channels[ch].ops[0].phase_acc.wrapping_add(mod_inc) & 0x3FFFF;
        self.channels[ch].ops[1].phase_acc =
            self.channels[ch].ops[1].phase_acc.wrapping_add(car_inc) & 0x3FFFF;

        let mod_phase = self.channels[ch].ops[0].phase_acc;
        let car_phase = self.channels[ch].ops[1].phase_acc;

        // Modulator output with feedback
        let fb_shift = if mod_fb == 0 {
            return {
                let v = Self::sine(car_phase);
                v * 0.2
            };
        } else {
            mod_fb
        };
        let fb_amt = self.channels[ch].ops[0].feedback_acc;
        let mod_fb_phase = (mod_phase as i64 + (fb_amt >> (7 - fb_shift as i32)) as i64)
            .rem_euclid(1i64 << 18) as u32;
        let mod_out_raw = Self::sine(mod_fb_phase);
        let mod_tl_amp = 1.0 - mod_tl_raw as f32 / 63.0;
        let mod_env_amp = Self::env_to_amp(self.channels[ch].ops[0].env_level);
        let mod_out = mod_out_raw * mod_tl_amp * mod_env_amp;

        // Update feedback accumulator (average of last two outputs, scaled to phase units)
        let fb_scaled = (mod_out * (1i32 << 17) as f32) as i32;
        self.channels[ch].ops[0].feedback_acc = fb_scaled;

        // Carrier output, phase-modulated by modulator
        let mod_phase_shift = (mod_out * (1i32 << 17) as f32) as i64;
        let car_mod_phase = (car_phase as i64 + mod_phase_shift).rem_euclid(1i64 << 18) as u32;
        let car_out_raw = Self::sine(car_mod_phase);
        // Carrier volume: channel volume register (0=loud, 15=quiet)
        let car_vol_amp = 1.0 - self.channels[ch].volume as f32 / 15.0;
        let car_env_amp = Self::env_to_amp(self.channels[ch].ops[1].env_level);
        car_out_raw * car_vol_amp * car_env_amp * 0.2
    }

    fn update_envelope(&mut self, ch: usize) {
        let instrument = self.channels[ch].instrument;
        let key_on = self.channels[ch].key_on;
        let sustain = self.channels[ch].sustain;
        let patch = self.get_patch(instrument);

        for op in 0..2 {
            let ar = (patch[4 + op] >> 4) & 0x0F;
            let dr = patch[4 + op] & 0x0F;
            let sl = ((patch[6 + op] >> 4) & 0x0F) as u32 * 32;
            let eg_type = (patch[op] >> 5) & 1;
            let rr = if sustain && !key_on {
                5u32
            } else {
                (patch[6 + op] & 0x0F) as u32
            };

            let level = self.channels[ch].ops[op].env_level;
            let state = self.channels[ch].ops[op].env_state;

            let (new_level, new_state) = match state {
                EnvState::Attack => {
                    if ar == 15 {
                        (0, EnvState::Decay)
                    } else if ar > 0 {
                        let step = (15 - ar as u32).max(1);
                        let new_lvl = level.saturating_sub(step);
                        if new_lvl == 0 {
                            (0, EnvState::Decay)
                        } else {
                            (new_lvl, EnvState::Attack)
                        }
                    } else {
                        (level, EnvState::Attack)
                    }
                }
                EnvState::Decay => {
                    if dr > 0 {
                        let new_lvl = (level + dr as u32).min(511);
                        if new_lvl >= sl {
                            (sl, EnvState::Sustain)
                        } else {
                            (new_lvl, EnvState::Decay)
                        }
                    } else {
                        (sl, EnvState::Sustain)
                    }
                }
                EnvState::Sustain => {
                    if eg_type == 0 {
                        // Non-sustained: decay continues at RR
                        let new_lvl = if rr > 0 { (level + rr).min(511) } else { level };
                        let new_st = if !key_on {
                            EnvState::Release
                        } else {
                            EnvState::Sustain
                        };
                        (new_lvl, new_st)
                    } else {
                        let new_st = if !key_on {
                            EnvState::Release
                        } else {
                            EnvState::Sustain
                        };
                        (level, new_st)
                    }
                }
                EnvState::Release => {
                    let new_lvl = (level + rr * 2).min(511);
                    if new_lvl >= 511 {
                        (511, EnvState::Off)
                    } else {
                        (new_lvl, EnvState::Release)
                    }
                }
                EnvState::Off => (level, EnvState::Off),
            };

            self.channels[ch].ops[op].env_level = new_level;
            self.channels[ch].ops[op].env_state = new_state;
        }
    }

    fn key_on(&mut self, ch: usize) {
        self.channels[ch].ops[0].phase_acc = 0;
        self.channels[ch].ops[1].phase_acc = 0;
        self.channels[ch].ops[0].env_level = 511;
        self.channels[ch].ops[1].env_level = 511;
        self.channels[ch].ops[0].env_state = EnvState::Attack;
        self.channels[ch].ops[1].env_state = EnvState::Attack;
        self.channels[ch].ops[0].feedback_acc = 0;
    }

    fn key_off(&mut self, ch: usize) {
        if self.channels[ch].ops[0].env_state != EnvState::Off {
            self.channels[ch].ops[0].env_state = EnvState::Release;
        }
        if self.channels[ch].ops[1].env_state != EnvState::Off {
            self.channels[ch].ops[1].env_state = EnvState::Release;
        }
    }
}

impl SoundChipEmulator for YM2413 {
    fn name(&self) -> &'static str {
        "YM2413"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        let addr = addr & 0x3F;
        self.regs[addr as usize] = data;

        match addr {
            // Custom patch bytes
            0x00..=0x07 => {
                self.custom_patch[addr as usize] = data;
            }
            // Rhythm mode
            0x0E => {
                let new_rhythm = (data & 0x20) != 0;
                let new_keys = data & 0x1F;
                self.rhythm_mode = new_rhythm;
                self.rhythm_keys = new_keys;
            }
            // F-number LSB for channels 1-9 (addr 0x10-0x18 → ch 0-8)
            0x10..=0x18 => {
                let ch = (addr - 0x10) as usize;
                self.channels[ch].f_num = (self.channels[ch].f_num & 0x100) | data as u16;
            }
            // Key-on / Block / F-num MSB for channels 1-9 (addr 0x20-0x28 → ch 0-8)
            0x20..=0x28 => {
                let ch = (addr - 0x20) as usize;
                let new_key_on = (data & 0x10) != 0;
                let sustain = (data & 0x20) != 0;
                let block = (data >> 1) & 0x07;
                let f_num_hi = (data & 0x01) as u16;
                self.channels[ch].sustain = sustain;
                self.channels[ch].block = block;
                self.channels[ch].f_num = (self.channels[ch].f_num & 0x0FF) | (f_num_hi << 8);
                let was_on = self.channels[ch].key_on;
                self.channels[ch].key_on = new_key_on;
                if new_key_on && !was_on {
                    self.key_on(ch);
                } else if !new_key_on && was_on {
                    self.key_off(ch);
                }
            }
            // Instrument + volume for channels 1-9 (addr 0x30-0x38 → ch 0-8)
            0x30..=0x38 => {
                let ch = (addr - 0x30) as usize;
                self.channels[ch].instrument = (data >> 4) & 0x0F;
                self.channels[ch].volume = data & 0x0F;
            }
            _ => {}
        }
    }

    fn read(&self, addr: u8) -> u8 {
        self.regs[(addr & 0x3F) as usize]
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;
        // Envelope update rate: ~3579 kHz / 72 cycles per sample ≈ 49.7 kHz
        // We update envelopes every N samples
        const ENV_UPDATE_PERIOD: usize = 8;

        for (i, frame) in buffer.chunks_mut(2).enumerate() {
            // Periodic envelope update
            if i % ENV_UPDATE_PERIOD == 0 {
                for ch in 0..9 {
                    self.update_envelope(ch);
                }
            }

            let mut mixed = 0.0f32;
            for ch in 0..9 {
                mixed += self.channel_sample(ch);
            }
            let out = (mixed / 9.0).clamp(-1.0, 1.0);
            if frame.len() >= 2 {
                frame[0] = out;
                frame[1] = out;
            }
        }
    }
}

impl Default for YM2413 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2413_new() {
        let chip = YM2413::new();
        assert_eq!(chip.name(), "YM2413");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_ym2413_reset() {
        let mut chip = YM2413::new();
        chip.write(0x10, 0xAA);
        chip.reset();
        assert_eq!(chip.regs[0x10], 0x00);
    }

    #[test]
    fn test_ym2413_key_on_off() {
        let mut chip = YM2413::new();
        // Set f_num and block, key on channel 0
        chip.write(0x10, 0x61); // F-num LSB
        chip.write(0x20, 0b0001_0010); // key=1, block=1, f_num_hi=0
        assert!(chip.channels[0].key_on);
        assert_eq!(chip.channels[0].ops[0].env_state, EnvState::Attack);

        // Key off
        chip.write(0x20, 0b0000_0010);
        assert!(!chip.channels[0].key_on);
    }

    #[test]
    fn test_ym2413_instrument_volume() {
        let mut chip = YM2413::new();
        chip.write(0x30, 0x37); // instrument=3, volume=7
        assert_eq!(chip.channels[0].instrument, 3);
        assert_eq!(chip.channels[0].volume, 7);
    }

    #[test]
    fn test_ym2413_custom_patch() {
        let mut chip = YM2413::new();
        chip.write(0x00, 0x71);
        chip.write(0x01, 0x61);
        assert_eq!(chip.custom_patch[0], 0x71);
        assert_eq!(chip.custom_patch[1], 0x61);
    }

    #[test]
    fn test_ym2413_rhythm_mode() {
        let mut chip = YM2413::new();
        chip.write(0x0E, 0x20); // Enable rhythm mode
        assert!(chip.rhythm_mode);
        chip.write(0x0E, 0x1F); // key-on all rhythm, disable rhythm mode
        assert!(!chip.rhythm_mode);
        assert_eq!(chip.rhythm_keys, 0x1F);
    }

    #[test]
    fn test_ym2413_generate_samples_active() {
        let mut chip = YM2413::new();
        // Set instrument 1 (Bell)
        chip.write(0x30, 0x10); // instrument=1, volume=0 (loud)
                                // F-num for ~440 Hz at block 4: f_num ≈ 287
        chip.write(0x10, 0x1F); // f_num LSB
        chip.write(0x20, 0b0001_1001); // key=1, block=4, f_num_hi=1
        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        assert!(
            buffer.iter().any(|&s| s != 0.0),
            "active channel must produce output"
        );
    }

    #[test]
    fn test_ym2413_soundchip_trait() {
        let mut chip = YM2413::new();
        chip.reset();
        chip.write(0x10, 0x00);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
