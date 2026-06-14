//! AY-3-8910 / YM2149F PSG emulation
//!
//! 3-channel square wave generator with noise and hardware envelope.
//! VGM opcode 0xA0: two-byte payload [addr, data] (port select in top bit of addr).
//!
//! Register map (16 regs):
//!   0x00-0x01: Channel A period (12-bit: lo=0x00, hi=0x01 bits[3:0])
//!   0x02-0x03: Channel B period
//!   0x04-0x05: Channel C period
//!   0x06:      Noise period (5-bit)
//!   0x07:      Mixer: bits[2:0] tone disable (A,B,C), bits[5:3] noise disable (A,B,C)
//!   0x08-0x0A: Channel A/B/C volume (bits[3:0]=amplitude, bit4=envelope mode)
//!   0x0B-0x0C: Envelope period (lo/hi)
//!   0x0D:      Envelope shape (bits[3:0])

use super::SoundChipEmulator;
use std::f32::consts::PI;

const AY_VOL_TABLE: [f32; 16] = [
    0.000, 0.012, 0.016, 0.024, 0.033, 0.046, 0.064, 0.090, 0.127, 0.179, 0.253, 0.358, 0.506,
    0.715, 1.010, 1.000,
];

#[derive(Debug, Clone, Copy, Default)]
struct ToneChannel {
    period: u16,
    volume: u8,
    env_mode: bool,
    phase_acc: f32,
}

/// AY 8910.
pub struct AY8910 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 16],
    channels: [ToneChannel; 3],
    noise_period: u8,
    mixer: u8,
    env_period: u16,
    env_counter: f32,
    env_step: u8,
    env_hold: bool,
    env_alternate: bool,
    env_attack: bool,
    env_continue: bool,
    noise_lfsr: u32,
    noise_phase: f32,
}

impl AY8910 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(1_789_750)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 16],
            channels: [ToneChannel::default(); 3],
            noise_period: 1,
            mixer: 0xFF,
            env_period: 1,
            env_counter: 0.0,
            env_step: 0,
            env_hold: false,
            env_alternate: false,
            env_attack: false,
            env_continue: false,
            noise_lfsr: 1,
            noise_phase: 0.0,
        }
    }

    fn apply_reg(&mut self, addr: u8, data: u8) {
        let addr = (addr & 0x0F) as usize;
        self.regs[addr] = data;
        match addr {
            0x00 => self.channels[0].period = (self.channels[0].period & 0xF00) | data as u16,
            0x01 => {
                self.channels[0].period =
                    (self.channels[0].period & 0x0FF) | ((data as u16 & 0x0F) << 8)
            }
            0x02 => self.channels[1].period = (self.channels[1].period & 0xF00) | data as u16,
            0x03 => {
                self.channels[1].period =
                    (self.channels[1].period & 0x0FF) | ((data as u16 & 0x0F) << 8)
            }
            0x04 => self.channels[2].period = (self.channels[2].period & 0xF00) | data as u16,
            0x05 => {
                self.channels[2].period =
                    (self.channels[2].period & 0x0FF) | ((data as u16 & 0x0F) << 8)
            }
            0x06 => self.noise_period = data & 0x1F,
            0x07 => self.mixer = data,
            0x08 => {
                self.channels[0].volume = data & 0x0F;
                self.channels[0].env_mode = (data & 0x10) != 0;
            }
            0x09 => {
                self.channels[1].volume = data & 0x0F;
                self.channels[1].env_mode = (data & 0x10) != 0;
            }
            0x0A => {
                self.channels[2].volume = data & 0x0F;
                self.channels[2].env_mode = (data & 0x10) != 0;
            }
            0x0B => self.env_period = (self.env_period & 0xFF00) | data as u16,
            0x0C => self.env_period = (self.env_period & 0x00FF) | ((data as u16) << 8),
            0x0D => {
                self.env_attack = (data & 0x04) != 0;
                self.env_alternate = (data & 0x02) != 0;
                self.env_hold = (data & 0x01) != 0;
                self.env_continue = (data & 0x08) != 0;
                self.env_step = if self.env_attack { 0 } else { 15 };
                self.env_counter = 0.0;
            }
            _ => {}
        }
    }

    fn tone_freq_hz(&self, ch: usize) -> f32 {
        let period = self.channels[ch].period.max(1) as f32;
        self.clock_rate as f32 / (16.0 * period)
    }

    fn noise_freq_hz(&self) -> f32 {
        let period = self.noise_period.max(1) as f32;
        self.clock_rate as f32 / (16.0 * period)
    }

    fn envelope_amplitude(&self) -> f32 {
        AY_VOL_TABLE[self.env_step as usize & 0xF]
    }

    fn advance_envelope(&mut self, sample_rate: u32) {
        if self.env_period == 0 {
            return;
        }
        let ep = self.env_period.max(1) as f32;
        let env_hz = self.clock_rate as f32 / (256.0 * ep);
        let inc = env_hz / sample_rate as f32;
        self.env_counter += inc;
        if self.env_counter >= 1.0 {
            self.env_counter -= 1.0;
            if !self.env_hold {
                if self.env_attack {
                    self.env_step = self.env_step.wrapping_add(1);
                } else {
                    self.env_step = self.env_step.wrapping_sub(1);
                }
                if self.env_step >= 16 {
                    if self.env_continue {
                        if self.env_alternate {
                            self.env_attack = !self.env_attack;
                        }
                        self.env_step = if self.env_attack { 0 } else { 15 };
                    } else {
                        self.env_step = 0;
                        self.env_hold = true;
                    }
                }
            }
        }
    }

    fn advance_noise(&mut self, sample_rate: u32) {
        let noise_hz = self.noise_freq_hz();
        let inc = noise_hz / sample_rate as f32;
        self.noise_phase += inc;
        if self.noise_phase >= 1.0 {
            self.noise_phase -= 1.0;
            // 17-bit LFSR (taps at bits 0 and 3)
            let bit = (self.noise_lfsr ^ (self.noise_lfsr >> 3)) & 1;
            self.noise_lfsr = (self.noise_lfsr >> 1) | (bit << 16);
        }
    }

    fn noise_out(&self) -> f32 {
        if (self.noise_lfsr & 1) != 0 {
            1.0
        } else {
            -1.0
        }
    }
}

impl SoundChipEmulator for AY8910 {
    fn name(&self) -> &'static str {
        "AY-3-8910"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.apply_reg(addr, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        // VGM 0xA0: port byte top bit selects chip instance (we only have one)
        let _ = port;
        self.apply_reg(addr, data);
    }

    fn read(&self, addr: u8) -> u8 {
        self.regs[(addr & 0x0F) as usize]
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;
        for frame in buffer.chunks_mut(2) {
            self.advance_envelope(sample_rate);
            self.advance_noise(sample_rate);

            let mut sample = 0.0f32;
            for ch in 0..3 {
                let tone_disable = (self.mixer >> ch) & 1 != 0;
                let noise_disable = (self.mixer >> (ch + 3)) & 1 != 0;

                let freq = self.tone_freq_hz(ch);
                let inc = freq * 2.0 * PI / sample_rate as f32;
                self.channels[ch].phase_acc += inc;
                if self.channels[ch].phase_acc > 2.0 * PI {
                    self.channels[ch].phase_acc -= 2.0 * PI;
                }
                let tone_out = if self.channels[ch].phase_acc < PI {
                    1.0f32
                } else {
                    -1.0
                };

                let combined = match (tone_disable, noise_disable) {
                    (true, true) => 1.0,
                    (false, true) => tone_out,
                    (true, false) => self.noise_out(),
                    (false, false) => {
                        if tone_out > 0.0 && self.noise_out() > 0.0 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                };

                let amplitude = if self.channels[ch].env_mode {
                    self.envelope_amplitude()
                } else {
                    AY_VOL_TABLE[self.channels[ch].volume as usize]
                };

                sample += combined * amplitude * 0.1;
            }

            let out = (sample / 3.0).clamp(-1.0, 1.0);
            frame[0] = out;
            frame[1] = out;
        }
    }
}

impl Default for AY8910 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ay8910_new() {
        let chip = AY8910::new();
        assert_eq!(chip.name(), "AY-3-8910");
        assert_eq!(chip.clock_rate(), 1_789_750);
    }

    #[test]
    fn test_ay8910_write_period() {
        let mut chip = AY8910::new();
        chip.write(0x00, 0xFE); // ch A period lo
        chip.write(0x01, 0x01); // ch A period hi
        assert_eq!(chip.channels[0].period, 0x1FE);
    }

    #[test]
    fn test_ay8910_mixer_and_volume() {
        let mut chip = AY8910::new();
        chip.write(0x07, 0x38); // tones enabled, noise disabled
        chip.write(0x08, 0x0F); // ch A max volume
        assert_eq!(chip.mixer, 0x38);
        assert_eq!(chip.channels[0].volume, 0x0F);
    }

    #[test]
    fn test_ay8910_generate_samples_active() {
        let mut chip = AY8910::new();
        chip.write(0x00, 0xFE);
        chip.write(0x01, 0x01);
        chip.write(0x07, 0x3E); // ch A tone enabled
        chip.write(0x08, 0x0F);
        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        assert!(
            buffer.iter().any(|&s| s != 0.0),
            "active channel must produce output"
        );
    }

    #[test]
    fn test_ay8910_soundchip_trait() {
        let mut chip = AY8910::new();
        chip.reset();
        chip.write(0x00, 0x55);
        chip.write(0x07, 0x3E);
        chip.write(0x08, 0x0F);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
