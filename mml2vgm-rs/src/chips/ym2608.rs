//! YM2608 (OPNA) sound chip emulation
//!
//! The YM2608 is a 6-channel FM synthesis + 3-channel SSG chip used in various systems
//! including the PC-8801, PC-9801, and X68000.
//! It is part of the OPNA family of Yamaha sound chips.
//!
//! # Features
//! - 6 FM channels (each with 4 operators)
//! - 3 SSG channels (square wave generators)
//! - 6 Rhythm channels (BD, SD, TOP, HH, TOM, RIM)
//! - ADPCM playback
//! - Stereo output

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// SSG (square wave) channel
#[derive(Debug, Clone, Copy, Default)]
struct SsgChannel {
    frequency: u16,
    volume: u8,
    phase: u16,
}

/// FM channel for YM2608
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    frequency: u16,
    octave: u8,
    algorithm: u8,
    feedback: u8,
    output_level: u8,
    key_on: bool,
    output_phase: u32,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            frequency: 0,
            octave: 0,
            algorithm: 0,
            feedback: 0,
            output_level: 0,
            key_on: false,
            output_phase: 0,
        }
    }
}

/// YM2608 chip emulator with 6 FM + 3 SSG channels
pub struct YM2608 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache (extended to 0x400 for OPNA)
    regs: [u8; 0x400],

    /// 6 FM channels
    fm_channels: [FmChannel; 6],

    /// 3 SSG channels
    ssg_channels: [SsgChannel; 3],

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl YM2608 {
    /// Create a new YM2608 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(7_987_200)
    }

    /// Create a new YM2608 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x400],
            fm_channels: [FmChannel::default(); 6],
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
                let phase_f = (ch.output_phase as f32 / 1000000.0) * freq / self.sample_rate as f32 * 2.0 * PI;
                let sample = phase_f.sin() * (1.0 - ch.output_level as f32 / 127.0) * 0.2;
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
                let freq = 3_579_545.0 / (32.0 * ch.frequency as f32);
                let period = self.sample_rate as f32 / freq;
                let phase_norm = (ch.phase as f32 / period).fract();
                let sample = if phase_norm < 0.5 { 1.0 } else { -1.0 } * (1.0 - ch.volume as f32 / 15.0) * 0.15;
                left += sample;
                right += sample;
            }
        }

        (left, right)
    }
}

impl SoundChipEmulator for YM2608 {
    fn name(&self) -> &'static str {
        "YM2608 (OPNA)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        // Store in register cache
        if (addr as usize) < self.regs.len() {
            self.regs[addr as usize] = data;
        }

        // Handle register writes
        match addr {
            // FM channel frequency/octave registers
            0x28..=0x2D => {
                let ch = (addr - 0x28) as usize;
                if ch < 6 {
                    self.fm_channels[ch].frequency = ((data as u16 & 0x7F) << 2);
                }
            }
            // FM key on/off
            0x08 => {
                let ch = (data >> 4) & 0x07;
                if ch < 6 {
                    self.fm_channels[ch as usize].key_on = (data & 0x80) != 0;
                }
            }
            // FM output level
            0x60..=0x65 => {
                let ch = (addr - 0x60) as usize;
                if ch < 6 {
                    self.fm_channels[ch].output_level = 127 - (data >> 1);
                }
            }
            // SSG tone generators (0x0E-0x10)
            0x0E..=0x10 => {
                let ch = (addr - 0x0E) as usize;
                if ch < 3 {
                    self.ssg_channels[ch].frequency = (data as u16) << 4;
                }
            }
            // SSG volume (0x18-0x1A)
            0x18..=0x1A => {
                let ch = (addr - 0x18) as usize;
                if ch < 3 {
                    self.ssg_channels[ch].volume = data & 0x0F;
                }
            }
            _ => {
                // Unimplemented register
            }
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        // Update FM output phases
        for ch in &mut self.fm_channels {
            if ch.key_on {
                ch.output_phase = ch.output_phase.wrapping_add(1);
            }
        }

        // Update SSG phases
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

impl Default for YM2608 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2608_new() {
        let chip = YM2608::new();
        assert_eq!(chip.name(), "YM2608 (OPNA)");
        assert_eq!(chip.clock_rate(), 7_987_200);
        assert_eq!(chip.fm_channels.len(), 6);
        assert_eq!(chip.ssg_channels.len(), 3);
    }

    #[test]
    fn test_ym2608_write_fm() {
        let mut chip = YM2608::new();
        chip.write(0x28, 0x42); // FM channel 0 frequency
        assert_eq!(chip.regs[0x28], 0x42);
    }

    #[test]
    fn test_ym2608_write_ssg() {
        let mut chip = YM2608::new();
        chip.write(0x0E, 0x10); // SSG channel 0 frequency
        chip.write(0x18, 0x0F); // SSG channel 0 volume
        assert_eq!(chip.regs[0x0E], 0x10);
        assert_eq!(chip.regs[0x18], 0x0F);
    }

    #[test]
    fn test_ym2608_soundchip_trait() {
        let mut chip = YM2608::new();

        assert_eq!(chip.name(), "YM2608 (OPNA)");
        assert_eq!(chip.clock_rate(), 7_987_200);

        chip.reset();
        chip.write(0x08, 0x80); // FM key on (channel 0)
        chip.write(0x28, 0x45); // FM frequency (MIDI note 69 = A4 = 440 Hz)
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);

        // Should generate some output with key on
        assert!(buffer[0].abs() > 0.0 || buffer[1].abs() > 0.0);
    }
}
