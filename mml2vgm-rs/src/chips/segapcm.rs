//! Sega PCM sound chip emulation
//!
//! The Sega PCM chip is used in various Sega arcade systems for PCM sample playback.
//! It provides 16 channels of 8-bit PCM playback with volume control and panning.
//!
//! # Features
//! - 16 PCM channels
//! - 8-bit PCM samples
//! - Independent volume control per channel
//! - Stereo panning
//! - Adjustable playback rate per channel

use super::SoundChipEmulator;

/// PCM channel state
#[derive(Debug, Clone, Copy)]
struct PcmChannel {
    active: bool,
    position: u32,
    playback_rate: u16,
    volume: u8,
    pan: u8,
    loop_start: u32,
    loop_end: u32,
    loop_enabled: bool,
}

impl Default for PcmChannel {
    fn default() -> Self {
        Self {
            active: false,
            position: 0,
            playback_rate: 1,
            volume: 0,
            pan: 8,
            loop_start: 0,
            loop_end: 0,
            loop_enabled: false,
        }
    }
}

/// Sega PCM chip emulator
pub struct SegaPCM {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Clock divider
    clock_divider: f64,

    /// Accumulated cycles
    accumulated_cycles: f64,

    /// All 16 PCM channels
    channels: [PcmChannel; 16],

    /// PCM data memory (256KB)
    pcm_memory: Vec<u8>,

    /// Register cache
    regs: [u8; 0x400],
}

impl SegaPCM {
    /// Create a new SegaPCM emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(15_468_750)
    }

    /// Create a new SegaPCM emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            clock_divider: 0.0,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 16],
            pcm_memory: vec![0; 262_144], // 256KB
            regs: [0; 0x400],
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }
}

impl SoundChipEmulator for SegaPCM {
    fn name(&self) -> &'static str {
        "SegaPCM"
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
            // Channel select and control
            0x00 => {
                let ch = (data & 0x0F) as usize;
                if ch < 16 {
                    self.channels[ch].active = (data & 0x80) != 0;
                }
            }
            // Volume
            0x01 => {
                if let Some(ch) = self.channels.iter_mut().find(|c| c.active) {
                    ch.volume = data;
                }
            }
            // Panning
            0x02 => {
                if let Some(ch) = self.channels.iter_mut().find(|c| c.active) {
                    ch.pan = data & 0x0F;
                }
            }
            // Playback rate
            0x03..=0x04 => {
                if let Some(ch) = self.channels.iter_mut().find(|c| c.active) {
                    if addr == 0x03 {
                        ch.playback_rate = (ch.playback_rate & 0xFF00) | (data as u16);
                    } else {
                        ch.playback_rate = (ch.playback_rate & 0x00FF) | ((data as u16) << 8);
                    }
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
            if ch.active && ch.playback_rate > 0 {
                ch.position += ch.playback_rate as u32;
                if ch.loop_enabled && ch.position >= ch.loop_end {
                    ch.position = ch.loop_start;
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

                    let pan_left = if ch.pan < 8 {
                        1.0
                    } else {
                        1.0 - ((ch.pan - 8) as f32 / 8.0)
                    };
                    let pan_right = if ch.pan > 8 {
                        1.0
                    } else {
                        (ch.pan as f32 / 8.0)
                    };

                    left += sample * pan_left;
                    right += sample * pan_right;
                }
            }

            frame[0] = (left / 16.0).clamp(-1.0, 1.0);
            frame[1] = (right / 16.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for SegaPCM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segapcm_new() {
        let chip = SegaPCM::new();
        assert_eq!(chip.name(), "SegaPCM");
        assert_eq!(chip.clock_rate(), 15_468_750);
    }

    #[test]
    fn test_segapcm_channels() {
        let chip = SegaPCM::new();
        assert_eq!(chip.channels.len(), 16);
        for ch in &chip.channels {
            assert!(!ch.active);
        }
    }

    #[test]
    fn test_segapcm_write() {
        let mut chip = SegaPCM::new();
        chip.write(0x00, 0x80);
        assert!(chip.channels[0].active);
    }

    #[test]
    fn test_segapcm_soundchip_trait() {
        let mut chip = SegaPCM::new();
        assert_eq!(chip.name(), "SegaPCM");

        chip.reset();
        chip.write(0x00, 0x80);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
