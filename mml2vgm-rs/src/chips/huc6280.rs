//! HuC6280 PSG emulation (PC Engine / TurboGrafx-16)
//!
//! 6 channels: wavetable (32 4-bit samples per channel), noise (ch 5/6), LFO (ch 1/2).
//! VGM opcode 0xB9: two-byte payload [addr, data].
//!
//! Register map (accessed via channel select at 0x00):
//!   0x00: Channel select (bits[2:0])
//!   0x01: Main L/R volume (bits[7:4]=L, bits[3:0]=R)
//!   0x02: Frequency LSB
//!   0x03: Frequency MSB (4-bit)
//!   0x04: Channel control (bit7=enable, bit6=DDA mode, bits[3:0]=volume)
//!   0x05: Balance L/R (bits[7:4]=L, bits[3:0]=R)
//!   0x06: Waveform data write (advances write pointer)
//!   0x07: Noise control (bit7=enable, bits[4:0]=noise freq)
//!   0x08: LFO frequency
//!   0x09: LFO control (bits[1:0]=depth, bit7=reset)

use super::SoundChipEmulator;

const WAVE_LEN: usize = 32;
const NUM_CHANNELS: usize = 6;
const PSG_CLOCK_DIV: f32 = 32.0;

#[derive(Debug, Clone)]
struct PsgChannel {
    enabled: bool,
    dda_mode: bool,
    volume: u8,
    balance_l: u8,
    balance_r: u8,
    frequency: u16,
    wavetable: [u8; WAVE_LEN],
    wave_ptr: usize,
    phase_acc: f32,
    dda_sample: u8,
    noise_enable: bool,
    noise_freq: u8,
}

impl Default for PsgChannel {
    fn default() -> Self {
        Self {
            enabled: false,
            dda_mode: false,
            volume: 0,
            balance_l: 15,
            balance_r: 15,
            frequency: 0,
            wavetable: [0; WAVE_LEN],
            wave_ptr: 0,
            phase_acc: 0.0,
            dda_sample: 0,
            noise_enable: false,
            noise_freq: 0,
        }
    }
}

/// Hu C6280.
pub struct HuC6280 {
    clock_rate: u32,
    sample_rate: u32,
    channels: [PsgChannel; NUM_CHANNELS],
    selected_ch: usize,
    main_vol_l: u8,
    main_vol_r: u8,
    lfo_freq: u8,
    lfo_ctrl: u8,
    noise_lfsr: u32,
    noise_phase: f32,
}

impl HuC6280 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            channels: std::array::from_fn(|_| PsgChannel::default()),
            selected_ch: 0,
            main_vol_l: 15,
            main_vol_r: 15,
            lfo_freq: 0,
            lfo_ctrl: 0,
            noise_lfsr: 1,
            noise_phase: 0.0,
        }
    }

    fn ch_freq_hz(&self, ch: usize) -> f32 {
        let period = self.channels[ch].frequency.max(1) as f32;
        self.clock_rate as f32 / (PSG_CLOCK_DIV * period)
    }

    #[allow(dead_code)]
    fn noise_freq_hz(&self, ch: usize) -> f32 {
        let nf = self.channels[ch].noise_freq as u32;
        let divisor = 1u32 << (nf & 0x1F);
        self.clock_rate as f32 / (divisor as f32 * 64.0)
    }

    fn advance_noise(&mut self, sample_rate: u32) {
        let freq = if let Some(ch) = self.channels[4..6].iter().find(|c| c.noise_enable) {
            let nf = ch.noise_freq as u32;
            let divisor = 1u32 << (nf & 0x1F);
            self.clock_rate as f32 / (divisor as f32 * 64.0)
        } else {
            return;
        };
        self.noise_phase += freq / sample_rate as f32;
        if self.noise_phase >= 1.0 {
            self.noise_phase -= 1.0;
            let bit = (self.noise_lfsr ^ (self.noise_lfsr >> 1)) & 1;
            self.noise_lfsr = (self.noise_lfsr >> 1) | (bit << 17);
        }
    }

    fn get_channel_sample(&mut self, ch: usize, sample_rate: u32) -> (f32, f32) {
        let enabled = self.channels[ch].enabled;
        if !enabled {
            return (0.0, 0.0);
        }

        let vol = self.channels[ch].volume as f32 / 15.0;
        let bal_l = self.channels[ch].balance_l as f32 / 15.0;
        let bal_r = self.channels[ch].balance_r as f32 / 15.0;
        let main_l = self.main_vol_l as f32 / 15.0;
        let main_r = self.main_vol_r as f32 / 15.0;

        let raw = if self.channels[ch].dda_mode {
            // DDA mode: output single PCM sample continuously
            self.channels[ch].dda_sample as f32 / 15.0 * 2.0 - 1.0
        } else if ch >= 4 && self.channels[ch].noise_enable {
            // Noise output
            if (self.noise_lfsr & 1) != 0 {
                1.0
            } else {
                -1.0
            }
        } else {
            // Wavetable output: advance phase by frequency
            let freq = self.ch_freq_hz(ch);
            if freq > 0.0 && self.channels[ch].frequency > 0 {
                self.channels[ch].phase_acc += freq / sample_rate as f32;
                if self.channels[ch].phase_acc >= 1.0 {
                    self.channels[ch].phase_acc -= 1.0;
                    self.channels[ch].wave_ptr = (self.channels[ch].wave_ptr + 1) % WAVE_LEN;
                }
            }
            let idx = self.channels[ch].wave_ptr;
            let sample_4bit = self.channels[ch].wavetable[idx] & 0x1F;
            sample_4bit as f32 / 15.5 * 2.0 - 1.0
        };

        let amplitude = vol * 0.15;
        (
            raw * amplitude * bal_l * main_l,
            raw * amplitude * bal_r * main_r,
        )
    }
}

impl SoundChipEmulator for HuC6280 {
    fn name(&self) -> &'static str {
        "HuC6280"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        match addr {
            0x00 => {
                self.selected_ch = (data & 0x07).min(5) as usize;
            }
            0x01 => {
                self.main_vol_l = (data >> 4) & 0x0F;
                self.main_vol_r = data & 0x0F;
            }
            0x02 => {
                self.channels[self.selected_ch].frequency =
                    (self.channels[self.selected_ch].frequency & 0xF00) | data as u16;
            }
            0x03 => {
                self.channels[self.selected_ch].frequency =
                    (self.channels[self.selected_ch].frequency & 0x0FF)
                        | ((data as u16 & 0x0F) << 8);
            }
            0x04 => {
                let ch = self.selected_ch;
                self.channels[ch].enabled = (data & 0x80) != 0;
                self.channels[ch].dda_mode = (data & 0x40) != 0;
                self.channels[ch].volume = data & 0x1F;
                if self.channels[ch].dda_mode {
                    // In DDA write mode (bit6=1, bit7=1): next write to 0x06 is audio sample
                }
                if !self.channels[ch].enabled {
                    self.channels[ch].wave_ptr = 0;
                    self.channels[ch].phase_acc = 0.0;
                }
            }
            0x05 => {
                let ch = self.selected_ch;
                self.channels[ch].balance_l = (data >> 4) & 0x0F;
                self.channels[ch].balance_r = data & 0x0F;
            }
            0x06 => {
                let ch = self.selected_ch;
                if self.channels[ch].dda_mode {
                    self.channels[ch].dda_sample = data & 0x1F;
                } else {
                    self.channels[ch].wavetable[self.channels[ch].wave_ptr] = data & 0x1F;
                    self.channels[ch].wave_ptr = (self.channels[ch].wave_ptr + 1) % WAVE_LEN;
                }
            }
            0x07 if self.selected_ch >= 4 => {
                let ch = self.selected_ch;
                self.channels[ch].noise_enable = (data & 0x80) != 0;
                self.channels[ch].noise_freq = data & 0x1F;
            }
            0x08 => {
                self.lfo_freq = data;
            }
            0x09 => {
                self.lfo_ctrl = data;
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }
    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;
        for frame in buffer.chunks_mut(2) {
            self.advance_noise(sample_rate);
            let mut left = 0.0f32;
            let mut right = 0.0f32;
            for ch in 0..NUM_CHANNELS {
                let (l, r) = self.get_channel_sample(ch, sample_rate);
                left += l;
                right += r;
            }
            frame[0] = (left / NUM_CHANNELS as f32).clamp(-1.0, 1.0);
            frame[1] = (right / NUM_CHANNELS as f32).clamp(-1.0, 1.0);
        }
    }
}

impl Default for HuC6280 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huc6280_new() {
        let chip = HuC6280::new();
        assert_eq!(chip.name(), "HuC6280");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_huc6280_channel_select_and_freq() {
        let mut chip = HuC6280::new();
        chip.write(0x00, 0x02); // select channel 2
        chip.write(0x02, 0xA0); // freq lo
        chip.write(0x03, 0x03); // freq hi
        assert_eq!(chip.channels[2].frequency, 0x3A0);
    }

    #[test]
    fn test_huc6280_wavetable_write() {
        let mut chip = HuC6280::new();
        chip.write(0x00, 0x00); // select channel 0
        chip.write(0x04, 0x00); // disable (resets wave_ptr)
        for i in 0..32u8 {
            chip.write(0x06, i & 0x1F);
        }
        assert_eq!(chip.channels[0].wavetable[0], 0);
        assert_eq!(chip.channels[0].wavetable[1], 1);
        assert_eq!(chip.channels[0].wavetable[31], 31);
    }

    #[test]
    fn test_huc6280_generate_samples_active() {
        let mut chip = HuC6280::new();
        chip.write(0x00, 0x00);
        chip.write(0x04, 0x00); // disable to reset
        for i in 0..32u8 {
            chip.write(0x06, i);
        }
        chip.write(0x02, 0x00);
        chip.write(0x03, 0x08); // period = 0x800
        chip.write(0x04, 0x9F); // enable, volume=15
        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        assert!(
            buffer.iter().any(|&s| s != 0.0),
            "active channel must produce output"
        );
    }

    #[test]
    fn test_huc6280_soundchip_trait() {
        let mut chip = HuC6280::new();
        chip.reset();
        chip.write(0x00, 0x00);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
