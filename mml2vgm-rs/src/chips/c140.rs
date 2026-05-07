//! C140 sound chip emulation
//!
//! The C140 is a Capcom PCM sample playback chip used in arcade systems.
//! It provides 24 channels of PCM playback with volume and panning control.
//!
//! # Features
//! - 24 PCM channels
//! - 8-bit PCM samples
//! - Volume and panning per channel
//! - Adjustable sample rates per channel
//! - Loop support

use super::SoundChipEmulator;

/// PCM channel state
#[derive(Debug, Clone, Copy)]
struct PcmChannel {
    active: bool,
    position: u32,
    sample_rate: u16,
    volume: u8,
    pan: u8,
    loop_start: u32,
    loop_length: u32,
    loop_enabled: bool,
}

impl Default for PcmChannel {
    fn default() -> Self {
        Self {
            active: false,
            position: 0,
            sample_rate: 1,
            volume: 0,
            pan: 7,
            loop_start: 0,
            loop_length: 0,
            loop_enabled: false,
        }
    }
}

/// C140 chip emulator
pub struct C140 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Clock divider
    clock_divider: f64,

    /// Accumulated cycles
    accumulated_cycles: f64,

    /// All 24 PCM channels
    channels: [PcmChannel; 24],

    /// PCM data memory (1MB)
    pcm_memory: Vec<u8>,

    /// Register cache
    regs: [u8; 0x400],
}

impl C140 {
    /// Create a new C140 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(8_000_000)
    }

    /// Create a new C140 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            clock_divider: clock_rate as f64 / 44100.0,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 24],
            pcm_memory: vec![0; 1_048_576], // 1MB
            regs: [0; 0x400],
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }
}

impl SoundChipEmulator for C140 {
    fn name(&self) -> &'static str {
        "C140"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.regs[addr as usize] = data;

        let ch = (addr / 4) as usize;
        if ch >= 24 {
            return;
        }

        match addr % 4 {
            0 => {
                // Control / status
                self.channels[ch].active = (data & 0x80) != 0;
            }
            1 => {
                // Volume
                self.channels[ch].volume = data;
            }
            2 => {
                // Panning
                self.channels[ch].pan = data & 0x0F;
            }
            3 => {
                // Sample rate divisor
                self.channels[ch].sample_rate = (data as u16) << 2;
            }
            _ => {}
        }
    }

    fn read(&self, addr: u8) -> u8 {
        self.regs[addr as usize]
    }

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x08 {
            let len = data.len().min(self.pcm_memory.len());
            self.pcm_memory[..len].copy_from_slice(&data[..len]);
        }
    }

    fn clock(&mut self) {
        for ch in &mut self.channels {
            if ch.active && ch.sample_rate > 0 {
                ch.position += ch.sample_rate as u32;

                if ch.loop_enabled {
                    if ch.position >= ch.loop_start + ch.loop_length {
                        ch.position = ch.loop_start;
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

                    let pan_left = if ch.pan < 7 {
                        1.0 - ((7 - ch.pan) as f32 / 15.0)
                    } else {
                        1.0
                    };
                    let pan_right = if ch.pan > 7 {
                        1.0 - ((ch.pan - 7) as f32 / 15.0)
                    } else {
                        1.0
                    };

                    left += sample * pan_left;
                    right += sample * pan_right;
                }
            }

            frame[0] = (left / 24.0).clamp(-1.0, 1.0);
            frame[1] = (right / 24.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for C140 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c140_new() {
        let chip = C140::new();
        assert_eq!(chip.name(), "C140");
        assert_eq!(chip.clock_rate(), 8_000_000);
    }

    #[test]
    fn test_c140_channels() {
        let chip = C140::new();
        assert_eq!(chip.channels.len(), 24);
        for ch in &chip.channels {
            assert!(!ch.active);
        }
    }

    #[test]
    fn test_c140_write() {
        let mut chip = C140::new();
        chip.write(0x00, 0x80);
        assert!(chip.channels[0].active);
    }

    #[test]
    fn test_c140_soundchip_trait() {
        let mut chip = C140::new();
        assert_eq!(chip.name(), "C140");

        chip.reset();
        chip.write(0x00, 0x80);
        chip.write(0x01, 0xFF);
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
