//! YM3812 (OPL2) sound chip emulation
//!
//! The YM3812 is the improved OPL2 synthesizer used in many sound cards and arcade systems.
//! It provides improved FM synthesis quality over the original OPL (YM3526).
//!
//! # Features
//! - 9 FM channels (can be 6 melodic + 5 rhythm)
//! - 2 operators per channel
//! - Better FM synthesis quality than OPL
//! - Stereo panning

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// FM operator for OPL2
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

/// FM channel for YM3812
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

/// YM3812 chip emulator with 9 FM channels
pub struct YM3812 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache
    regs: [u8; 0x100],

    /// 9 FM channels (2 operators per channel)
    channels: [FmChannel; 9],

    /// Rhythm mode enable
    rhythm_mode: bool,

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl YM3812 {
    /// Create a new YM3812 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// Create a new YM3812 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            channels: [FmChannel::default(); 9],
            rhythm_mode: false,
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from a channel
    fn get_channel_output(&self, ch: usize) -> (f32, f32) {
        if ch >= 9 {
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

        let sample = output * 0.12;
        let left = if channel.left_enable { sample } else { 0.0 };
        let right = if channel.right_enable { sample } else { 0.0 };

        (left, right)
    }

    /// Get total output
    fn get_output(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for ch in 0..9 {
            let (ch_left, ch_right) = self.get_channel_output(ch);
            left += ch_left;
            right += ch_right;
        }

        (left / 9.0, right / 9.0)
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
            // Channel frequency number
            0x00..=0x08 => {
                let ch = addr as usize;
                if ch < 9 {
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0x300) | (data as u16);
                }
            }
            // Channel key on, block, frequency high
            0x10..=0x18 => {
                let ch = (addr - 0x10) as usize;
                if ch < 9 {
                    self.channels[ch].key_on = (data & 0x20) != 0;
                    self.channels[ch].block = (data & 0x1C) >> 2;
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0x0FF) | (((data as u16) & 0x03) << 8);
                }
            }
            // Operator 1 volume
            0x30..=0x38 => {
                let ch = (addr - 0x30) as usize;
                if ch < 9 {
                    self.channels[ch].operators[0].volume = (data >> 2) & 0x3F;
                }
            }
            // Operator 2 volume
            0x40..=0x48 => {
                let ch = (addr - 0x40) as usize;
                if ch < 9 {
                    self.channels[ch].operators[1].volume = (data >> 2) & 0x3F;
                }
            }
            // Panning
            0x50..=0x58 => {
                let ch = (addr - 0x50) as usize;
                if ch < 9 {
                    self.channels[ch].left_enable = (data & 0x10) != 0;
                    self.channels[ch].right_enable = (data & 0x20) != 0;
                }
            }
            // Rhythm mode
            0xBD => {
                self.rhythm_mode = (data & 0x20) != 0;
            }
            _ => {}
        }
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
    fn test_ym3812_write() {
        let mut chip = YM3812::new();
        chip.write(0x10, 0x20);
        assert!(chip.channels[0].key_on);
    }

    #[test]
    fn test_ym3812_soundchip_trait() {
        let mut chip = YM3812::new();
        assert_eq!(chip.name(), "YM3812 (OPL2)");

        chip.reset();
        chip.write(0x10, 0x20);
        chip.write(0x00, 0x40);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
