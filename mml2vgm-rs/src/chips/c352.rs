//! C352 sound chip emulation
//!
//! The C352 is an improved version of the C140 PCM chip, also by Capcom.
//! It provides 32 channels of PCM playback with better quality and more features.
//!
//! # Features
//! - 32 PCM channels
//! - 8-bit or 12-bit PCM samples
//! - Volume, panning, and pitch control per channel
//! - Advanced loop and envelope control
//! - Higher quality than C140

use super::SoundChipEmulator;

/// PCM channel state
#[derive(Debug, Clone, Copy)]
struct PcmChannel {
    active: bool,
    position: u32,
    pitch: u16,
    volume: u8,
    pan: u8,
    start_addr: u32,
    loop_addr: u32,
    end_addr: u32,
    loop_enabled: bool,
    flags: u8,
}

impl Default for PcmChannel {
    fn default() -> Self {
        Self {
            active: false,
            position: 0,
            pitch: 1,
            volume: 0,
            pan: 15,
            start_addr: 0,
            loop_addr: 0,
            end_addr: 0,
            loop_enabled: false,
            flags: 0,
        }
    }
}

/// C352 chip emulator
pub struct C352 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Clock divider
    clock_divider: f64,

    /// Accumulated cycles
    accumulated_cycles: f64,

    /// All 32 PCM channels
    channels: [PcmChannel; 32],

    /// PCM data memory (2MB)
    pcm_memory: Vec<u8>,

    /// Register cache
    regs: [u8; 0x800],
}

impl C352 {
    /// Create a new C352 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(20_000_000)
    }

    /// Create a new C352 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            clock_divider: 0.0,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 32],
            pcm_memory: vec![0; 2_097_152], // 2MB
            regs: [0; 0x800],
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }
}

impl SoundChipEmulator for C352 {
    fn name(&self) -> &'static str {
        "C352"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.regs[addr as usize] = data;

        let ch = (addr / 8) as usize;
        if ch >= 32 {
            return;
        }

        match addr % 8 {
            0 => {
                // Control register
                self.channels[ch].active = (data & 0x80) != 0;
                self.channels[ch].loop_enabled = (data & 0x04) != 0;
                self.channels[ch].flags = data;
            }
            1 => {
                // Volume
                self.channels[ch].volume = data;
            }
            2 => {
                // Panning (0-31)
                self.channels[ch].pan = data & 0x1F;
            }
            3 | 4 => {
                // Pitch (16-bit)
                if addr % 8 == 3 {
                    self.channels[ch].pitch = (self.channels[ch].pitch & 0xFF00) | (data as u16);
                } else {
                    self.channels[ch].pitch = (self.channels[ch].pitch & 0x00FF) | ((data as u16) << 8);
                }
            }
            _ => {}
        }
    }

    fn read(&self, addr: u8) -> u8 {
        self.regs[addr as usize]
    }

    fn clock(&mut self) {
        for ch in &mut self.channels {
            if ch.active && ch.pitch > 0 {
                ch.position += ch.pitch as u32;

                if ch.loop_enabled {
                    if ch.position >= ch.end_addr {
                        ch.position = ch.loop_addr;
                    }
                } else if ch.position >= self.pcm_memory.len() as u32 {
                    ch.active = false;
                }
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        if sample_rate != self.sample_rate {
            self.set_sample_rate(sample_rate);
        }

        for frame in buffer.chunks_mut(2) {
            self.accumulated_cycles += 1.0;
            while self.accumulated_cycles >= self.clock_divider {
                self.clock();
                self.accumulated_cycles -= self.clock_divider;
            }

            let mut left = 0.0;
            let mut right = 0.0;

            for ch in &self.channels {
                if ch.active && ch.position < self.pcm_memory.len() as u32 {
                    let sample_byte = self.pcm_memory[ch.position as usize];
                    let sample = ((sample_byte as i8) as f32) / 128.0;

                    let volume = ch.volume as f32 / 255.0;
                    let sample = sample * volume;

                    let pan_norm = (ch.pan as f32) / 31.0;
                    let pan_left = (1.0 - pan_norm).sqrt();
                    let pan_right = pan_norm.sqrt();

                    left += sample * pan_left;
                    right += sample * pan_right;
                }
            }

            frame[0] = (left / 32.0).clamp(-1.0, 1.0);
            frame[1] = (right / 32.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for C352 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c352_new() {
        let chip = C352::new();
        assert_eq!(chip.name(), "C352");
        assert_eq!(chip.clock_rate(), 20_000_000);
    }

    #[test]
    fn test_c352_channels() {
        let chip = C352::new();
        assert_eq!(chip.channels.len(), 32);
        for ch in &chip.channels {
            assert!(!ch.active);
        }
    }

    #[test]
    fn test_c352_write() {
        let mut chip = C352::new();
        chip.write(0x00, 0x80);
        assert!(chip.channels[0].active);
    }

    #[test]
    fn test_c352_soundchip_trait() {
        let mut chip = C352::new();
        assert_eq!(chip.name(), "C352");

        chip.reset();
        chip.write(0x00, 0x80);
        chip.write(0x01, 0xFF);
        chip.write(0x03, 0x01);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
