//! YM2608 (OPNA) sound chip emulation
//!
//! The YM2608 is a 6-channel FM synthesis + 3-channel SSG chip used in various systems
//! including the PC-8801, PC-9801, and X68000.
//! It is part of the OPNA family of Yamaha sound chips.
//!
//! # Features
//! - 6 FM channels (each with 4 operators)
//! - 3 SSG channels (square wave generators)
//! - 6 Rhythm channels (BD, SD, TOP, HH, TOM, RIM)
//! - ADPCM playback
//! - Stereo output

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// SSG (square wave) channel
#[derive(Debug, Clone, Copy, Default)]
struct SsgChannel {
    frequency: u16, // 12-bit period register (0 = silent)
    volume: u8,     // amplitude 0-15
    phase: u32,     // clock-cycle phase accumulator
}

/// FM channel for YM2608
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    frequency: u16,
    octave: u8,
    algorithm: u8,
    feedback: u8,
    output_level: u8,
    key_on: bool,
    output_phase: u32,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            frequency: 0,
            octave: 0,
            algorithm: 0,
            feedback: 0,
            output_level: 0,
            key_on: false,
            output_phase: 0,
        }
    }
}

/// YM2608 chip emulator with 6 FM + 3 SSG channels
pub struct YM2608 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache (extended to 0x400 for OPNA)
    regs: [u8; 0x400],

    /// 6 FM channels
    fm_channels: [FmChannel; 6],

    /// 3 SSG channels
    ssg_channels: [SsgChannel; 3],

    /// Accumulated clock cycles
    accumulated_cycles: f32,
}

impl YM2608 {
    /// Create a new YM2608 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(7_987_200)
    }

    /// Create a new YM2608 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x400],
            fm_channels: [FmChannel::default(); 6],
            ssg_channels: [SsgChannel::default(); 3],
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from FM channels using the correct OPN F-number formula.
    /// phase_rad = output_phase × 2π × F-number / (144 × 2^(21−block))
    /// The clock_rate cancels out because output_phase increments per master clock.
    fn get_fm_output(&self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for ch in &self.fm_channels {
            if ch.key_on && ch.frequency > 0 {
                let shift = 21u32.saturating_sub(ch.octave.min(7) as u32);
                let denom = 144.0 * (1u64 << shift) as f32;
                let phase_radians = (ch.output_phase as f32) * 2.0 * PI * (ch.frequency as f32) / denom;
                let amplitude = (1.0 - ch.output_level as f32 / 127.0) * 0.15;
                left += phase_radians.sin() * amplitude;
                right += phase_radians.sin() * amplitude;
            }
        }

        (left, right)
    }

    /// Get output from SSG channels as square waves.
    fn get_ssg_output(&self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for ch in &self.ssg_channels {
            if ch.frequency == 0 || ch.volume == 0 {
                continue;
            }
            // SSG full period = period_register × 32 master-clock cycles
            let full_period = (ch.frequency as u32).saturating_mul(32);
            if full_period == 0 {
                continue;
            }
            let phase_in_period = ch.phase % full_period;
            let is_high = phase_in_period < full_period / 2;
            let amplitude = ((ch.volume & 0x0F) as f32 / 15.0) * 0.12;
            let sample = if is_high { amplitude } else { -amplitude };
            left += sample;
            right += sample;
        }

        (left, right)
    }

    /// Route a decoded register write to the appropriate chip state.
    fn apply_register(&mut self, port: u8, addr: u8, data: u8) {
        if port == 0 {
            match addr {
                // FM key on/off
                // bits[7:4] = slot select, bits[1:0] = channel (0-2), bit[2] = port (0=ch1-3, 1=ch4-6)
                0x28 => {
                    let ch_sel = (data & 0x03) as usize;
                    let port_bit = ((data >> 2) & 0x01) as usize;
                    let ch = ch_sel + port_bit * 3;
                    if ch < 6 {
                        let was_on = self.fm_channels[ch].key_on;
                        self.fm_channels[ch].key_on = (data & 0xF0) != 0;
                        if !was_on && self.fm_channels[ch].key_on {
                            // Reset phase on key-on for clean attack
                            self.fm_channels[ch].output_phase = 0;
                        }
                    }
                }
                // F-number low byte for ch 1-3
                0xA0 | 0xA1 | 0xA2 => {
                    let ch = (addr - 0xA0) as usize;
                    let hi = self.regs[0xA4 + ch];
                    let block = (hi >> 3) & 0x07;
                    let fnumber = ((hi & 0x07) as u16) << 8 | data as u16;
                    self.fm_channels[ch].frequency = fnumber;
                    self.fm_channels[ch].octave = block;
                }
                // F-number high + block for ch 1-3
                0xA4 | 0xA5 | 0xA6 => {
                    let ch = (addr - 0xA4) as usize;
                    let block = (data >> 3) & 0x07;
                    let lo = self.regs[0xA0 + ch];
                    let fnumber = ((data & 0x07) as u16) << 8 | lo as u16;
                    self.fm_channels[ch].frequency = fnumber;
                    self.fm_channels[ch].octave = block;
                }
                // SSG period low (ch A=0x00, B=0x02, C=0x04)
                0x00 | 0x02 | 0x04 => {
                    let ch = (addr / 2) as usize;
                    let hi = self.regs[addr as usize + 1] & 0x0F;
                    if ch < 3 {
                        self.ssg_channels[ch].frequency = data as u16 | ((hi as u16) << 8);
                    }
                }
                // SSG period high (ch A=0x01, B=0x03, C=0x05)
                0x01 | 0x03 | 0x05 => {
                    let ch = (addr / 2) as usize;
                    let lo = self.regs[addr as usize - 1];
                    if ch < 3 {
                        self.ssg_channels[ch].frequency = lo as u16 | (((data & 0x0F) as u16) << 8);
                    }
                }
                // SSG volume/amplitude (ch A=0x08, B=0x09, C=0x0A)
                0x08 | 0x09 | 0x0A => {
                    let ch = (addr - 0x08) as usize;
                    if ch < 3 {
                        self.ssg_channels[ch].volume = data & 0x1F;
                    }
                }
                _ => {}
            }
        } else {
            // Port 1: FM channels 4-6 (indices 3-5)
            match addr {
                0x28 => { self.apply_register(0, addr, data); }
                0xA0 | 0xA1 | 0xA2 => {
                    let ch = (addr - 0xA0) as usize + 3;
                    let hi_idx = 0x100 + (0xA4 + (addr - 0xA0)) as usize;
                    let hi = if hi_idx < self.regs.len() { self.regs[hi_idx] } else { 0 };
                    let block = (hi >> 3) & 0x07;
                    let fnumber = ((hi & 0x07) as u16) << 8 | data as u16;
                    if ch < 6 {
                        self.fm_channels[ch].frequency = fnumber;
                        self.fm_channels[ch].octave = block;
                    }
                }
                0xA4 | 0xA5 | 0xA6 => {
                    let ch = (addr - 0xA4) as usize + 3;
                    let block = (data >> 3) & 0x07;
                    let lo_idx = 0x100 + (0xA0 + (addr - 0xA4)) as usize;
                    let lo = if lo_idx < self.regs.len() { self.regs[lo_idx] } else { 0 };
                    let fnumber = ((data & 0x07) as u16) << 8 | lo as u16;
                    if ch < 6 {
                        self.fm_channels[ch].frequency = fnumber;
                        self.fm_channels[ch].octave = block;
                    }
                }
                _ => {}
            }
        }
    }
}

impl SoundChipEmulator for YM2608 {
    fn name(&self) -> &'static str {
        "YM2608 (OPNA)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        if (addr as usize) < 0x200 {
            self.regs[addr as usize] = data;
        }
        self.apply_register(0, addr, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let base = if port == 0 { 0usize } else { 0x100usize };
        let idx = base + addr as usize;
        if idx < self.regs.len() {
            self.regs[idx] = data;
        }
        self.apply_register(port, addr, data);
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {
        // Update FM output phases
        for ch in &mut self.fm_channels {
            if ch.key_on {
                ch.output_phase = ch.output_phase.wrapping_add(1);
            }
        }

        // Update SSG phases
        for ch in &mut self.ssg_channels {
            ch.phase = ch.phase.wrapping_add(1);
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;

        for frame in buffer.chunks_mut(2) {
            let cycles_per_sample = self.clock_rate as f32 / sample_rate as f32;
            self.accumulated_cycles += cycles_per_sample;

            while self.accumulated_cycles >= 1.0 {
                self.clock();
                self.accumulated_cycles -= 1.0;
            }

            let (fm_left, fm_right) = self.get_fm_output();
            let (ssg_left, ssg_right) = self.get_ssg_output();

            frame[0] = (fm_left + ssg_left).clamp(-1.0, 1.0);
            frame[1] = (fm_right + ssg_right).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YM2608 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2608_new() {
        let chip = YM2608::new();
        assert_eq!(chip.name(), "YM2608 (OPNA)");
        assert_eq!(chip.clock_rate(), 7_987_200);
        assert_eq!(chip.fm_channels.len(), 6);
        assert_eq!(chip.ssg_channels.len(), 3);
    }

    #[test]
    fn test_ym2608_write_fm() {
        let mut chip = YM2608::new();
        chip.write(0x28, 0x42); // FM channel 0 frequency
        assert_eq!(chip.regs[0x28], 0x42);
    }

    #[test]
    fn test_ym2608_write_ssg() {
        let mut chip = YM2608::new();
        chip.write(0x0E, 0x10); // SSG channel 0 frequency
        chip.write(0x18, 0x0F); // SSG channel 0 volume
        assert_eq!(chip.regs[0x0E], 0x10);
        assert_eq!(chip.regs[0x18], 0x0F);
    }

    #[test]
    fn test_ym2608_soundchip_trait() {
        let mut chip = YM2608::new();

        assert_eq!(chip.name(), "YM2608 (OPNA)");
        assert_eq!(chip.clock_rate(), 7_987_200);

        chip.reset();
        // A4 (440 Hz): block=4, F-number=1038 (0x40E → high=0x04, low=0x0E)
        // A4 register: (block<<3)|(fnumber>>8) = (4<<3)|4 = 0x24
        chip.write(0xA0, 0x0E); // F-number low for ch1
        chip.write(0xA4, 0x24); // Block=4, F-number high=4
        chip.write(0x28, 0xF0); // Key on all slots for ch1: (0xF<<4)|0x00
        chip.clock();

        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);

        // Should generate some output with key on and valid frequency
        assert!(buffer[0].abs() > 0.0 || buffer[1].abs() > 0.0);
    }
}
