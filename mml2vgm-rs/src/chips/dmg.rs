//! DMG APU (Game Boy) sound chip emulation
//!
//! 4 channels: 2 square wave pulse, 1 wave (custom waveform), 1 noise (LFSR).
//! VGM opcode 0xB3: two-byte payload [addr, data] (addr relative to 0xFF10).
//!
//! Register map (offsets from 0xFF10, i.e. NR10 = 0x00):
//!   0x00 (NR10): CH1 sweep (bit6:4=period, bit3=direction, bit2:0=shift)
//!   0x01 (NR11): CH1 duty/length (bit7:6=duty, bit5:0=length)
//!   0x02 (NR12): CH1 envelope (bit7:4=initial vol, bit3=dir, bit2:0=period)
//!   0x03 (NR13): CH1 frequency lo
//!   0x04 (NR14): CH1 frequency hi + trigger (bit7=trigger, bit6=length enable, bit2:0=freq hi)
//!   0x06 (NR21): CH2 duty/length
//!   0x07 (NR22): CH2 envelope
//!   0x08 (NR23): CH2 frequency lo
//!   0x09 (NR24): CH2 frequency hi + trigger
//!   0x0A (NR30): CH3 DAC enable (bit7)
//!   0x0B (NR31): CH3 length
//!   0x0C (NR32): CH3 output level (bit6:5 — 0=mute,1=full,2=half,3=quarter)
//!   0x0D (NR33): CH3 frequency lo
//!   0x0E (NR34): CH3 frequency hi + trigger
//!   0x10 (NR41): CH4 length (bit5:0)
//!   0x11 (NR42): CH4 envelope
//!   0x12 (NR43): CH4 frequency/LFSR (bit7:4=shift, bit3=width, bit2:0=divisor)
//!   0x13 (NR44): CH4 trigger/length enable
//!   0x14 (NR50): Master volume (bit6:4=left, bit2:0=right)
//!   0x15 (NR51): Panning (bit7:4=left ch4-1, bit3:0=right ch4-1)
//!   0x16 (NR52): Power (bit7=master enable)
//!   0x20-0x2F: CH3 wave RAM (32 nibbles in 16 bytes)

use super::SoundChipEmulator;

const WAVE_RAM_LEN: usize = 16;
const WAVE_SAMPLES: usize = 32;
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];
const CH4_DIVISORS: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

#[derive(Debug, Clone, Copy, Default)]
struct PulseChannel {
    enabled: bool,
    duty: u8,
    freq: u16,
    env_vol: u8,
    env_dir: bool,
    env_period: u8,
    phase_acc: f32,
    duty_step: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct WaveChannel {
    enabled: bool,
    dac_on: bool,
    freq: u16,
    output_level: u8,
    phase_acc: f32,
    wave_pos: usize,
}

#[derive(Debug, Clone, Copy)]
struct NoiseChannel {
    enabled: bool,
    env_vol: u8,
    env_dir: bool,
    env_period: u8,
    shift_clock: u8,
    width_mode: bool,
    divisor_code: u8,
    lfsr: u16,
    phase_acc: f32,
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self {
            enabled: false,
            env_vol: 0,
            env_dir: false,
            env_period: 0,
            shift_clock: 0,
            width_mode: false,
            divisor_code: 0,
            lfsr: 0x7FFF,
            phase_acc: 0.0,
        }
    }
}

/// Dmg.
pub struct Dmg {
    clock_rate: u32,
    pulse: [PulseChannel; 2],
    wave: WaveChannel,
    noise: NoiseChannel,
    wave_ram: [u8; WAVE_RAM_LEN],
    master_on: bool,
    nr50: u8,
    nr51: u8,
}

impl Dmg {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(4_194_304)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            pulse: [PulseChannel::default(); 2],
            wave: WaveChannel::default(),
            noise: NoiseChannel::default(),
            wave_ram: [0u8; WAVE_RAM_LEN],
            master_on: false,
            nr50: 0,
            nr51: 0,
        }
    }

    fn pulse_freq_hz(&self, ch: usize) -> f32 {
        let period = (2048u32).saturating_sub(self.pulse[ch].freq as u32).max(1) as f32;
        self.clock_rate as f32 / (8.0 * period)
    }

    fn wave_freq_hz(&self) -> f32 {
        let period = (2048u32).saturating_sub(self.wave.freq as u32).max(1) as f32;
        self.clock_rate as f32 / (16.0 * period)
    }

    fn noise_freq_hz(&self) -> f32 {
        let div = CH4_DIVISORS[self.noise.divisor_code as usize & 0x07] as f32;
        let shift = self.noise.shift_clock as u32;
        // f = clock / (div × 2^(shift+1))
        self.clock_rate as f32 / (div * (1u32 << (shift + 1)) as f32)
    }

    fn wave_sample(&self, pos: usize) -> f32 {
        let byte_idx = pos / 2;
        let nibble = if pos.is_multiple_of(2) {
            (self.wave_ram[byte_idx] >> 4) & 0x0F
        } else {
            self.wave_ram[byte_idx] & 0x0F
        };
        // Scale 0-15 to -1.0..+1.0
        nibble as f32 / 7.5 - 1.0
    }
}

impl SoundChipEmulator for Dmg {
    fn name(&self) -> &'static str {
        "DMG (Game Boy)"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        match addr {
            // CH1 pulse
            0x01 => self.pulse[0].duty = (data >> 6) & 0x03,
            0x02 => {
                self.pulse[0].env_vol = (data >> 4) & 0x0F;
                self.pulse[0].env_dir = (data & 0x08) != 0;
                self.pulse[0].env_period = data & 0x07;
            }
            0x03 => self.pulse[0].freq = (self.pulse[0].freq & 0x700) | data as u16,
            0x04 => {
                self.pulse[0].freq = (self.pulse[0].freq & 0x0FF) | (((data & 0x07) as u16) << 8);
                if (data & 0x80) != 0 {
                    self.pulse[0].enabled = true;
                    self.pulse[0].phase_acc = 0.0;
                }
            }
            // CH2 pulse
            0x06 => self.pulse[1].duty = (data >> 6) & 0x03,
            0x07 => {
                self.pulse[1].env_vol = (data >> 4) & 0x0F;
                self.pulse[1].env_dir = (data & 0x08) != 0;
                self.pulse[1].env_period = data & 0x07;
            }
            0x08 => self.pulse[1].freq = (self.pulse[1].freq & 0x700) | data as u16,
            0x09 => {
                self.pulse[1].freq = (self.pulse[1].freq & 0x0FF) | (((data & 0x07) as u16) << 8);
                if (data & 0x80) != 0 {
                    self.pulse[1].enabled = true;
                    self.pulse[1].phase_acc = 0.0;
                }
            }
            // CH3 wave
            0x0A => self.wave.dac_on = (data & 0x80) != 0,
            0x0C => self.wave.output_level = (data >> 5) & 0x03,
            0x0D => self.wave.freq = (self.wave.freq & 0x700) | data as u16,
            0x0E => {
                self.wave.freq = (self.wave.freq & 0x0FF) | (((data & 0x07) as u16) << 8);
                if (data & 0x80) != 0 {
                    self.wave.enabled = true;
                    self.wave.phase_acc = 0.0;
                    self.wave.wave_pos = 0;
                }
            }
            // CH4 noise
            0x11 => {
                self.noise.env_vol = (data >> 4) & 0x0F;
                self.noise.env_dir = (data & 0x08) != 0;
                self.noise.env_period = data & 0x07;
            }
            0x12 => {
                self.noise.shift_clock = (data >> 4) & 0x0F;
                self.noise.width_mode = (data & 0x08) != 0;
                self.noise.divisor_code = data & 0x07;
            }
            0x13 => {
                if (data & 0x80) != 0 {
                    self.noise.enabled = true;
                    self.noise.lfsr = 0x7FFF;
                    self.noise.phase_acc = 0.0;
                }
            }
            // NR50/NR51/NR52
            0x14 => self.nr50 = data,
            0x15 => self.nr51 = data,
            0x16 => self.master_on = (data & 0x80) != 0,
            // Wave RAM
            0x20..=0x2F => {
                self.wave_ram[(addr - 0x20) as usize] = data;
            }
            _ => {}
        }
    }

    fn read(&self, addr: u8) -> u8 {
        if (0x20..=0x2F).contains(&addr) {
            self.wave_ram[(addr - 0x20) as usize]
        } else {
            0xFF
        }
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for frame in buffer.chunks_mut(2) {
            if !self.master_on {
                if frame.len() >= 2 {
                    frame[0] = 0.0;
                    frame[1] = 0.0;
                }
                continue;
            }

            let mut out = 0.0f32;

            // CH1 + CH2: square wave pulses
            for ch in 0..2 {
                if !self.pulse[ch].enabled || self.pulse[ch].env_vol == 0 {
                    continue;
                }
                let freq = self.pulse_freq_hz(ch);
                let phase_inc = freq / sample_rate as f32;
                self.pulse[ch].phase_acc += phase_inc;
                if self.pulse[ch].phase_acc >= 1.0 {
                    self.pulse[ch].phase_acc -= 1.0;
                    self.pulse[ch].duty_step = (self.pulse[ch].duty_step + 1) % 8;
                }
                let duty = self.pulse[ch].duty as usize & 0x03;
                let hi = DUTY_TABLE[duty][self.pulse[ch].duty_step] != 0;
                let s = if hi { 1.0f32 } else { -1.0 };
                let vol = self.pulse[ch].env_vol as f32 / 15.0;
                out += s * vol * 0.12;
            }

            // CH3: wave channel
            if self.wave.enabled && self.wave.dac_on && self.wave.output_level != 0 {
                let freq = self.wave_freq_hz();
                let phase_inc = freq / sample_rate as f32;
                self.wave.phase_acc += phase_inc;
                if self.wave.phase_acc >= 1.0 {
                    self.wave.phase_acc -= 1.0;
                    self.wave.wave_pos = (self.wave.wave_pos + 1) % WAVE_SAMPLES;
                }
                let raw = self.wave_sample(self.wave.wave_pos);
                let shift = match self.wave.output_level {
                    1 => 0, // full
                    2 => 1, // half
                    3 => 2, // quarter
                    _ => 4, // mute
                };
                let s = if shift < 4 {
                    raw / (1 << shift) as f32
                } else {
                    0.0
                };
                out += s * 0.12;
            }

            // CH4: noise
            if self.noise.enabled && self.noise.env_vol != 0 {
                let freq = self.noise_freq_hz();
                let phase_inc = freq / sample_rate as f32;
                self.noise.phase_acc += phase_inc;
                if self.noise.phase_acc >= 1.0 {
                    self.noise.phase_acc -= 1.0;
                    let tap = if self.noise.width_mode { 6 } else { 14 };
                    let fb = (self.noise.lfsr ^ (self.noise.lfsr >> tap)) & 1;
                    self.noise.lfsr = (self.noise.lfsr >> 1) | (fb << 14);
                    if self.noise.width_mode {
                        self.noise.lfsr = (self.noise.lfsr & !0x40) | (fb << 6);
                    }
                }
                let s = if (self.noise.lfsr & 1) == 0 {
                    1.0f32
                } else {
                    -1.0
                };
                let vol = self.noise.env_vol as f32 / 15.0;
                out += s * vol * 0.12;
            }

            let mixed = out.clamp(-1.0, 1.0);
            if frame.len() >= 2 {
                frame[0] = mixed;
                frame[1] = mixed;
            }
        }
    }
}

impl Default for Dmg {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmg_new() {
        let chip = Dmg::new();
        assert_eq!(chip.name(), "DMG (Game Boy)");
        assert_eq!(chip.clock_rate(), 4_194_304);
    }

    #[test]
    fn test_dmg_pulse_trigger() {
        let mut chip = Dmg::new();
        chip.write(0x16, 0x80); // master on
        chip.write(0x02, 0xF0); // CH1 envelope: vol=15, no dir, period=0
        chip.write(0x03, 0x00); // CH1 freq lo
        chip.write(0x04, 0x87); // CH1 trigger + freq hi=7
        assert!(chip.pulse[0].enabled);
    }

    #[test]
    fn test_dmg_wave_ram_write() {
        let mut chip = Dmg::new();
        chip.write(0x20, 0xAB);
        chip.write(0x2F, 0x12);
        assert_eq!(chip.wave_ram[0], 0xAB);
        assert_eq!(chip.wave_ram[15], 0x12);
    }

    #[test]
    fn test_dmg_generate_pulse_output() {
        let mut chip = Dmg::new();
        chip.write(0x16, 0x80); // master on
        chip.write(0x01, 0x80); // CH1: 50% duty
        chip.write(0x02, 0xF0); // CH1 envelope vol=15
        chip.write(0x03, 0x80); // CH1 freq lo
        chip.write(0x04, 0x87); // CH1 trigger
        let mut buf = [0.0f32; 16];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().any(|&s| s != 0.0),
            "active DMG pulse must produce output"
        );
    }

    #[test]
    fn test_dmg_reset() {
        let mut chip = Dmg::new();
        chip.write(0x04, 0x80); // trigger CH1
        chip.reset();
        assert!(!chip.pulse[0].enabled);
    }
}
