//! YM2151 (OPM) sound chip emulation
//!
//! The YM2151 provides 8 FM channels, each with 4 operators (M1, M2, C1, C2).
//! Operators connect via one of 8 algorithms.
//!
//! OPM register map:
//! - 0x01: LFO reset
//! - 0x08: Key on/off: bits[2:0]=ch, bits[6:4]=operators (M1=bit3, C1=bit4, M2=bit5, C2=bit6)
//! - 0x18: LFO frequency
//! - 0x20-0x27: Channel L/R/FB/CON (left=bit7, right=bit6, FB=bits5:3, CON=bits2:0)
//! - 0x28-0x2F: Channel KC (key code: bits[6:4]=OCT, bits[3:0]=NOTE)
//! - 0x30-0x37: Channel KF (key fraction, bits[7:2])
//! - 0x40-0x5F: Operator DT1/MULT (DT1=bits[6:4], MULT=bits[3:0])
//! - 0x60-0x7F: Operator TL (bits[6:0])
//! - 0x80-0x9F: Operator KS/AR
//! - 0xA0-0xBF: Operator AM/D1R
//! - 0xC0-0xDF: Operator DT2/D2R
//! - 0xE0-0xFF: Operator D1L/RR
//! Operator layout: base + op*8 + ch (op: 0=M1, 1=M2, 2=C1, 3=C2)

use super::SoundChipEmulator;
use std::f32::consts::PI;

const FREQ_MULT: [f32; 16] = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 10.0, 12.0, 12.0, 15.0, 15.0];

/// OPM note-code-to-semitone mapping (KC bits 3:0); values 3/7/11/15 are unused (treat as invalid)
const KC_NOTE_SEMITONE: [i32; 16] = [0, 1, 2, 2, 3, 4, 5, 5, 6, 7, 8, 8, 9, 10, 11, 11];

/// FM operator (M1, M2, C1, or C2)
#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase_acc: f32,
    total_level: u8,
    mult: u8,
    key_on: bool,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self { phase_acc: 0.0, total_level: 0, mult: 1, key_on: false }
    }
}

/// FM channel (8 in total for OPM)
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    operators: [FmOperator; 4],
    kc: u8,
    kf: u8,
    algorithm: u8,
    feedback: u8,
    left_enable: bool,
    right_enable: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            operators: [FmOperator::default(); 4],
            kc: 0, kf: 0, algorithm: 0, feedback: 0,
            left_enable: true, right_enable: true,
        }
    }
}

/// YM2151 chip emulator with 8 FM channels
pub struct YM2151 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 0x100],
    channels: [FmChannel; 8],
    lfo_counter: u16,
    lfo_enabled: bool,
    lfo_frequency: u8,
    accumulated_cycles: f32,
    op_feedback: [f32; 8],
}

impl YM2151 {
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            channels: [FmChannel::default(); 8],
            lfo_counter: 0,
            lfo_enabled: false,
            lfo_frequency: 0,
            accumulated_cycles: 0.0,
            op_feedback: [0.0; 8],
        }
    }

    fn channel_freq_hz(&self, ch: usize) -> f32 {
        let kc = self.channels[ch].kc;
        let kf = self.channels[ch].kf;
        let oct = ((kc >> 4) & 0x07) as i32;
        let note = kc & 0x0F;
        let semitone = KC_NOTE_SEMITONE[note as usize];
        let kf_cents = (kf >> 2) as f32 / 64.0;
        let total_semitones = (oct - 4) * 12 + semitone - 9;
        440.0 * 2_f32.powf((total_semitones as f32 + kf_cents) / 12.0)
    }

    fn advance_phases(&mut self, sample_rate: u32) {
        for ch in 0..8 {
            let base_freq = self.channel_freq_hz(ch);
            for op in 0..4 {
                if !self.channels[ch].operators[op].key_on { continue; }
                let mult = FREQ_MULT[self.channels[ch].operators[op].mult as usize & 0xF];
                let inc = base_freq * mult * 2.0 * PI / sample_rate as f32;
                self.channels[ch].operators[op].phase_acc += inc;
                if self.channels[ch].operators[op].phase_acc > 2.0 * PI {
                    self.channels[ch].operators[op].phase_acc -= 2.0 * PI;
                }
            }
        }
    }

    fn op_output(&self, ch: usize, op: usize, mod_input: f32) -> f32 {
        if !self.channels[ch].operators[op].key_on { return 0.0; }
        let phase = self.channels[ch].operators[op].phase_acc + mod_input;
        let tl = 1.0 - self.channels[ch].operators[op].total_level as f32 / 127.0;
        phase.sin() * tl
    }

    fn get_channel_output(&mut self, ch: usize) -> (f32, f32) {
        let any_key_on = self.channels[ch].operators.iter().any(|op| op.key_on);
        if !any_key_on { return (0.0, 0.0); }

        let fb = if self.channels[ch].feedback > 0 {
            self.op_feedback[ch] * (self.channels[ch].feedback as f32 / 7.0) * 0.5
        } else {
            0.0
        };

        let m1 = self.op_output(ch, 0, fb);
        let m2 = self.op_output(ch, 1, 0.0);
        let c1 = self.op_output(ch, 2, 0.0);
        let c2 = self.op_output(ch, 3, 0.0);
        self.op_feedback[ch] = m1;

        let output = match self.channels[ch].algorithm {
            0 => {
                // M1→M2→C1→C2
                let m2_mod = self.op_output(ch, 1, m1 * PI);
                let c1_mod = self.op_output(ch, 2, m2_mod * PI);
                self.op_output(ch, 3, c1_mod * PI)
            }
            1 => {
                // (M1+M2)→C1→C2
                let c1_mod = self.op_output(ch, 2, (m1 + m2) * PI * 0.5);
                self.op_output(ch, 3, c1_mod * PI)
            }
            2 => {
                // M1→(M2→C1)→C2
                let m2_mod = self.op_output(ch, 1, 0.0);
                let c1_mod = self.op_output(ch, 2, (m1 + m2_mod) * PI * 0.5);
                self.op_output(ch, 3, c1_mod * PI)
            }
            3 => {
                // (M1→M2+C1)→C2
                let m2_mod = self.op_output(ch, 1, m1 * PI);
                let c1_mod = self.op_output(ch, 2, m1 * PI);
                self.op_output(ch, 3, (m2_mod + c1_mod) * PI * 0.5)
            }
            4 => {
                // (M1→M2)+(C1→C2)
                let m2_out = self.op_output(ch, 1, m1 * PI);
                let c2_out = self.op_output(ch, 3, c1 * PI);
                (m2_out + c2_out) * 0.5
            }
            5 => {
                // M1→(M2+C1+C2)
                let m2_out = self.op_output(ch, 1, m1 * PI);
                let c1_out = self.op_output(ch, 2, m1 * PI);
                let c2_out = self.op_output(ch, 3, m1 * PI);
                (m2_out + c1_out + c2_out) / 3.0
            }
            6 => {
                // (M1→M2)+C1+C2
                let m2_out = self.op_output(ch, 1, m1 * PI);
                (m2_out + c1 + c2) / 3.0
            }
            _ => {
                // AL=7: M1+M2+C1+C2 (all additive)
                (m1 + m2 + c1 + c2) * 0.25
            }
        };

        let sample = (output * 0.2).clamp(-1.0, 1.0);
        let left = if self.channels[ch].left_enable { sample } else { 0.0 };
        let right = if self.channels[ch].right_enable { sample } else { 0.0 };
        (left, right)
    }
}

impl SoundChipEmulator for YM2151 {
    fn name(&self) -> &'static str {
        "YM2151 (OPM)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.regs[addr as usize] = data;

        match addr {
            0x01 => { self.lfo_enabled = (data & 0x02) != 0; }
            0x08 => {
                // Key on/off: ch=bits[2:0], M1=bit3, C1=bit4, M2=bit5, C2=bit6
                let ch = (data & 0x07) as usize;
                self.channels[ch].operators[0].key_on = (data & 0x08) != 0; // M1
                self.channels[ch].operators[2].key_on = (data & 0x10) != 0; // C1
                self.channels[ch].operators[1].key_on = (data & 0x20) != 0; // M2
                self.channels[ch].operators[3].key_on = (data & 0x40) != 0; // C2
                if (data & 0x78) == 0 {
                    // All key-off: reset phases
                    for op in &mut self.channels[ch].operators {
                        op.phase_acc = 0.0;
                    }
                }
            }
            0x18 => { self.lfo_frequency = data; }
            // Channel control: L/R/FB/CON
            0x20..=0x27 => {
                let ch = (addr - 0x20) as usize;
                self.channels[ch].left_enable = (data & 0x40) != 0;
                self.channels[ch].right_enable = (data & 0x80) != 0;
                self.channels[ch].feedback = (data >> 3) & 0x07;
                self.channels[ch].algorithm = data & 0x07;
            }
            // Channel key code (KC)
            0x28..=0x2F => {
                let ch = (addr - 0x28) as usize;
                self.channels[ch].kc = data & 0x7F;
            }
            // Channel key fraction (KF)
            0x30..=0x37 => {
                let ch = (addr - 0x30) as usize;
                self.channels[ch].kf = data;
            }
            // Operator DT1/MULT: op=bits[4:3] of slot, ch=bits[2:0] — wait, slot = addr-base
            // Slot layout: base + op*8 + ch
            0x40..=0x5F => {
                let slot = (addr - 0x40) as usize;
                let op = slot / 8;
                let ch = slot % 8;
                if op < 4 {
                    self.channels[ch].operators[op].mult = data & 0x0F;
                }
            }
            // Operator TL (total level)
            0x60..=0x7F => {
                let slot = (addr - 0x60) as usize;
                let op = slot / 8;
                let ch = slot % 8;
                if op < 4 {
                    self.channels[ch].operators[op].total_level = data & 0x7F;
                }
            }
            // KS/AR, AM/D1R, DT2/D2R, D1L/RR — stored in cache, not yet wired to envelope
            0x80..=0xFF => {}
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        if self.lfo_enabled {
            self.lfo_counter = self.lfo_counter.wrapping_add(1);
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;

        for frame in buffer.chunks_mut(2) {
            self.advance_phases(sample_rate);

            let mut left = 0.0f32;
            let mut right = 0.0f32;
            for ch in 0..8 {
                let (l, r) = self.get_channel_output(ch);
                left += l;
                right += r;
            }
            frame[0] = (left / 8.0).clamp(-1.0, 1.0);
            frame[1] = (right / 8.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YM2151 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2151_new() {
        let chip = YM2151::new();
        assert_eq!(chip.name(), "YM2151 (OPM)");
        assert_eq!(chip.clock_rate(), 3_579_545);
        assert_eq!(chip.channels.len(), 8);
    }

    #[test]
    fn test_ym2151_channels() {
        let chip = YM2151::new();
        for ch in &chip.channels {
            assert_eq!(ch.operators.len(), 4);
        }
    }

    #[test]
    fn test_ym2151_write_kc() {
        let mut chip = YM2151::new();
        chip.write(0x28, 0x4C); // ch0 KC: OCT=4, NOTE=0xC → A4=440Hz
        assert_eq!(chip.channels[0].kc, 0x4C);
    }

    #[test]
    fn test_ym2151_write_key_on() {
        let mut chip = YM2151::new();
        // 0x08 data: bits[2:0]=ch, bits[6:3]=operators
        chip.write(0x08, 0x78); // key on all operators of ch 0
        assert!(chip.channels[0].operators[0].key_on); // M1
        assert!(chip.channels[0].operators[1].key_on); // M2
        assert!(chip.channels[0].operators[2].key_on); // C1
        assert!(chip.channels[0].operators[3].key_on); // C2
    }

    #[test]
    fn test_ym2151_write_channel_control() {
        let mut chip = YM2151::new();
        // 0x20 for ch0: left=1, right=1, FB=3, CON=5
        chip.write(0x20, 0b1100_0000 | (3 << 3) | 5);
        assert!(chip.channels[0].left_enable);
        assert!(chip.channels[0].right_enable);
        assert_eq!(chip.channels[0].feedback, 3);
        assert_eq!(chip.channels[0].algorithm, 5);
    }

    #[test]
    fn test_ym2151_write_operator_tl() {
        let mut chip = YM2151::new();
        // TL for M1 (op 0) of ch 0: addr = 0x60 + 0*8 + 0 = 0x60
        chip.write(0x60, 0x20); // TL = 32
        assert_eq!(chip.channels[0].operators[0].total_level, 32);
        // TL for C1 (op 2) of ch 3: addr = 0x60 + 2*8 + 3 = 0x73
        chip.write(0x73, 0x10);
        assert_eq!(chip.channels[3].operators[2].total_level, 16);
    }

    #[test]
    fn test_ym2151_generate_samples_active() {
        let mut chip = YM2151::new();
        chip.write(0x20, 0b1100_0111); // ch0: L+R, AL=7 (additive)
        chip.write(0x28, 0x4C);         // ch0 KC: A4
        chip.write(0x08, 0x78);         // key on all ops ch0
        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        assert!(buffer.iter().any(|&s| s != 0.0), "active channel must produce output");
    }

    #[test]
    fn test_ym2151_soundchip_trait() {
        let mut chip = YM2151::new();
        assert_eq!(chip.name(), "YM2151 (OPM)");
        chip.reset();
        chip.write(0x28, 0x4C);
        chip.write(0x08, 0xF8); // key on all ops ch 0 (0xF8 = 0b11111000)
        chip.clock();
        let mut buffer = [0.0f32; 2];
        chip.generate_samples(&mut buffer, 44100);
        assert!(buffer[0].abs() > 0.0 || buffer[1].abs() > 0.0);
    }
}
