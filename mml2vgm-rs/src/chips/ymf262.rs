//! YMF262 (OPL3) sound chip emulation
//!
//! The YMF262 is the advanced OPL3 synthesizer with 18 FM channels.
//! It is the most capable OPL variant, capable of producing high-quality FM synthesis.
//!
//! # Features
//! - 18 FM channels (two 9-channel OPL2 cores)
//! - 2 operators per channel
//! - Stereo panning per channel
//! - Can operate in OPL2 compatibility mode

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// FM operator for OPL3
#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase: u16,
    key_on: bool,
    volume: u8,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self {
            phase: 0,
            key_on: false,
            volume: 127,
        }
    }
}

/// FM channel for YMF262
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    operators: [FmOperator; 2],
    frequency: u16,
    block: u8,
    algorithm: u8,
    key_on: bool,
    left_enable: bool,
    right_enable: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            operators: [FmOperator::default(); 2],
            frequency: 0,
            block: 0,
            algorithm: 0,
            key_on: false,
            left_enable: true,
            right_enable: true,
        }
    }
}

/// YMF262 chip emulator with 18 FM channels (two OPL2 cores)
pub struct YMF262 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache (0x100 for first chip, 0x100 for second)
    regs: [u8; 0x200],

    /// 18 FM channels (9 per OPL2 core)
    channels: [FmChannel; 18],

    /// OPL3 enable flag (true for OPL3 mode, false for OPL2 compat)
    opl3_mode: bool,

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl YMF262 {
    /// Create a new YMF262 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(14_318_180)
    }

    /// Create a new YMF262 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x200],
            channels: [FmChannel::default(); 18],
            opl3_mode: true,
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from a channel
    fn get_channel_output(&self, ch: usize) -> (f32, f32) {
        if ch >= 18 {
            return (0.0, 0.0);
        }

        let channel = &self.channels[ch];
        if !channel.key_on {
            return (0.0, 0.0);
        }

        let base_freq = 440.0 * 2_f32.powi((channel.frequency as i32 - 69) / 12);
        let freq = base_freq * 2_f32.powi((channel.block as i32) - 4);

        let op1_phase = (channel.operators[0].phase as f32 / 65536.0) * freq / self.sample_rate as f32 * 2.0 * PI;
        let op2_phase = (channel.operators[1].phase as f32 / 65536.0) * freq / self.sample_rate as f32 * 2.0 * PI;

        let output = match channel.algorithm {
            0 => {
                let carrier = op2_phase.sin() * (1.0 - channel.operators[1].volume as f32 / 127.0);
                carrier
            }
            _ => {
                let modulator = op1_phase.sin() * (1.0 - channel.operators[0].volume as f32 / 127.0);
                let carrier = (op2_phase + modulator).sin() * (1.0 - channel.operators[1].volume as f32 / 127.0);
                carrier
            }
        };

        let sample = output * 0.08; // Reduce amplitude for 18 channels
        let left = if channel.left_enable { sample } else { 0.0 };
        let right = if channel.right_enable { sample } else { 0.0 };

        (left, right)
    }

    /// Get total output
    fn get_output(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for ch in 0..18 {
            let (ch_left, ch_right) = self.get_channel_output(ch);
            left += ch_left;
            right += ch_right;
        }

        (left / 18.0, right / 18.0)
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
        self.regs[bank * 0x100 + addr as usize] = data;
        let ch_base = bank * 9;

        match addr {
            0x00..=0x08 => {
                let ch = ch_base + addr as usize;
                if ch < 18 {
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0x300) | data as u16;
                }
            }
            0x10..=0x18 => {
                let ch = ch_base + (addr - 0x10) as usize;
                if ch < 18 {
                    self.channels[ch].key_on = (data & 0x20) != 0;
                    self.channels[ch].block = (data & 0x1C) >> 2;
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0x0FF) | (((data as u16) & 0x03) << 8);
                }
            }
            0x30..=0x38 => {
                let ch = ch_base + (addr - 0x30) as usize;
                if ch < 18 {
                    self.channels[ch].operators[0].volume = (data >> 2) & 0x3F;
                }
            }
            0x40..=0x48 => {
                let ch = ch_base + (addr - 0x40) as usize;
                if ch < 18 {
                    self.channels[ch].operators[1].volume = (data >> 2) & 0x3F;
                }
            }
            0x50..=0x58 => {
                let ch = ch_base + (addr - 0x50) as usize;
                if ch < 18 {
                    self.channels[ch].left_enable = (data & 0x10) != 0;
                    self.channels[ch].right_enable = (data & 0x20) != 0;
                }
            }
            // OPL3 mode enable — only in bank 0
            0x04 if bank == 0 => {
                self.opl3_mode = (data & 0x01) != 0;
            }
            _ => {}
        }
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.write_port(0, addr, data);
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        for ch in &mut self.channels {
            for op in &mut ch.operators {
                if op.key_on {
                    op.phase = op.phase.wrapping_add(1);
                }
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;

        for frame in buffer.chunks_mut(2) {
            let cycles_per_sample = self.clock_rate as f32 / sample_rate as f32;
            self.accumulated_cycles += cycles_per_sample;

            while self.accumulated_cycles >= 1.0 {
                self.clock();
                self.accumulated_cycles -= 1.0;
            }

            let (left, right) = self.get_output();
            frame[0] = left.clamp(-1.0, 1.0);
            frame[1] = right.clamp(-1.0, 1.0);
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
    fn test_ymf262_write() {
        let mut chip = YMF262::new();
        chip.write(0x10, 0x20);
        assert!(chip.channels[0].key_on);
    }

    #[test]
    fn test_ymf262_soundchip_trait() {
        let mut chip = YMF262::new();
        assert_eq!(chip.name(), "YMF262 (OPL3)");

        chip.reset();
        chip.write(0x10, 0x20);
        chip.write(0x00, 0x40);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
