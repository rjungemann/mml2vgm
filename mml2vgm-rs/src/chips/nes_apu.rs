//! NES APU (2A03/2A07) emulation
//!
//! 5 channels: 2 pulse waves, 1 triangle, 1 noise, 1 DPCM.
//! VGM opcode 0xB4: two-byte payload [addr, data] (addr relative to 0x4000).
//!
//! Register map (offsets from 0x4000):
//!   0x00-0x03: Pulse 1 (duty/length/envelope, sweep, period lo, period hi+length)
//!   0x04-0x07: Pulse 2 (same layout as pulse 1)
//!   0x08-0x0B: Triangle (linear counter, period lo, period hi+length)
//!   0x0C-0x0F: Noise (length/envelope, -, period/mode, length)
//!   0x10-0x13: DMC (flags/rate, direct load, address, length) — outputs silence here
//!   0x15:      Status (enable bits for each channel)

use super::SoundChipEmulator;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 75% (negated 25%)
];

const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

#[derive(Debug, Clone, Copy, Default)]
struct PulseChannel {
    enabled: bool,
    duty: u8,
    volume: u8,
    env_loop: bool,
    env_disable: bool,
    period: u16,
    phase_acc: f32,
    duty_step: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct TriangleChannel {
    enabled: bool,
    period: u16,
    phase_acc: f32,
    step: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct NoiseChannel {
    enabled: bool,
    volume: u8,
    env_loop: bool,
    env_disable: bool,
    period: u16,
    mode: bool,
    lfsr: u16,
    phase_acc: f32,
}

pub struct NesApu {
    clock_rate: u32,
    pulse: [PulseChannel; 2],
    triangle: TriangleChannel,
    noise: NoiseChannel,
    regs: [u8; 0x18],
}

impl NesApu {
    pub fn new() -> Self {
        Self::with_clock_rate(1_789_772)
    }

    pub fn with_clock_rate(clock_rate: u32) -> Self {
        let mut apu = Self {
            clock_rate,
            pulse: [PulseChannel::default(); 2],
            triangle: TriangleChannel::default(),
            noise: NoiseChannel { lfsr: 1, ..Default::default() },
            regs: [0u8; 0x18],
        };
        // Default noise LFSR seed
        apu.noise.lfsr = 1;
        apu
    }

    fn apply_reg(&mut self, addr: u8, data: u8) {
        let addr = addr as usize;
        if addr < 0x18 { self.regs[addr] = data; }
        match addr {
            // Pulse 1
            0x00 => {
                self.pulse[0].duty = (data >> 6) & 0x03;
                self.pulse[0].env_loop = (data & 0x20) != 0;
                self.pulse[0].env_disable = (data & 0x10) != 0;
                self.pulse[0].volume = data & 0x0F;
            }
            0x02 => {
                self.pulse[0].period =
                    (self.pulse[0].period & 0x0700) | data as u16;
            }
            0x03 => {
                self.pulse[0].period =
                    (self.pulse[0].period & 0x00FF) | (((data & 0x07) as u16) << 8);
                self.pulse[0].phase_acc = 0.0;
            }
            // Pulse 2
            0x04 => {
                self.pulse[1].duty = (data >> 6) & 0x03;
                self.pulse[1].env_loop = (data & 0x20) != 0;
                self.pulse[1].env_disable = (data & 0x10) != 0;
                self.pulse[1].volume = data & 0x0F;
            }
            0x06 => {
                self.pulse[1].period =
                    (self.pulse[1].period & 0x0700) | data as u16;
            }
            0x07 => {
                self.pulse[1].period =
                    (self.pulse[1].period & 0x00FF) | (((data & 0x07) as u16) << 8);
                self.pulse[1].phase_acc = 0.0;
            }
            // Triangle
            0x0A => {
                self.triangle.period =
                    (self.triangle.period & 0x0700) | data as u16;
            }
            0x0B => {
                self.triangle.period =
                    (self.triangle.period & 0x00FF) | (((data & 0x07) as u16) << 8);
            }
            // Noise
            0x0C => {
                self.noise.env_loop = (data & 0x20) != 0;
                self.noise.env_disable = (data & 0x10) != 0;
                self.noise.volume = data & 0x0F;
            }
            0x0E => {
                self.noise.mode = (data & 0x80) != 0;
                self.noise.period = NOISE_PERIOD_TABLE[(data & 0x0F) as usize];
            }
            // Status register: enable/disable channels
            0x15 => {
                self.pulse[0].enabled = (data & 0x01) != 0;
                self.pulse[1].enabled = (data & 0x02) != 0;
                self.triangle.enabled = (data & 0x04) != 0;
                self.noise.enabled = (data & 0x08) != 0;
            }
            _ => {}
        }
    }

    fn pulse_freq_hz(&self, ch: usize) -> f32 {
        let period = self.pulse[ch].period.max(8) as f32;
        // NES pulse freq = CPU_clock / (16 × (period + 1))
        self.clock_rate as f32 / (16.0 * (period + 1.0))
    }

    fn triangle_freq_hz(&self) -> f32 {
        let period = self.triangle.period.max(1) as f32;
        // Triangle freq = CPU_clock / (32 × (period + 1))
        self.clock_rate as f32 / (32.0 * (period + 1.0))
    }

    fn noise_freq_hz(&self) -> f32 {
        let period = self.noise.period.max(1) as f32;
        self.clock_rate as f32 / (16.0 * period)
    }

    fn pulse_volume(ch: &PulseChannel) -> f32 {
        if ch.env_disable {
            ch.volume as f32 / 15.0
        } else {
            // Constant volume from envelope (simplified: use volume field)
            ch.volume as f32 / 15.0
        }
    }
}

// 32-step triangle sequence used by the NES APU
const TRIANGLE_SEQ: [f32; 32] = [
    15.0, 14.0, 13.0, 12.0, 11.0, 10.0,  9.0,  8.0,
     7.0,  6.0,  5.0,  4.0,  3.0,  2.0,  1.0,  0.0,
     0.0,  1.0,  2.0,  3.0,  4.0,  5.0,  6.0,  7.0,
     8.0,  9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
];

impl SoundChipEmulator for NesApu {
    fn name(&self) -> &'static str { "NES APU" }
    fn clock_rate(&self) -> u32 { self.clock_rate }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.apply_reg(addr, data);
    }

    fn read(&self, addr: u8) -> u8 {
        let a = addr as usize;
        if a < 0x18 { self.regs[a] } else { 0xFF }
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for frame in buffer.chunks_mut(2) {
            let mut out = 0.0f32;

            // Pulse channels
            for ch in 0..2 {
                if !self.pulse[ch].enabled { continue; }
                let freq = self.pulse_freq_hz(ch);
                let phase_inc = freq / sample_rate as f32;
                self.pulse[ch].phase_acc += phase_inc;
                if self.pulse[ch].phase_acc >= 1.0 {
                    self.pulse[ch].phase_acc -= 1.0;
                    self.pulse[ch].duty_step = (self.pulse[ch].duty_step + 1) % 8;
                }
                let duty = self.pulse[ch].duty as usize;
                let high = DUTY_TABLE[duty][self.pulse[ch].duty_step] != 0;
                let sample = if high { 1.0f32 } else { -1.0 };
                out += sample * Self::pulse_volume(&self.pulse[ch]) * 0.12;
            }

            // Triangle
            if self.triangle.enabled {
                let freq = self.triangle_freq_hz();
                let phase_inc = freq / sample_rate as f32;
                self.triangle.phase_acc += phase_inc;
                if self.triangle.phase_acc >= 1.0 {
                    self.triangle.phase_acc -= 1.0;
                    self.triangle.step = (self.triangle.step + 1) % 32;
                }
                let s = TRIANGLE_SEQ[self.triangle.step] / 15.0 * 2.0 - 1.0;
                out += s * 0.12;
            }

            // Noise
            if self.noise.enabled {
                let freq = self.noise_freq_hz();
                let phase_inc = freq / sample_rate as f32;
                self.noise.phase_acc += phase_inc;
                if self.noise.phase_acc >= 1.0 {
                    self.noise.phase_acc -= 1.0;
                    let tap = if self.noise.mode { 6 } else { 1 };
                    let feedback = (self.noise.lfsr ^ (self.noise.lfsr >> tap)) & 1;
                    self.noise.lfsr = (self.noise.lfsr >> 1) | (feedback << 14);
                }
                let noise_out = if (self.noise.lfsr & 1) == 0 { 1.0f32 } else { -1.0 };
                let vol = if self.noise.env_disable { self.noise.volume as f32 / 15.0 } else { self.noise.volume as f32 / 15.0 };
                out += noise_out * vol * 0.12;
            }

            let mixed = out.clamp(-1.0, 1.0);
            if frame.len() >= 2 {
                frame[0] = mixed;
                frame[1] = mixed;
            }
        }
    }
}

impl Default for NesApu {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nes_apu_new() {
        let chip = NesApu::new();
        assert_eq!(chip.name(), "NES APU");
        assert_eq!(chip.clock_rate(), 1_789_772);
    }

    #[test]
    fn test_nes_apu_pulse_enable() {
        let mut chip = NesApu::new();
        chip.write(0x15, 0x01); // enable pulse 1
        assert!(chip.pulse[0].enabled);
    }

    #[test]
    fn test_nes_apu_pulse_period() {
        let mut chip = NesApu::new();
        chip.write(0x02, 0xFE); // pulse 1 period lo
        chip.write(0x03, 0x03); // pulse 1 period hi
        assert_eq!(chip.pulse[0].period, 0x3FE);
    }

    #[test]
    fn test_nes_apu_generate_pulse_output() {
        let mut chip = NesApu::new();
        chip.write(0x00, 0x3F); // pulse 1: 50% duty, volume=15
        chip.write(0x02, 0x80); // period lo
        chip.write(0x03, 0x00); // period hi
        chip.write(0x15, 0x01); // enable pulse 1
        let mut buf = [0.0f32; 16];
        chip.generate_samples(&mut buf, 44100);
        assert!(buf.iter().any(|&s| s != 0.0), "active pulse channel must produce output");
    }

    #[test]
    fn test_nes_apu_noise_output() {
        let mut chip = NesApu::new();
        chip.write(0x0C, 0x1F); // noise volume=15
        chip.write(0x0E, 0x01); // noise period index 1
        chip.write(0x15, 0x08); // enable noise
        let mut buf = [0.0f32; 16];
        chip.generate_samples(&mut buf, 44100);
        assert!(buf.iter().any(|&s| s != 0.0), "active noise channel must produce output");
    }

    #[test]
    fn test_nes_apu_reset() {
        let mut chip = NesApu::new();
        chip.write(0x15, 0x0F);
        chip.reset();
        assert!(!chip.pulse[0].enabled);
        assert!(!chip.triangle.enabled);
    }
}
