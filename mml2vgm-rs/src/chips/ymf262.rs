//! YMF262 (OPL3) sound chip emulation
//!
//! The YMF262 provides 18 FM channels in two 9-channel banks.
//! VGM uses opcodes 0x5E (port 0 / bank 0, channels 0-8)
//! and 0x5F (port 1 / bank 1, channels 9-17).
//!
//! OPL3 register map (per bank):
//! - 0x20-0x35: Operator AM/VIB/EG/KSR/MULT (slot-addressed)
//! - 0x40-0x55: Operator KSL/TL (total level)
//! - 0x60-0x75: Operator AR/DR
//! - 0x80-0x95: Operator SL/RR
//! - 0xA0-0xA8: Channel F-number LSB
//! - 0xB0-0xB8: Channel Key-on/Block/F-number MSB
//! - 0xBD: Rhythm mode (bank 0 only)
//! - 0xC0-0xC8: Channel feedback/connection + OPL3 L/R enable
//! - 0xE0-0xF5: Operator waveform
//!
//! Bank 0 addr 0x05 bit 0: OPL3 mode enable

use super::SoundChipEmulator;
use std::f32::consts::PI;

const FREQ_MULT: [f32; 16] = [
    0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 10.0, 12.0, 12.0, 15.0, 15.0,
];

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

#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase_acc: f32,
    total_level: u8,
    mult: u8,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self {
            phase_acc: 0.0,
            total_level: 0,
            mult: 1,
        }
    }
}

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

/// YMF262 chip emulator with 18 FM channels (two 9-channel banks)
pub struct YMF262 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 0x200],
    channels: [FmChannel; 18],
    opl3_mode: bool,
    #[allow(dead_code)]
    accumulated_cycles: f32,
    mod_feedback: [f32; 18],
}

impl YMF262 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(14_318_180)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x200],
            channels: [FmChannel::default(); 18],
            opl3_mode: true,
            accumulated_cycles: 0.0,
            mod_feedback: [0.0; 18],
        }
    }

    fn channel_freq_hz(&self, ch: usize) -> f32 {
        let f_num = self.channels[ch].f_num as f32;
        let block = self.channels[ch].block as i32;
        // OPL3 runs at clock/288, YM3812 at clock/72; adjust for actual OPL3 clock
        f_num * 2_f32.powi(block - 1) * (self.clock_rate as f32 / 4.0) / (1u32 << 19) as f32
    }

    fn advance_phases(&mut self, sample_rate: u32) {
        for ch in 0..18 {
            if !self.channels[ch].key_on {
                continue;
            }
            let base_freq = self.channel_freq_hz(ch);
            for op in 0..2 {
                let mult = FREQ_MULT[self.channels[ch].operators[op].mult as usize & 0xF];
                let inc = base_freq * mult * 2.0 * PI / sample_rate as f32;
                self.channels[ch].operators[op].phase_acc += inc;
                if self.channels[ch].operators[op].phase_acc > 2.0 * PI {
                    self.channels[ch].operators[op].phase_acc -= 2.0 * PI;
                }
            }
        }
    }

    fn get_channel_output(&mut self, ch: usize) -> (f32, f32) {
        if ch >= 18 || !self.channels[ch].key_on {
            return (0.0, 0.0);
        }
        let op0_phase = self.channels[ch].operators[0].phase_acc;
        let op1_phase = self.channels[ch].operators[1].phase_acc;
        let tl0 = 1.0 - self.channels[ch].operators[0].total_level as f32 / 63.0;
        let tl1 = 1.0 - self.channels[ch].operators[1].total_level as f32 / 63.0;
        let mod_out = op0_phase.sin() * tl0 + self.mod_feedback[ch] * 0.25;
        let output = if self.channels[ch].connection == 0 {
            (op1_phase + mod_out * PI).sin() * tl1
        } else {
            mod_out + op1_phase.sin() * tl1
        };
        self.mod_feedback[ch] = mod_out;
        let sample = (output * 0.1).clamp(-1.0, 1.0);
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

    fn write_bank(&mut self, bank: usize, addr: u8, data: u8) {
        let ch_base = bank * 9;
        self.regs[bank * 0x100 + addr as usize] = data;

        match addr {
            // OPL3 enable (bank 0 only)
            0x05 if bank == 0 => {
                self.opl3_mode = (data & 0x01) != 0;
            }

            // Operator registers (slot-addressed within bank)
            0x20..=0x35 => {
                if let Some((ch, op)) = slot_to_ch_op(addr - 0x20) {
                    let ch_abs = ch_base + ch;
                    if ch_abs < 18 {
                        self.channels[ch_abs].operators[op].mult = data & 0x0F;
                    }
                }
            }
            0x40..=0x55 => {
                if let Some((ch, op)) = slot_to_ch_op(addr - 0x40) {
                    let ch_abs = ch_base + ch;
                    if ch_abs < 18 {
                        self.channels[ch_abs].operators[op].total_level = data & 0x3F;
                    }
                }
            }
            0x60..=0x75 | 0x80..=0x95 | 0xE0..=0xF5 => {}

            // Channel registers
            0xA0..=0xA8 => {
                let ch = ch_base + (addr - 0xA0) as usize;
                if ch < 18 {
                    self.channels[ch].f_num = (self.channels[ch].f_num & 0x300) | data as u16;
                }
            }
            0xB0..=0xB8 => {
                let ch = ch_base + (addr - 0xB0) as usize;
                if ch < 18 {
                    let prev = self.channels[ch].key_on;
                    self.channels[ch].key_on = (data & 0x20) != 0;
                    self.channels[ch].block = (data >> 2) & 0x07;
                    self.channels[ch].f_num =
                        (self.channels[ch].f_num & 0x0FF) | (((data as u16) & 0x03) << 8);
                    if !prev && self.channels[ch].key_on {
                        self.channels[ch].operators[0].phase_acc = 0.0;
                        self.channels[ch].operators[1].phase_acc = 0.0;
                        self.mod_feedback[ch] = 0.0;
                    }
                }
            }
            0xBD if bank == 0 => {}
            0xC0..=0xC8 => {
                let ch = ch_base + (addr - 0xC0) as usize;
                if ch < 18 {
                    self.channels[ch].connection = data & 0x01;
                    self.channels[ch].feedback = (data >> 1) & 0x07;
                    // OPL3 L/R bits in 0xCx
                    if self.opl3_mode {
                        self.channels[ch].left_enable = (data & 0x10) != 0;
                        self.channels[ch].right_enable = (data & 0x20) != 0;
                    }
                }
            }
            _ => {}
        }
    }
}

impl SoundChipEmulator for YMF262 {
    fn name(&self) -> &'static str {
        "YMF262 (OPL3)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let bank = (port & 1) as usize;
        self.write_bank(bank, addr, data);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.write_bank(0, addr, data);
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
            for ch in 0..18 {
                let (l, r) = self.get_channel_output(ch);
                left += l;
                right += r;
            }
            frame[0] = (left / 18.0).clamp(-1.0, 1.0);
            frame[1] = (right / 18.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YMF262 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ymf262_new() {
        let chip = YMF262::new();
        assert_eq!(chip.name(), "YMF262 (OPL3)");
        assert_eq!(chip.clock_rate(), 14_318_180);
    }

    #[test]
    fn test_ymf262_channels() {
        let chip = YMF262::new();
        assert_eq!(chip.channels.len(), 18);
        for ch in &chip.channels {
            assert_eq!(ch.operators.len(), 2);
        }
    }

    #[test]
    fn test_ymf262_write_bank0_key_on() {
        let mut chip = YMF262::new();
        chip.write(0xA0, 0x57); // ch0 f_num lo
        chip.write(0xB0, 0x31); // ch0 key_on=1, block=4
        assert!(chip.channels[0].key_on);
        assert_eq!(chip.channels[0].block, 4);
    }

    #[test]
    fn test_ymf262_write_port_bank1_key_on() {
        let mut chip = YMF262::new();
        chip.write_port(1, 0xA0, 0x57); // ch9 f_num lo
        chip.write_port(1, 0xB0, 0x31); // ch9 key_on=1, block=4
        assert!(chip.channels[9].key_on);
        assert_eq!(chip.channels[9].block, 4);
    }

    #[test]
    fn test_ymf262_generate_samples_active() {
        let mut chip = YMF262::new();
        chip.write(0xA0, 0x57);
        chip.write(0xB0, 0x31);
        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        assert!(
            buffer.iter().any(|&s| s != 0.0),
            "active channel must produce output"
        );
    }

    #[test]
    fn test_ymf262_opl3_lr_enable() {
        let mut chip = YMF262::new();
        chip.write_port(0, 0x05, 0x01); // enable OPL3 mode
        chip.write_port(0, 0xC0, 0x30); // ch0: left=1, right=1
        assert!(chip.channels[0].left_enable);
        assert!(chip.channels[0].right_enable);
    }

    #[test]
    fn test_ymf262_soundchip_trait() {
        let mut chip = YMF262::new();
        chip.reset();
        chip.write(0xA0, 0x57);
        chip.write(0xB0, 0x31);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
