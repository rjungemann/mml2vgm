//! Sega PCM sound chip emulation
//!
//! The Sega PCM chip is used in various Sega arcade systems for PCM sample playback.
//! It provides 16 channels of 8-bit PCM playback with independent volume and panning.
//!
//! VGM register map (16-bit address):
//! - 0x00-0x7F: 16 channels × 8 bytes each (ch * 8 + sub)
//!   - sub 0: loop_start low
//!   - sub 1: loop_start high
//!   - sub 2: start address page (addr bit 16+)
//!   - sub 3: end address page
//!   - sub 4: delta low (playback rate)
//!   - sub 5: delta high
//!   - sub 6: left volume (0-127)
//!   - sub 7: right volume (0-127), bit 7 = loop enable
//! - 0x86: channel active bitmask (bit N = channel N)

use super::SoundChipEmulator;

/// PCM channel state
#[derive(Debug, Clone, Copy)]
struct PcmChannel {
    active: bool,
    position: u32,
    delta: u16,
    vol_left: u8,
    vol_right: u8,
    start_addr: u32,
    end_addr: u32,
    loop_start: u32,
    loop_enabled: bool,
}

impl Default for PcmChannel {
    fn default() -> Self {
        Self {
            active: false,
            position: 0,
            delta: 1,
            vol_left: 0,
            vol_right: 0,
            start_addr: 0,
            end_addr: 0x10000,
            loop_start: 0,
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

    /// Register cache (16-bit address space)
    regs: Vec<u8>,
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
            clock_divider: clock_rate as f64 / 44100.0,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 16],
            pcm_memory: vec![0; 262_144],
            regs: vec![0; 0x100],
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }

    fn decode_register(&mut self, addr16: usize, data: u8) {
        if addr16 == 0x86 {
            for ch in 0..8usize {
                self.channels[ch].active = (data & (1 << ch)) == 0; // active when bit is CLEAR (inverted)
            }
            return;
        }

        let ch = addr16 / 8;
        if ch >= 16 {
            return;
        }

        match addr16 % 8 {
            0 => {
                self.channels[ch].loop_start = (self.channels[ch].loop_start & 0xFF00) | data as u32
            }
            1 => {
                self.channels[ch].loop_start =
                    (self.channels[ch].loop_start & 0x00FF) | ((data as u32) << 8)
            }
            2 => {
                self.channels[ch].start_addr = (data as u32) << 16;
                self.channels[ch].position = (data as u32) << 16;
            }
            3 => self.channels[ch].end_addr = (data as u32) << 16,
            4 => self.channels[ch].delta = (self.channels[ch].delta & 0xFF00) | data as u16,
            5 => {
                self.channels[ch].delta = (self.channels[ch].delta & 0x00FF) | ((data as u16) << 8)
            }
            6 => self.channels[ch].vol_left = data & 0x7F,
            7 => {
                self.channels[ch].vol_right = data & 0x7F;
                self.channels[ch].loop_enabled = (data & 0x80) != 0;
            }
            _ => {}
        }
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
        let addr16 = addr as usize;
        if addr16 < self.regs.len() {
            self.regs[addr16] = data;
        }
        self.decode_register(addr16, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let addr16 = ((port as usize) << 8) | addr as usize;
        if addr16 < self.regs.len() {
            self.regs[addr16] = data;
        }
        self.decode_register(addr16, data);
    }

    fn read(&self, addr: u8) -> u8 {
        let idx = addr as usize;
        if idx < self.regs.len() {
            self.regs[idx]
        } else {
            0
        }
    }

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x04 {
            let len = data.len().min(self.pcm_memory.len());
            self.pcm_memory[..len].copy_from_slice(&data[..len]);
        }
    }

    fn clock(&mut self) {
        for ch in &mut self.channels {
            if ch.active && ch.delta > 0 {
                ch.position += ch.delta as u32;
                if ch.end_addr > 0 && ch.position >= ch.end_addr {
                    if ch.loop_enabled {
                        ch.position = ch.start_addr + ch.loop_start;
                    } else {
                        ch.active = false;
                    }
                }
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        if sample_rate != self.sample_rate {
            self.set_sample_rate(sample_rate);
        }

        let mem_len = self.pcm_memory.len() as u32;

        for frame in buffer.chunks_mut(2) {
            self.accumulated_cycles += 1.0;
            while self.accumulated_cycles >= self.clock_divider {
                self.clock();
                self.accumulated_cycles -= self.clock_divider;
            }

            let mut left = 0.0f32;
            let mut right = 0.0f32;

            for ch in &self.channels {
                if ch.active && ch.position < mem_len {
                    let sample_byte = self.pcm_memory[ch.position as usize];
                    let sample = ((sample_byte as i8) as f32) / 128.0;

                    left += sample * (ch.vol_left as f32 / 127.0);
                    right += sample * (ch.vol_right as f32 / 127.0);
                }
            }

            frame[0] = (left / 8.0).clamp(-1.0, 1.0);
            frame[1] = (right / 8.0).clamp(-1.0, 1.0);
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
    fn test_segapcm_write_port_channel_decode() {
        let mut chip = SegaPCM::new();
        // Channel 0: set delta to 0x0100
        chip.write_port(0x00, 0x04, 0x00); // ch0 delta_lo
        chip.write_port(0x00, 0x05, 0x01); // ch0 delta_hi
        assert_eq!(chip.channels[0].delta, 0x0100);

        // Channel 1: set left vol
        chip.write_port(0x00, 0x0E, 0x40); // ch1 (offset 8) vol_left = 0x40
        assert_eq!(chip.channels[1].vol_left, 0x40);
    }

    #[test]
    fn test_segapcm_write_port_loop_flag() {
        let mut chip = SegaPCM::new();
        // bit 7 of vol_right = loop enable
        chip.write_port(0x00, 0x07, 0x80);
        assert!(chip.channels[0].loop_enabled);
        assert_eq!(chip.channels[0].vol_right, 0); // high bit stripped
    }

    #[test]
    fn test_segapcm_soundchip_trait() {
        let mut chip = SegaPCM::new();
        assert_eq!(chip.name(), "SegaPCM");

        chip.reset();
        chip.write_port(0x00, 0x04, 0x01); // ch0 delta_lo
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
