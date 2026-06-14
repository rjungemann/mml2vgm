//! YM3812 (OPL2) sound chip emulation
//!
//! The YM3812 provides 9 FM channels (or 6 melodic + 5 rhythm in rhythm mode).
//! Each channel has 2 operators in either FM (modulator→carrier) or additive mode.
//!
//! OPL2 register map:
//! - 0x20-0x35: Operator AM/VIB/EG/KSR/MULT (slot-addressed)
//! - 0x40-0x55: Operator KSL/TL (total level = attenuation)
//! - 0x60-0x75: Operator AR/DR
//! - 0x80-0x95: Operator SL/RR
//! - 0xA0-0xA8: Channel F-number LSB
//! - 0xB0-0xB8: Channel Key-on/Block/F-number MSB
//! - 0xBD: Rhythm mode enable
//! - 0xC0-0xC8: Channel feedback/connection
//! - 0xE0-0xF5: Operator waveform

use super::SoundChipEmulator;
use std::f32::consts::PI;

const FREQ_MULT: [f32; 16] = [
    0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 10.0, 12.0, 12.0, 15.0, 15.0,
];

/// Operator slot offset → (channel, operator_index)
/// OPL2 layout: slots 0-2 = ch0-2 mod, 3-5 = ch0-2 car; 8-10 = ch3-5 mod; etc.
fn slot_to_ch_op(slot_offset: u8) -> Option<(usize, usize)> {
    let row = (slot_offset / 8) as usize;
    let col = (slot_offset % 8) as usize;
    if row > 2 || col > 5 {
        return None;
    }
    if col < 3 {
        Some((row * 3 + col, 0))
    } else {
        Some((row * 3 + col - 3, 1))
    }
}

/// FM operator
#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase_acc: f32,
    total_level: u8,
    mult: u8,
    #[allow(dead_code)]
    key_on: bool,
}

impl Default for FmOperator {
    fn default() -> Self {
        // Real OPL hardware: all regs = 0 after reset → TL = 0 (max volume)
        Self {
            phase_acc: 0.0,
            total_level: 0,
            mult: 1,
            key_on: false,
        }
    }
}

/// FM channel
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    operators: [FmOperator; 2],
    f_num: u16,
    block: u8,
    key_on: bool,
    connection: u8,
    feedback: u8,
    left_enable: bool,
    right_enable: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            operators: [FmOperator::default(); 2],
            f_num: 0,
            block: 0,
            key_on: false,
            connection: 0,
            feedback: 0,
            left_enable: true,
            right_enable: true,
        }
    }
}

/// YM3812 chip emulator with 9 FM channels
pub struct YM3812 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 0x100],
    channels: [FmChannel; 9],
    rhythm_mode: bool,
    #[allow(dead_code)]
    accumulated_cycles: f32,
    mod_feedback: [f32; 9],
}

impl YM3812 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            channels: [FmChannel::default(); 9],
            rhythm_mode: false,
            accumulated_cycles: 0.0,
            mod_feedback: [0.0; 9],
        }
    }

    fn channel_freq_hz(&self, ch: usize) -> f32 {
        let f_num = self.channels[ch].f_num as f32;
        let block = self.channels[ch].block as i32;
        f_num * 2_f32.powi(block - 1) * self.clock_rate as f32 / (1u32 << 19) as f32
    }

    fn get_channel_output(&mut self, ch: usize) -> (f32, f32) {
        if ch >= 9 || !self.channels[ch].key_on {
            return (0.0, 0.0);
        }

        let base_freq = self.channel_freq_hz(ch);
        let mult0 = FREQ_MULT[self.channels[ch].operators[0].mult as usize & 0xF];
        let mult1 = FREQ_MULT[self.channels[ch].operators[1].mult as usize & 0xF];

        let op0_phase = self.channels[ch].operators[0].phase_acc;
        let op1_phase = self.channels[ch].operators[1].phase_acc;

        let tl0 = 1.0 - self.channels[ch].operators[0].total_level as f32 / 63.0;
        let tl1 = 1.0 - self.channels[ch].operators[1].total_level as f32 / 63.0;

        let _ = (base_freq, mult0, mult1);

        let mod_out = op0_phase.sin() * tl0 + self.mod_feedback[ch] * 0.25;

        let output = if self.channels[ch].connection == 0 {
            (op1_phase + mod_out * PI).sin() * tl1
        } else {
            mod_out + op1_phase.sin() * tl1
        };

        self.mod_feedback[ch] = mod_out;

        let sample = (output * 0.15).clamp(-1.0, 1.0);
        let left = if self.channels[ch].left_enable {
            sample
        } else {
            0.0
        };
        let right = if self.channels[ch].right_enable {
            sample
        } else {
            0.0
        };
        (left, right)
    }

    fn advance_phases(&mut self, sample_rate: u32) {
        for ch in 0..9 {
            if !self.channels[ch].key_on {
                continue;
            }
            let base_freq = self.channel_freq_hz(ch);
            for op in 0..2 {
                let mult = FREQ_MULT[self.channels[ch].operators[op].mult as usize & 0xF];
                let freq = base_freq * mult;
                let inc = freq * 2.0 * PI / sample_rate as f32;
                self.channels[ch].operators[op].phase_acc += inc;
                if self.channels[ch].operators[op].phase_acc > 2.0 * PI {
                    self.channels[ch].operators[op].phase_acc -= 2.0 * PI;
                }
            }
        }
    }
}

impl SoundChipEmulator for YM3812 {
    fn name(&self) -> &'static str {
        "YM3812 (OPL2)"
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
            // Operator registers (slot-addressed)
            0x20..=0x35 => {
                if let Some((ch, op)) = slot_to_ch_op(addr - 0x20) {
                    self.channels[ch].operators[op].mult = data & 0x0F;
                }
            }
            0x40..=0x55 => {
                if let Some((ch, op)) = slot_to_ch_op(addr - 0x40) {
                    self.channels[ch].operators[op].total_level = data & 0x3F;
                }
            }
            0x60..=0x75 | 0x80..=0x95 | 0xE0..=0xF5 => {
                // AR/DR, SL/RR, waveform — stored in regs cache, not yet wired to envelope
            }

            // Channel registers
            0xA0..=0xA8 => {
                let ch = (addr - 0xA0) as usize;
                self.channels[ch].f_num = (self.channels[ch].f_num & 0x300) | data as u16;
            }
            0xB0..=0xB8 => {
                let ch = (addr - 0xB0) as usize;
                let prev_key_on = self.channels[ch].key_on;
                self.channels[ch].key_on = (data & 0x20) != 0;
                self.channels[ch].block = (data >> 2) & 0x07;
                self.channels[ch].f_num =
                    (self.channels[ch].f_num & 0x0FF) | (((data as u16) & 0x03) << 8);
                if !prev_key_on && self.channels[ch].key_on {
                    self.channels[ch].operators[0].phase_acc = 0.0;
                    self.channels[ch].operators[1].phase_acc = 0.0;
                    self.mod_feedback[ch] = 0.0;
                }
            }
            0xBD => {
                self.rhythm_mode = (data & 0x20) != 0;
            }
            0xC0..=0xC8 => {
                let ch = (addr - 0xC0) as usize;
                self.channels[ch].connection = data & 0x01;
                self.channels[ch].feedback = (data >> 1) & 0x07;
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;

        for frame in buffer.chunks_mut(2) {
            self.advance_phases(sample_rate);

            let mut left = 0.0f32;
            let mut right = 0.0f32;
            for ch in 0..9 {
                let (l, r) = self.get_channel_output(ch);
                left += l;
                right += r;
            }

            frame[0] = (left / 9.0).clamp(-1.0, 1.0);
            frame[1] = (right / 9.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YM3812 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym3812_new() {
        let chip = YM3812::new();
        assert_eq!(chip.name(), "YM3812 (OPL2)");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_ym3812_channels() {
        let chip = YM3812::new();
        assert_eq!(chip.channels.len(), 9);
        for ch in &chip.channels {
            assert_eq!(ch.operators.len(), 2);
        }
    }

    #[test]
    fn test_ym3812_write_freq_and_keyon() {
        let mut chip = YM3812::new();
        chip.write(0xA0, 0x57); // ch0 f_num lo
        chip.write(0xB0, 0x31); // ch0 key_on=1, block=4, f_num hi=1 → f_num = 0x157
        assert!(chip.channels[0].key_on);
        assert_eq!(chip.channels[0].block, 4);
        assert_eq!(chip.channels[0].f_num, 0x157);
    }

    #[test]
    fn test_ym3812_slot_to_ch_op() {
        assert_eq!(slot_to_ch_op(0), Some((0, 0))); // ch0 mod
        assert_eq!(slot_to_ch_op(3), Some((0, 1))); // ch0 car
        assert_eq!(slot_to_ch_op(1), Some((1, 0))); // ch1 mod
        assert_eq!(slot_to_ch_op(8), Some((3, 0))); // ch3 mod
        assert_eq!(slot_to_ch_op(11), Some((3, 1))); // ch3 car
        assert_eq!(slot_to_ch_op(7), None); // unused
    }

    #[test]
    fn test_ym3812_write_operator() {
        let mut chip = YM3812::new();
        chip.write(0x40, 0x1F); // slot 0 (ch0 mod) TL=31
        assert_eq!(chip.channels[0].operators[0].total_level, 31);
        chip.write(0x43, 0x0A); // slot 3 (ch0 car) TL=10
        assert_eq!(chip.channels[0].operators[1].total_level, 10);
    }

    #[test]
    fn test_ym3812_keyon_resets_phase() {
        let mut chip = YM3812::new();
        chip.channels[0].operators[0].phase_acc = 1.5;
        chip.write(0xB0, 0x20); // key on
        assert_eq!(chip.channels[0].operators[0].phase_acc, 0.0);
    }

    #[test]
    fn test_ym3812_generate_samples_active_channel() {
        let mut chip = YM3812::new();
        chip.write(0xA0, 0x57);
        chip.write(0xB0, 0x31); // key_on ch0

        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        let nonzero = buffer.iter().any(|&s| s != 0.0);
        assert!(nonzero, "active channel should produce non-zero output");
    }

    #[test]
    fn test_ym3812_soundchip_trait() {
        let mut chip = YM3812::new();
        assert_eq!(chip.name(), "YM3812 (OPL2)");
        chip.reset();
        chip.write(0xA0, 0x57);
        chip.write(0xB0, 0x31);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
