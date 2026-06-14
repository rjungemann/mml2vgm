//! RF5C164 PCM chip emulation
//!
//! The RF5C164 is a PCM sample playback chip used in the Sega Mega CD / Sega CD.
//! It provides 8 channels of 8-bit PCM playback with ADPCM compression support.
//!
//! # Features
//! - 8 PCM channels
//! - 8-bit linear PCM or 4-bit ADPCM
//! - Independent volume control per channel
//! - Stereo panning
//! - Sample rate control per channel
//!
//! This is a placeholder implementation that will be fully implemented
//! in a future phase.

use super::SoundChipEmulator;

/// PCM channel state
#[derive(Debug, Clone, Copy)]
pub struct PcmChannel {
    /// Channel is active (playing)
    pub active: bool,
    /// Current position in PCM data
    pub position: usize,
    /// Sample rate divider
    pub sample_rate_divider: u16,
    /// Current sample rate counter
    pub sample_rate_counter: u16,
    /// Volume (0-255)
    pub volume: u8,
    /// Panning (0 = full left, 15 = full right, 7-8 = center)
    pub pan: u8,
    /// Start address
    pub start_addr: u32,
    /// End address
    pub end_addr: u32,
    /// Current address
    pub current_addr: u32,
    /// Loop flag
    pub loop_flag: bool,
}

impl Default for PcmChannel {
    fn default() -> Self {
        Self {
            active: false,
            position: 0,
            sample_rate_divider: 1,
            sample_rate_counter: 0,
            volume: 0,
            pan: 8, // Center
            start_addr: 0,
            end_addr: 0,
            current_addr: 0,
            loop_flag: false,
        }
    }
}

/// RF5C164 chip emulator (placeholder)
pub struct RF5C164 {
    /// Master clock rate in Hz (default: 7,670,453 Hz)
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Clock divider
    clock_divider: f64,

    /// Accumulated cycles
    accumulated_cycles: f64,

    /// All 8 PCM channels
    channels: [PcmChannel; 8],

    /// PCM data memory (2MB max, but typically 1MB on Mega CD)
    pcm_memory: Vec<u8>,

    /// Register cache
    regs: [u8; 0x100],
}

impl RF5C164 {
    /// Create a new RF5C164 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(7_670_453)
    }

    /// Create a new RF5C164 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            clock_divider: clock_rate as f64 / 44100.0,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 8],
            pcm_memory: vec![0; 1_048_576], // 1MB default
            regs: [0; 0x100],
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }
}

impl SoundChipEmulator for RF5C164 {
    fn name(&self) -> &'static str {
        "RF5C164"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        // Store in register cache
        self.regs[addr as usize] = data;

        // Handle register writes
        match addr {
            // Channel select and control
            0x00 => {
                let ch = (data & 0x07) as usize;
                if ch < 8 {
                    self.channels[ch].active = (data & 0x80) != 0;
                }
            }
            // PCM data write
            0x01 => {
                // Write PCM sample to current address
                if let Some(ch_data) = self.channels.iter_mut().find(|c| c.active) {
                    if ch_data.current_addr < self.pcm_memory.len() as u32 {
                        self.pcm_memory[ch_data.current_addr as usize] = data;
                        ch_data.current_addr += 1;
                    }
                }
            }
            // Sample rate divider
            0x02..=0x09 => {
                let ch = (addr - 0x02) as usize;
                if ch < 8 {
                    self.channels[ch].sample_rate_divider = (data as u16) << 2;
                }
            }
            // Volume control
            0x0A..=0x11 => {
                let ch = (addr - 0x0A) as usize;
                if ch < 8 {
                    self.channels[ch].volume = data;
                }
            }
            // Panning
            0x12..=0x19 => {
                let ch = (addr - 0x12) as usize;
                if ch < 8 {
                    self.channels[ch].pan = data & 0x0F;
                }
            }
            // Start address
            0x1A..=0x27 => {
                let ch = ((addr - 0x1A) / 2) as usize;
                if ch < 8 {
                    if addr & 1 == 0 {
                        self.channels[ch].start_addr =
                            (self.channels[ch].start_addr & 0xFF00) | (data as u32);
                    } else {
                        self.channels[ch].start_addr =
                            (self.channels[ch].start_addr & 0x00FF) | ((data as u32) << 8);
                    }
                }
            }
            _ => {}
        }
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let addr16 = ((port as u16) << 8) | addr as u16;
        match addr16 {
            // PCM memory region: direct write to sample RAM
            0x1000..=0xFFFF => {
                let mem_addr = (addr16 - 0x1000) as usize;
                if mem_addr < self.pcm_memory.len() {
                    self.pcm_memory[mem_addr] = data;
                }
            }
            // Control register region: delegate to write()
            _ => self.write(addr, data),
        }
    }

    fn read(&self, addr: u8) -> u8 {
        // Return register cache
        self.regs[addr as usize]
    }

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x02 {
            let len = data.len().min(self.pcm_memory.len());
            self.pcm_memory[..len].copy_from_slice(&data[..len]);
        }
    }

    fn clock(&mut self) {
        // Update all channels
        for ch in &mut self.channels {
            if ch.active && ch.sample_rate_divider > 0 {
                ch.sample_rate_counter += 1;
                if ch.sample_rate_counter >= ch.sample_rate_divider {
                    ch.sample_rate_counter = 0;
                    if ch.current_addr < ch.end_addr {
                        ch.current_addr += 1;
                    } else if ch.loop_flag {
                        ch.current_addr = ch.start_addr;
                    } else {
                        ch.active = false;
                    }
                }
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        // Update sample rate
        if sample_rate != self.sample_rate {
            self.set_sample_rate(sample_rate);
        }

        // Fill buffer with samples from all channels
        for frame in buffer.chunks_mut(2) {
            // Update clock
            self.accumulated_cycles += 1.0;
            while self.accumulated_cycles >= self.clock_divider {
                self.clock();
                self.accumulated_cycles -= self.clock_divider;
            }

            let mut left = 0.0;
            let mut right = 0.0;

            // Mix all active channels
            for ch in &self.channels {
                if ch.active && ch.current_addr < self.pcm_memory.len() as u32 {
                    // Read sample from PCM memory (8-bit signed)
                    let sample_byte = self.pcm_memory[ch.current_addr as usize];
                    let sample = ((sample_byte as i8) as f32) / 128.0;

                    // Apply volume
                    let volume = ch.volume as f32 / 255.0;
                    let sample = sample * volume;

                    // Apply panning
                    let pan_left = if ch.pan < 8 {
                        1.0
                    } else {
                        1.0 - ((ch.pan - 8) as f32 / 8.0)
                    };
                    let pan_right = if ch.pan > 8 { 1.0 } else { ch.pan as f32 / 8.0 };

                    left += sample * pan_left;
                    right += sample * pan_right;
                }
            }

            // Normalize
            frame[0] = (left / 8.0).clamp(-1.0, 1.0);
            frame[1] = (right / 8.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for RF5C164 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rf5c164_new() {
        let chip = RF5C164::new();
        assert_eq!(chip.name(), "RF5C164");
        assert_eq!(chip.clock_rate(), 7_670_453);
    }

    #[test]
    fn test_rf5c164_channels() {
        let chip = RF5C164::new();
        assert_eq!(chip.channels.len(), 8);
        for ch in &chip.channels {
            assert!(!ch.active);
        }
    }

    #[test]
    fn test_rf5c164_write_control() {
        let mut chip = RF5C164::new();
        chip.write(0x00, 0x80); // Enable channel 0
        assert!(chip.channels[0].active);
    }

    #[test]
    fn test_rf5c164_soundchip_trait() {
        let mut chip = RF5C164::new();

        assert_eq!(chip.name(), "RF5C164");
        assert_eq!(chip.clock_rate(), 7_670_453);

        chip.reset();
        chip.write(0x00, 0x80); // Enable channel 0
        chip.write(0x02, 0x01); // Sample rate divider
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        // Buffer should be filled without panicking
        assert_eq!(buffer.len(), 4);
    }
}
