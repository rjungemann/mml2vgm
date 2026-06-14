//! K053260 (Konami arcade 4-channel 8-bit PCM chip) emulation
//!
//! VGM opcode 0xBA: two-byte payload [addr, data].
//!
//! Register map (per channel, 8 registers, channel N at 0x08*N):
//!   +0x00-0x02: Start address (24-bit little-endian)
//!   +0x03: Length lo
//!   +0x04: Length hi
//!   +0x05: Volume (0-127)
//!   +0x06: Pan (0=left, 7=center, 15=right)
//!   +0x07: Control (bit7=loop, bit6=ADPCM_mode)
//!   0x28: Key-on register (bit N = trigger channel N)
//!   0x2C: Global control

use super::SoundChipEmulator;

const NUM_CHANNELS: usize = 4;
const PCM_MEM_SIZE: usize = 512 * 1024; // 512 KB

#[derive(Debug, Clone, Copy, Default)]
struct K053260Channel {
    start_addr: u32,
    length: u32,
    volume: u8,
    pan: u8,
    looping: bool,
    active: bool,
    position: f32,
}

/// K053260.
pub struct K053260 {
    clock_rate: u32,
    channels: [K053260Channel; NUM_CHANNELS],
    pcm_memory: Vec<u8>,
    regs: [u8; 0x30],
}

impl K053260 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            channels: [K053260Channel::default(); NUM_CHANNELS],
            pcm_memory: vec![0u8; PCM_MEM_SIZE],
            regs: [0u8; 0x30],
        }
    }

    fn update_channel_from_regs(&mut self, ch: usize) {
        let base = ch * 8;
        let start = self.regs[base] as u32
            | ((self.regs[base + 1] as u32) << 8)
            | ((self.regs[base + 2] as u32) << 16);
        let length = self.regs[base + 3] as u32 | ((self.regs[base + 4] as u32) << 8);
        let volume = self.regs[base + 5] & 0x7F;
        let pan = self.regs[base + 6] & 0x0F;
        let control = self.regs[base + 7];
        self.channels[ch].start_addr = start;
        self.channels[ch].length = length;
        self.channels[ch].volume = volume;
        self.channels[ch].pan = pan;
        self.channels[ch].looping = (control & 0x80) != 0;
    }
}

impl SoundChipEmulator for K053260 {
    fn name(&self) -> &'static str {
        "K053260"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        self.channels = [K053260Channel::default(); NUM_CHANNELS];
        self.regs = [0u8; 0x30];
    }

    fn write(&mut self, addr: u8, data: u8) {
        let addr = addr as usize;
        if addr < 0x30 {
            self.regs[addr] = data;
        }
        match addr {
            // Channel register writes — update channel structs
            0x00..=0x1F => {
                let ch = addr / 8;
                if ch < NUM_CHANNELS {
                    self.update_channel_from_regs(ch);
                }
            }
            // Key-on: bit N = trigger channel N
            0x28 => {
                for ch in 0..NUM_CHANNELS {
                    if (data >> ch) & 1 != 0 {
                        self.channels[ch].position = self.channels[ch].start_addr as f32;
                        self.channels[ch].active = self.channels[ch].length > 0;
                    }
                }
            }
            _ => {}
        }
    }

    fn read(&self, addr: u8) -> u8 {
        let a = addr as usize;
        if a < 0x30 {
            self.regs[a]
        } else {
            0xFF
        }
    }

    fn clock(&mut self) {}

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x13 {
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
                let end = self.channels[ch].start_addr as usize + self.channels[ch].length as usize;
                if pos >= end.min(self.pcm_memory.len()) {
                    if self.channels[ch].looping {
                        self.channels[ch].position = self.channels[ch].start_addr as f32;
                    } else {
                        self.channels[ch].active = false;
                    }
                    continue;
                }
                let raw = self.pcm_memory[pos.min(self.pcm_memory.len() - 1)] as i8;
                let sample = raw as f32 / 128.0;
                let vol = self.channels[ch].volume as f32 / 127.0;
                let sample = sample * vol;
                // Pan: 0=full left, 7=center, 15=full right (linear)
                let pan_norm = self.channels[ch].pan as f32 / 15.0;
                left += sample * (1.0 - pan_norm);
                right += sample * pan_norm;

                self.channels[ch].position += 1.0;
            }

            if frame.len() >= 2 {
                frame[0] = (left / 4.0).clamp(-1.0, 1.0);
                frame[1] = (right / 4.0).clamp(-1.0, 1.0);
            }
        }
    }
}

impl Default for K053260 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k053260_new() {
        let chip = K053260::new();
        assert_eq!(chip.name(), "K053260");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_k053260_channels() {
        let mut chip = K053260::new();
        // Set ch0 registers
        chip.write(0x00, 0x00); // start lo
        chip.write(0x01, 0x00); // start mid
        chip.write(0x02, 0x00); // start hi
        chip.write(0x03, 0x10); // length lo = 16
        chip.write(0x04, 0x00); // length hi
        chip.write(0x05, 0x7F); // volume = 127
        chip.write(0x06, 0x07); // pan center
        chip.write(0x07, 0x00); // no loop
        assert_eq!(chip.channels[0].volume, 127);
        assert_eq!(chip.channels[0].length, 16);
    }

    #[test]
    fn test_k053260_key_on() {
        let mut chip = K053260::new();
        chip.write(0x03, 0x10); // ch0 length=16
        chip.write(0x28, 0x01); // key-on ch0
        assert!(chip.channels[0].active);
        assert_eq!(
            chip.channels[0].position,
            chip.channels[0].start_addr as f32
        );
    }

    #[test]
    fn test_k053260_load_pcm() {
        let mut chip = K053260::new();
        let data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        chip.load_pcm_data(0x13, &data);
        assert_eq!(chip.pcm_memory[0], 0);
        assert_eq!(chip.pcm_memory[127], 127);
        // Wrong block type — should not overwrite
        let zeros = vec![0u8; 256];
        chip.load_pcm_data(0x00, &zeros);
        assert_eq!(
            chip.pcm_memory[127], 127,
            "wrong block type must not overwrite PCM memory"
        );
    }

    #[test]
    fn test_k053260_soundchip_trait() {
        let mut chip = K053260::new();
        assert!(chip.is_initialized());
        assert_eq!(chip.read(0x05), 0x00);
        chip.clock(); // must not panic
                      // Generate silence when no channels active
        let mut buf = [0.0f32; 8];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().all(|&s| s == 0.0),
            "no active channels should produce silence"
        );
    }
}
