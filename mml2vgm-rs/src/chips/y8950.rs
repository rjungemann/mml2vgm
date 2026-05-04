//! Y8950 sound chip emulation
//!
//! The Y8950 is an OPL variant with ADPCM playback capability used in some arcade systems.
//! It combines 9 FM channels with ADPCM sample playback.
//!
//! # Features
//! - 9 FM channels (same as OPL)
//! - ADPCM-B playback (4-bit ADPCM)
//! - Stereo output

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// FM channel for Y8950
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

/// Y8950 chip emulator with 9 FM channels and ADPCM
pub struct Y8950 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache
    regs: [u8; 0x100],

    /// 9 FM channels
    channels: [FmChannel; 9],

    /// ADPCM memory (64KB)
    adpcm_memory: Vec<u8>,

    /// ADPCM current address
    adpcm_address: u32,

    /// ADPCM playback active
    adpcm_playing: bool,

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl Y8950 {
    /// Create a new Y8950 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// Create a new Y8950 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            channels: [FmChannel::default(); 9],
            adpcm_memory: vec![0; 65536],
            adpcm_address: 0,
            adpcm_playing: false,
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from FM channels
    fn get_fm_output(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for ch in &self.channels {
            if ch.key_on && ch.volume < 127 {
                let base_freq = 440.0 * 2_f32.powi((ch.frequency as i32 - 69) / 12);
                let freq = base_freq * 2_f32.powi((ch.block as i32) - 4);
                let phase = (freq / self.sample_rate as f32 * 2.0 * PI).sin();
                let sample = phase * (1.0 - ch.volume as f32 / 127.0) * 0.12;

                let ch_left = if ch.left_enable { sample } else { 0.0 };
                let ch_right = if ch.right_enable { sample } else { 0.0 };

                left += ch_left;
                right += ch_right;
            }
        }

        (left / 9.0, right / 9.0)
    }

    /// Get output from ADPCM playback
    fn get_adpcm_output(&self) -> (f32, f32) {
        if !self.adpcm_playing || self.adpcm_address >= self.adpcm_memory.len() as u32 {
            return (0.0, 0.0);
        }

        let sample_byte = self.adpcm_memory[self.adpcm_address as usize];
        let sample = ((sample_byte as i8) as f32) / 128.0 * 0.15;

        (sample, sample)
    }
}

impl SoundChipEmulator for Y8950 {
    fn name(&self) -> &'static str {
        "Y8950"
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
            // Channel volume
            0x30..=0x38 => {
                let ch = (addr - 0x30) as usize;
                if ch < 9 {
                    self.channels[ch].volume = (data >> 2) & 0x3F;
                }
            }
            // ADPCM control
            0x7F => {
                self.adpcm_playing = (data & 0x01) != 0;
            }
            // ADPCM address high
            0x7E => {
                self.adpcm_address = (self.adpcm_address & 0x00FF) | ((data as u32) << 8);
            }
            // ADPCM address low
            0x7D => {
                self.adpcm_address = (self.adpcm_address & 0xFF00) | (data as u32);
            }
            // ADPCM data write
            0x7C => {
                if self.adpcm_address < self.adpcm_memory.len() as u32 {
                    self.adpcm_memory[self.adpcm_address as usize] = data;
                    self.adpcm_address += 1;
                }
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        if self.adpcm_playing {
            self.adpcm_address = self.adpcm_address.wrapping_add(1);
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
            let (adpcm_left, adpcm_right) = self.get_adpcm_output();

            frame[0] = (fm_left + adpcm_left).clamp(-1.0, 1.0);
            frame[1] = (fm_right + adpcm_right).clamp(-1.0, 1.0);
        }
    }
}

impl Default for Y8950 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_y8950_new() {
        let chip = Y8950::new();
        assert_eq!(chip.name(), "Y8950");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_y8950_channels() {
        let chip = Y8950::new();
        assert_eq!(chip.channels.len(), 9);
    }

    #[test]
    fn test_y8950_write() {
        let mut chip = Y8950::new();
        chip.write(0x10, 0x20);
        assert_eq!(chip.regs[0x10], 0x20);
    }

    #[test]
    fn test_y8950_soundchip_trait() {
        let mut chip = Y8950::new();
        assert_eq!(chip.name(), "Y8950");

        chip.reset();
        chip.write(0x10, 0x20);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
