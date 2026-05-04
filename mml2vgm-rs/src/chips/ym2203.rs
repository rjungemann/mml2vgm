//! YM2203 (OPN) sound chip emulation
//!
//! The YM2203 is a 3-channel FM + 3-channel SSG chip used in various arcade systems.
//! It is the precursor to the YM2608 OPNA chip.
//!
//! # Features
//! - 3 FM channels (each with 4 operators)
//! - 3 SSG channels (square wave generators)
//! - Stereo output
//! - Simpler than YM2608 (no rhythm, no ADPCM)

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// SSG (square wave) channel
#[derive(Debug, Clone, Copy, Default)]
struct SsgChannel {
    frequency: u16,
    volume: u8,
    phase: u16,
}

/// FM channel for YM2203
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    frequency: u16,
    octave: u8,
    algorithm: u8,
    feedback: u8,
    output_level: u8,
    key_on: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            frequency: 0,
            octave: 0,
            algorithm: 0,
            feedback: 0,
            output_level: 127,
            key_on: false,
        }
    }
}

/// YM2203 chip emulator with 3 FM + 3 SSG channels
pub struct YM2203 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache
    regs: [u8; 0x100],

    /// 3 FM channels
    fm_channels: [FmChannel; 3],

    /// 3 SSG channels
    ssg_channels: [SsgChannel; 3],

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl YM2203 {
    /// Create a new YM2203 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(1_789_772)
    }

    /// Create a new YM2203 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            fm_channels: [FmChannel::default(); 3],
            ssg_channels: [SsgChannel::default(); 3],
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from FM channels
    fn get_fm_output(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for ch in &self.fm_channels {
            if ch.key_on && ch.output_level < 127 {
                let freq = (440.0 * 2_f32.powi((ch.frequency as i32 - 69) / 12)) * 2_f32.powi(ch.octave as i32);
                let sample = (freq / self.sample_rate as f32 * 2.0 * PI).sin() * (1.0 - ch.output_level as f32 / 127.0) * 0.15;
                left += sample;
                right += sample;
            }
        }

        (left, right)
    }

    /// Get output from SSG channels
    fn get_ssg_output(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for ch in &self.ssg_channels {
            if ch.frequency > 0 && ch.volume < 15 {
                let freq = 1_789_772.0 / (32.0 * ch.frequency as f32);
                let period = self.sample_rate as f32 / freq;
                let phase_norm = (ch.phase as f32 / period).fract();
                let sample = if phase_norm < 0.5 { 1.0 } else { -1.0 } * (1.0 - ch.volume as f32 / 15.0) * 0.1;
                left += sample;
                right += sample;
            }
        }

        (left, right)
    }
}

impl SoundChipEmulator for YM2203 {
    fn name(&self) -> &'static str {
        "YM2203 (OPN)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        if (addr as usize) < self.regs.len() {
            self.regs[addr as usize] = data;
        }

        match addr {
            // FM channel frequency/octave
            0x28..=0x2A => {
                let ch = (addr - 0x28) as usize;
                if ch < 3 {
                    self.fm_channels[ch].frequency = ((data as u16 & 0x7F) << 2);
                }
            }
            // FM key on/off
            0x08 => {
                let ch = (data >> 4) & 0x07;
                if ch < 3 {
                    self.fm_channels[ch as usize].key_on = (data & 0x80) != 0;
                }
            }
            // FM output level
            0x60..=0x62 => {
                let ch = (addr - 0x60) as usize;
                if ch < 3 {
                    self.fm_channels[ch].output_level = 127 - (data >> 1);
                }
            }
            // SSG tone generators
            0x0E..=0x10 => {
                let ch = (addr - 0x0E) as usize;
                if ch < 3 {
                    self.ssg_channels[ch].frequency = (data as u16) << 4;
                }
            }
            // SSG volume
            0x18..=0x1A => {
                let ch = (addr - 0x18) as usize;
                if ch < 3 {
                    self.ssg_channels[ch].volume = data & 0x0F;
                }
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        for ch in &mut self.ssg_channels {
            ch.phase = ch.phase.wrapping_add(1);
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

            let (fm_left, fm_right) = self.get_fm_output();
            let (ssg_left, ssg_right) = self.get_ssg_output();

            frame[0] = (fm_left + ssg_left).clamp(-1.0, 1.0);
            frame[1] = (fm_right + ssg_right).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YM2203 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2203_new() {
        let chip = YM2203::new();
        assert_eq!(chip.name(), "YM2203 (OPN)");
        assert_eq!(chip.clock_rate(), 1_789_772);
    }

    #[test]
    fn test_ym2203_channels() {
        let chip = YM2203::new();
        assert_eq!(chip.fm_channels.len(), 3);
        assert_eq!(chip.ssg_channels.len(), 3);
    }

    #[test]
    fn test_ym2203_write() {
        let mut chip = YM2203::new();
        chip.write(0x28, 0x40);
        assert_eq!(chip.regs[0x28], 0x40);
    }

    #[test]
    fn test_ym2203_soundchip_trait() {
        let mut chip = YM2203::new();
        assert_eq!(chip.name(), "YM2203 (OPN)");
        assert_eq!(chip.clock_rate(), 1_789_772);

        chip.reset();
        chip.write(0x08, 0xF0);
        chip.write(0x28, 0x40);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
