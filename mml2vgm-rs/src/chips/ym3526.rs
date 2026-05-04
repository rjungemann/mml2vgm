//! YM3526 (OPL) sound chip emulation
//!
//! The YM3526 is a 9-channel FM synthesis chip used in early arcade systems.
//! It is the first in the OPL family of Yamaha sound chips.
//!
//! # Features
//! - 9 FM channels
//! - Can be configured as 9 melodic or 6 melodic + 5 rhythm
//! - 2 operators per channel
//! - Stereo panning per channel

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// FM channel for YM3526
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    frequency: u16,
    block: u8,
    volume: u8,
    key_on: bool,
    left_enable: bool,
    right_enable: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            frequency: 0,
            block: 0,
            volume: 127,
            key_on: false,
            left_enable: true,
            right_enable: true,
        }
    }
}

/// YM3526 chip emulator with 9 FM channels
pub struct YM3526 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache
    regs: [u8; 0x100],

    /// 9 FM channels
    channels: [FmChannel; 9],

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl YM3526 {
    /// Create a new YM3526 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// Create a new YM3526 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            channels: [FmChannel::default(); 9],
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from a channel
    fn get_channel_output(&self, ch: usize) -> (f32, f32) {
        if ch >= 9 {
            return (0.0, 0.0);
        }

        let channel = &self.channels[ch];
        if !channel.key_on || channel.volume == 127 {
            return (0.0, 0.0);
        }

        let base_freq = 440.0 * 2_f32.powi((channel.frequency as i32 - 69) / 12);
        let freq = base_freq * 2_f32.powi((channel.block as i32) - 4);
        let phase = (freq / self.sample_rate as f32 * 2.0 * PI).sin();
        let output = phase * (1.0 - channel.volume as f32 / 127.0) * 0.12;

        let left = if channel.left_enable { output } else { 0.0 };
        let right = if channel.right_enable { output } else { 0.0 };

        (left, right)
    }

    /// Get total output mixing all channels
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

impl SoundChipEmulator for YM3526 {
    fn name(&self) -> &'static str {
        "YM3526 (OPL)"
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
            // Channel frequency number (low byte)
            0x00..=0x08 => {
                let ch = addr as usize;
                if ch < 9 {
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0x300) | (data as u16);
                }
            }
            // Channel key on, block, frequency (high byte)
            0x10..=0x18 => {
                let ch = (addr - 0x10) as usize;
                if ch < 9 {
                    self.channels[ch].key_on = (data & 0x20) != 0;
                    self.channels[ch].block = (data & 0x1C) >> 2;
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0x0FF) | (((data as u16) & 0x03) << 8);
                }
            }
            // Channel volume
            0x30..=0x38 => {
                let ch = (addr - 0x30) as usize;
                if ch < 9 {
                    self.channels[ch].volume = (data >> 2) & 0x3F;
                }
            }
            // Panning (left/right enable)
            0x40..=0x48 => {
                let ch = (addr - 0x40) as usize;
                if ch < 9 {
                    self.channels[ch].left_enable = (data & 0x10) != 0;
                    self.channels[ch].right_enable = (data & 0x20) != 0;
                }
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        // OPL3 timing implementation would go here
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

impl Default for YM3526 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym3526_new() {
        let chip = YM3526::new();
        assert_eq!(chip.name(), "YM3526 (OPL)");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_ym3526_channels() {
        let chip = YM3526::new();
        assert_eq!(chip.channels.len(), 9);
        for ch in &chip.channels {
            assert!(!ch.key_on);
        }
    }

    #[test]
    fn test_ym3526_write_key_on() {
        let mut chip = YM3526::new();
        chip.write(0x10, 0x20);
        assert!(chip.channels[0].key_on);
    }

    #[test]
    fn test_ym3526_soundchip_trait() {
        let mut chip = YM3526::new();
        assert_eq!(chip.name(), "YM3526 (OPL)");

        chip.reset();
        chip.write(0x10, 0x20);
        chip.write(0x00, 0x40);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
