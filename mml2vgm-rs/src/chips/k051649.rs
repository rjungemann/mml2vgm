//! K051649 (SCC) / K052539 (SCC+) wavetable sound chip emulation (Konami)
//!
//! 5 channels, each with a 32-byte signed waveform, 12-bit frequency, and 4-bit volume.
//! VGM opcode 0xD2: three-byte payload [port, addr, data].
//!   port 0 = SCC (channels 0-4, wave bank shared for ch3+ch4)
//!   port 1 = SCC+ (independent waveform RAM for all 5 channels)
//!
//! Register map:
//!   0x00-0x1F: Channel 0 waveform (32 signed bytes)
//!   0x20-0x3F: Channel 1 waveform
//!   0x40-0x5F: Channel 2 waveform
//!   0x60-0x7F: Channel 3 waveform (in SCC mode ch4 reads this bank too)
//!   0x80-0x9F: Channel 4 waveform (SCC+ only)
//!   0xA0-0xA1: Channel 0 frequency (lo, hi — 12-bit)
//!   0xA2-0xA3: Channel 1 frequency
//!   0xA4-0xA5: Channel 2 frequency
//!   0xA6-0xA7: Channel 3 frequency
//!   0xA8-0xA9: Channel 4 frequency
//!   0xAA:      Channel 0 volume (bits[3:0])
//!   0xAB:      Channel 1 volume
//!   0xAC:      Channel 2 volume
//!   0xAD:      Channel 3 volume
//!   0xAE:      Channel 4 volume
//!   0xAF:      Key-on bitmask (bit N = channel N enable)

use super::SoundChipEmulator;

const NUM_CHANNELS: usize = 5;
const WAVE_LEN: usize = 32;

#[derive(Debug, Clone, Copy)]
struct SccChannel {
    frequency: u16,
    volume: u8,
    enabled: bool,
    phase_acc: f32,
}

impl Default for SccChannel {
    fn default() -> Self {
        Self { frequency: 0, volume: 0, enabled: false, phase_acc: 0.0 }
    }
}

pub struct K051649 {
    clock_rate: u32,
    channels: [SccChannel; NUM_CHANNELS],
    waveforms: [[i8; WAVE_LEN]; NUM_CHANNELS],
    plus_mode: bool,
}

impl K051649 {
    pub fn new() -> Self {
        Self::with_clock_rate(1_789_772)
    }

    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            channels: [SccChannel::default(); NUM_CHANNELS],
            waveforms: [[0i8; WAVE_LEN]; NUM_CHANNELS],
            plus_mode: false,
        }
    }

    fn apply_write(&mut self, addr: u8, data: u8) {
        match addr {
            0x00..=0x9F => {
                let ch = (addr as usize) / WAVE_LEN;
                let pos = (addr as usize) % WAVE_LEN;
                if ch < NUM_CHANNELS {
                    self.waveforms[ch][pos] = data as i8;
                    if ch == 3 && !self.plus_mode {
                        // In SCC mode ch4 shares ch3's waveform bank
                        self.waveforms[4][pos] = data as i8;
                    }
                }
            }
            0xA0..=0xA9 => {
                let idx = (addr - 0xA0) as usize;
                let ch = idx / 2;
                if ch < NUM_CHANNELS {
                    if idx % 2 == 0 {
                        self.channels[ch].frequency =
                            (self.channels[ch].frequency & 0xFF00) | data as u16;
                    } else {
                        self.channels[ch].frequency =
                            (self.channels[ch].frequency & 0x00FF) | ((data as u16 & 0x0F) << 8);
                    }
                }
            }
            0xAA..=0xAE => {
                let ch = (addr - 0xAA) as usize;
                self.channels[ch].volume = data & 0x0F;
            }
            0xAF => {
                for ch in 0..NUM_CHANNELS {
                    self.channels[ch].enabled = (data >> ch) & 1 != 0;
                }
            }
            _ => {}
        }
    }
}

impl SoundChipEmulator for K051649 {
    fn name(&self) -> &'static str { "K051649 (SCC)" }
    fn clock_rate(&self) -> u32 { self.clock_rate }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.apply_write(addr, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        if port == 1 {
            self.plus_mode = true;
        }
        self.apply_write(addr, data);
    }

    fn read(&self, _addr: u8) -> u8 { 0xFF }
    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for frame in buffer.chunks_mut(2) {
            let mut out = 0.0f32;
            for ch in 0..NUM_CHANNELS {
                if !self.channels[ch].enabled || self.channels[ch].volume == 0 { continue; }
                let period = self.channels[ch].frequency.max(1) as f32;
                let freq_hz = self.clock_rate as f32 / (WAVE_LEN as f32 * period);
                let phase_inc = freq_hz / sample_rate as f32;
                self.channels[ch].phase_acc += phase_inc;
                if self.channels[ch].phase_acc >= 1.0 {
                    self.channels[ch].phase_acc -= 1.0;
                }
                let wave_idx = (self.channels[ch].phase_acc * WAVE_LEN as f32) as usize % WAVE_LEN;
                let sample = self.waveforms[ch][wave_idx] as f32 / 128.0;
                let vol = self.channels[ch].volume as f32 / 15.0;
                out += sample * vol * 0.15;
            }
            let mixed = (out / NUM_CHANNELS as f32).clamp(-1.0, 1.0);
            if frame.len() >= 2 {
                frame[0] = mixed;
                frame[1] = mixed;
            }
        }
    }
}

impl Default for K051649 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k051649_new() {
        let chip = K051649::new();
        assert_eq!(chip.name(), "K051649 (SCC)");
        assert_eq!(chip.clock_rate(), 1_789_772);
    }

    #[test]
    fn test_k051649_waveform_write() {
        let mut chip = K051649::new();
        chip.write(0x00, 0x7F); // ch0 waveform byte 0
        chip.write(0x01, 0x40); // ch0 waveform byte 1
        assert_eq!(chip.waveforms[0][0], 0x7F_u8 as i8);
        assert_eq!(chip.waveforms[0][1], 0x40);
    }

    #[test]
    fn test_k051649_frequency_write() {
        let mut chip = K051649::new();
        chip.write(0xA0, 0x34); // ch0 freq lo
        chip.write(0xA1, 0x05); // ch0 freq hi
        assert_eq!(chip.channels[0].frequency, 0x534);
    }

    #[test]
    fn test_k051649_key_on_and_generate() {
        let mut chip = K051649::new();
        // Set sawtooth waveform on ch0
        for i in 0..32u8 {
            chip.write(i, i.wrapping_mul(8));
        }
        chip.write(0xA0, 0x80); // freq lo
        chip.write(0xA1, 0x00); // freq hi
        chip.write(0xAA, 0x0F); // vol = 15
        chip.write(0xAF, 0x01); // ch0 on
        let mut buf = [0.0f32; 8];
        chip.generate_samples(&mut buf, 44100);
        assert!(buf.iter().any(|&s| s != 0.0), "active SCC channel must produce output");
    }

    #[test]
    fn test_k051649_scc_ch4_shares_ch3_waveform() {
        let mut chip = K051649::new();
        // Write to ch3 waveform region — in SCC mode ch4 should see same data
        chip.write(0x60, 0x55); // ch3 waveform byte 0
        assert_eq!(chip.waveforms[3][0], 0x55_u8 as i8);
        assert_eq!(chip.waveforms[4][0], 0x55_u8 as i8, "ch4 should mirror ch3 in SCC mode");
    }

    #[test]
    fn test_k051649_reset() {
        let mut chip = K051649::new();
        chip.write(0xAF, 0x1F);
        chip.reset();
        assert!(!chip.channels[0].enabled);
    }
}
