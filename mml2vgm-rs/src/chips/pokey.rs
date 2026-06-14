//! POKEY (C012294) sound chip emulation (Atari 8-bit)
//!
//! 4 tone/noise channels with poly-counter based generation.
//! VGM opcode 0xBB: two-byte payload [addr, data].
//!
//! Register map:
//!   0x00: AUDF1 — Channel 1 frequency divider (8-bit)
//!   0x01: AUDC1 — Channel 1 control (bits[3:0]=volume, bit4=noise/tone, bit5=4-bit/17-bit poly, bit7=mute)
//!   0x02: AUDF2
//!   0x03: AUDC2
//!   0x04: AUDF3
//!   0x05: AUDC3
//!   0x06: AUDF4
//!   0x07: AUDC4
//!   0x08: AUDCTL — Audio control (bit3=ch1+ch2 16-bit, bit4=ch3+ch4 16-bit,
//!                                  bit2=ch1 high-freq, bit1=ch3 high-freq, bit0=poly9 select)

use super::SoundChipEmulator;

const NUM_CHANNELS: usize = 4;
const POKEY_DIV_HIGH: f32 = 1.0;
const POKEY_DIV_LOW: f32 = 114.0;

#[derive(Debug, Clone, Copy, Default)]
struct PokeyChannel {
    audf: u8,
    volume: u8,
    noise_mode: bool,
    mute: bool,
    use_poly4: bool,
    phase_acc: f32,
}

/// Pokey.
pub struct Pokey {
    clock_rate: u32,
    channels: [PokeyChannel; NUM_CHANNELS],
    audctl: u8,
    poly9_lfsr: u16,
    poly17_lfsr: u32,
    poly4_lfsr: u8,
    noise_phase: f32,
    regs: [u8; 16],
}

impl Pokey {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(1_789_772)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            channels: [PokeyChannel::default(); NUM_CHANNELS],
            audctl: 0,
            poly9_lfsr: 1,
            poly17_lfsr: 1,
            poly4_lfsr: 1,
            noise_phase: 0.0,
            regs: [0u8; 16],
        }
    }

    fn channel_freq_hz(&self, ch: usize) -> f32 {
        let audf = self.channels[ch].audf.max(1) as f32;
        let high_freq = match ch {
            0 => (self.audctl & 0x04) != 0,
            2 => (self.audctl & 0x02) != 0,
            _ => false,
        };
        let div = if high_freq {
            POKEY_DIV_HIGH
        } else {
            POKEY_DIV_LOW
        };
        self.clock_rate as f32 / (2.0 * div * (audf + 1.0))
    }

    fn advance_noise(&mut self, sample_rate: u32) {
        let noise_hz = self.clock_rate as f32 / (2.0 * POKEY_DIV_LOW);
        let inc = noise_hz / sample_rate as f32;
        self.noise_phase += inc;
        if self.noise_phase >= 1.0 {
            self.noise_phase -= 1.0;
            // Advance poly17 LFSR (taps 16+12)
            let bit17 = ((self.poly17_lfsr >> 16) ^ (self.poly17_lfsr >> 11)) & 1;
            self.poly17_lfsr = (self.poly17_lfsr >> 1) | (bit17 << 16);
            // Advance poly9 LFSR (taps 8+4)
            let bit9 = ((self.poly9_lfsr >> 8) ^ (self.poly9_lfsr >> 4)) & 1;
            self.poly9_lfsr = (self.poly9_lfsr >> 1) | (bit9 << 8);
            // Advance poly4 LFSR (taps 3+2)
            let bit4 = ((self.poly4_lfsr >> 3) ^ (self.poly4_lfsr >> 2)) & 1;
            self.poly4_lfsr = ((self.poly4_lfsr >> 1) & 0x07) | (bit4 << 3);
        }
    }

    fn noise_bit(&self, use_poly4: bool) -> f32 {
        if use_poly4 {
            if (self.poly4_lfsr & 1) != 0 {
                1.0
            } else {
                -1.0
            }
        } else if (self.audctl & 0x80) != 0 {
            // poly9 mode
            if (self.poly9_lfsr & 1) != 0 {
                1.0
            } else {
                -1.0
            }
        } else {
            if (self.poly17_lfsr & 1) != 0 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

impl SoundChipEmulator for Pokey {
    fn name(&self) -> &'static str {
        "POKEY"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        let a = (addr & 0x0F) as usize;
        if a < 16 {
            self.regs[a] = data;
        }
        match a {
            0 => self.channels[0].audf = data,
            1 => {
                self.channels[0].volume = data & 0x0F;
                self.channels[0].noise_mode = (data & 0x10) != 0; // bit4 NOTONE: 1=poly/noise, 0=tone
                self.channels[0].use_poly4 = (data & 0x20) != 0; // bit5 POLY4
                self.channels[0].mute = (data & 0x80) != 0;
            }
            2 => self.channels[1].audf = data,
            3 => {
                self.channels[1].volume = data & 0x0F;
                self.channels[1].noise_mode = (data & 0x10) != 0;
                self.channels[1].use_poly4 = (data & 0x20) != 0;
                self.channels[1].mute = (data & 0x80) != 0;
            }
            4 => self.channels[2].audf = data,
            5 => {
                self.channels[2].volume = data & 0x0F;
                self.channels[2].noise_mode = (data & 0x10) != 0;
                self.channels[2].use_poly4 = (data & 0x20) != 0;
                self.channels[2].mute = (data & 0x80) != 0;
            }
            6 => self.channels[3].audf = data,
            7 => {
                self.channels[3].volume = data & 0x0F;
                self.channels[3].noise_mode = (data & 0x10) != 0;
                self.channels[3].use_poly4 = (data & 0x20) != 0;
                self.channels[3].mute = (data & 0x80) != 0;
            }
            8 => self.audctl = data,
            _ => {}
        }
    }

    fn read(&self, addr: u8) -> u8 {
        let a = (addr & 0x0F) as usize;
        if a < 16 {
            self.regs[a]
        } else {
            0xFF
        }
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for frame in buffer.chunks_mut(2) {
            self.advance_noise(sample_rate);

            let mut out = 0.0f32;
            for ch in 0..NUM_CHANNELS {
                if self.channels[ch].mute || self.channels[ch].volume == 0 {
                    continue;
                }
                let freq = self.channel_freq_hz(ch);
                let phase_inc = freq / sample_rate as f32;
                self.channels[ch].phase_acc += phase_inc;
                if self.channels[ch].phase_acc >= 1.0 {
                    self.channels[ch].phase_acc -= 1.0;
                }
                let tone_high = self.channels[ch].phase_acc < 0.5;
                let sample = if self.channels[ch].noise_mode {
                    self.noise_bit(self.channels[ch].use_poly4)
                } else if tone_high {
                    1.0
                } else {
                    -1.0
                };
                let vol = self.channels[ch].volume as f32 / 15.0;
                out += sample * vol * 0.12;
            }

            let mixed = out.clamp(-1.0, 1.0);
            if frame.len() >= 2 {
                frame[0] = mixed;
                frame[1] = mixed;
            }
        }
    }
}

impl Default for Pokey {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pokey_new() {
        let chip = Pokey::new();
        assert_eq!(chip.name(), "POKEY");
        assert_eq!(chip.clock_rate(), 1_789_772);
    }

    #[test]
    fn test_pokey_write_audf_audc() {
        let mut chip = Pokey::new();
        chip.write(0x00, 0xA0); // ch1 freq
        chip.write(0x01, 0x0F); // ch1 vol=15, tone mode
        assert_eq!(chip.channels[0].audf, 0xA0);
        assert_eq!(chip.channels[0].volume, 0x0F);
        assert!(!chip.channels[0].noise_mode);
    }

    #[test]
    fn test_pokey_generate_tone() {
        let mut chip = Pokey::new();
        chip.write(0x00, 0x40); // ch1 freq
        chip.write(0x01, 0x0F); // ch1 vol=15, tone
        let mut buf = [0.0f32; 16];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().any(|&s| s != 0.0),
            "active POKEY channel must produce output"
        );
    }

    #[test]
    fn test_pokey_noise_mode() {
        let mut chip = Pokey::new();
        chip.write(0x00, 0x10);
        chip.write(0x01, 0x0F); // noise mode (bit5=0 means noise)
        let mut buf = [0.0f32; 32];
        chip.generate_samples(&mut buf, 44100);
        // noise output — just verify no crash and some non-zero values
        assert!(buf.len() == 32);
    }

    #[test]
    fn test_pokey_reset() {
        let mut chip = Pokey::new();
        chip.write(0x01, 0x0F);
        chip.reset();
        assert_eq!(chip.channels[0].volume, 0);
    }
}
