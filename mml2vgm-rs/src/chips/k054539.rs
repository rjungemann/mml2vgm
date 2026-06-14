//! K054539 (Konami arcade 8-channel 16-bit PCM chip) emulation
//!
//! VGM opcode 0xD3: three-byte payload [port, addr, data].
//! Port selects chip instance (ignored here — single chip).
//!
//! Register map (8 channels × 0x20 bytes = 0x100 bytes):
//!   ch_base = ch * 0x20
//!   +0x00: pitch lo
//!   +0x01: pitch hi
//!   +0x02: start addr lo
//!   +0x03: start addr mid
//!   +0x04: start addr hi
//!   +0x05: volume (0-255)
//!   +0x06: pan (0=left, 127=center, 255=right)
//!   +0x07: control (bit0=active, bit1=loop, bit4=reverse)
//!   +0x08: end addr lo
//!   +0x09: end addr mid
//!   +0x0A: end addr hi
//!   +0x0C: loop addr lo
//!   +0x0D: loop addr mid
//!   +0x0E: loop addr hi

use super::SoundChipEmulator;

const NUM_CHANNELS: usize = 8;
const PCM_MEM_SIZE: usize = 2 * 1024 * 1024; // 2 MB

#[derive(Debug, Clone, Copy, Default)]
struct K054539Channel {
    pitch: u16,
    start_addr: u32,
    end_addr: u32,
    loop_addr: u32,
    volume: u8,
    pan: u8,
    active: bool,
    looping: bool,
    position: f32,
}

/// K054539.
pub struct K054539 {
    clock_rate: u32,
    channels: [K054539Channel; NUM_CHANNELS],
    pcm_memory: Vec<u8>,
    regs: [u8; 0x100],
}

impl K054539 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(18_432_000)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            channels: [K054539Channel::default(); NUM_CHANNELS],
            pcm_memory: vec![0u8; PCM_MEM_SIZE],
            regs: [0u8; 0x100],
        }
    }

    fn update_channel_from_regs(&mut self, ch: usize) {
        let base = ch * 0x20;
        self.channels[ch].pitch = self.regs[base] as u16 | ((self.regs[base + 1] as u16) << 8);
        self.channels[ch].start_addr = self.regs[base + 2] as u32
            | ((self.regs[base + 3] as u32) << 8)
            | ((self.regs[base + 4] as u32) << 16);
        self.channels[ch].volume = self.regs[base + 5];
        self.channels[ch].pan = self.regs[base + 6];
        let ctrl = self.regs[base + 7];
        self.channels[ch].active = (ctrl & 0x01) != 0;
        self.channels[ch].looping = (ctrl & 0x02) != 0;
        self.channels[ch].end_addr = self.regs[base + 8] as u32
            | ((self.regs[base + 9] as u32) << 8)
            | ((self.regs[base + 10] as u32) << 16);
        self.channels[ch].loop_addr = self.regs[base + 12] as u32
            | ((self.regs[base + 13] as u32) << 8)
            | ((self.regs[base + 14] as u32) << 16);
    }
}

impl SoundChipEmulator for K054539 {
    fn name(&self) -> &'static str {
        "K054539"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        self.channels = [K054539Channel::default(); NUM_CHANNELS];
        self.regs = [0u8; 0x100];
    }

    fn write(&mut self, addr: u8, data: u8) {
        let idx = addr as usize;
        if idx < 0x100 {
            self.regs[idx] = data;
            let ch = idx / 0x20;
            if ch < NUM_CHANNELS {
                self.update_channel_from_regs(ch);
                // Reset position to start if active bit just set
                if idx % 0x20 == 0x07 && (data & 0x01) != 0 {
                    self.channels[ch].position = self.channels[ch].start_addr as f32;
                }
            }
        }
    }

    fn write_port(&mut self, _port: u8, addr: u8, data: u8) {
        self.write(addr, data);
    }

    fn read(&self, addr: u8) -> u8 {
        let idx = addr as usize;
        if idx < 0x100 {
            self.regs[idx]
        } else {
            0xFF
        }
    }

    fn clock(&mut self) {}

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x12 {
            let copy_len = data.len().min(self.pcm_memory.len());
            self.pcm_memory[..copy_len].copy_from_slice(&data[..copy_len]);
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], _sample_rate: u32) {
        for frame in buffer.chunks_mut(2) {
            let mut left = 0.0f32;
            let mut right = 0.0f32;

            for ch in 0..NUM_CHANNELS {
                if !self.channels[ch].active {
                    continue;
                }
                let pos = self.channels[ch].position as usize;
                let end = self.channels[ch].end_addr as usize;
                if pos >= end.min(self.pcm_memory.len()) {
                    if self.channels[ch].looping {
                        self.channels[ch].position = self.channels[ch].loop_addr as f32;
                    } else {
                        self.channels[ch].active = false;
                    }
                    continue;
                }
                let raw = self.pcm_memory[pos.min(self.pcm_memory.len() - 1)] as i8;
                let sample = raw as f32 / 128.0;
                let vol = self.channels[ch].volume as f32 / 255.0;
                let sample = sample * vol;
                // Pan: 0=full left, 255=full right
                let pan_norm = self.channels[ch].pan as f32 / 255.0;
                left += sample * (1.0 - pan_norm);
                right += sample * pan_norm;

                let step = (self.channels[ch].pitch as f32 / 256.0).max(1.0);
                self.channels[ch].position += step;
            }

            if frame.len() >= 2 {
                frame[0] = (left / NUM_CHANNELS as f32).clamp(-1.0, 1.0);
                frame[1] = (right / NUM_CHANNELS as f32).clamp(-1.0, 1.0);
            }
        }
    }
}

impl Default for K054539 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k054539_new() {
        let chip = K054539::new();
        assert_eq!(chip.name(), "K054539");
        assert_eq!(chip.clock_rate(), 18_432_000);
    }

    #[test]
    fn test_k054539_channels() {
        let mut chip = K054539::new();
        let base = 0x00usize; // channel 0 base
                              // Write pitch
        chip.write(base as u8, 0x00); // pitch lo
        chip.write(base as u8 + 1, 0x01); // pitch hi → pitch = 0x0100 = 256
                                          // Write volume
        chip.write(base as u8 + 5, 0xFF); // volume = 255
                                          // Write pan
        chip.write(base as u8 + 6, 0x7F); // pan center
        assert_eq!(chip.channels[0].pitch, 0x0100);
        assert_eq!(chip.channels[0].volume, 0xFF);
        assert_eq!(chip.channels[0].pan, 0x7F);
    }

    #[test]
    fn test_k054539_write_port() {
        let mut chip = K054539::new();
        // write_port ignores port, routes to write
        chip.write_port(0, 0x05, 0x80); // ch0 volume via port 0
        chip.write_port(1, 0x05, 0x40); // overwrite same addr via port 1
        assert_eq!(chip.channels[0].volume, 0x40);
    }

    #[test]
    fn test_k054539_soundchip_trait() {
        let mut chip = K054539::new();
        assert!(chip.is_initialized());
        assert_eq!(chip.read(0x05), 0x00);
        chip.clock(); // must not panic
                      // Load PCM with wrong block type — must not overwrite
        let ones = vec![0xFFu8; 16];
        chip.load_pcm_data(0x00, &ones);
        assert_eq!(
            chip.pcm_memory[0], 0x00,
            "wrong block type must not overwrite PCM memory"
        );
        // Load PCM with correct block type
        chip.load_pcm_data(0x12, &ones);
        assert_eq!(chip.pcm_memory[0], 0xFF);
        // Generate silence when no channels active
        let mut buf = [0.0f32; 8];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().all(|&s| s == 0.0),
            "no active channels should produce silence"
        );
    }
}
