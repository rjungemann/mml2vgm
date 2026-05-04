//! YM2151 (OPM) sound chip emulation
//!
//! The YM2151 is an 8-channel FM synthesis chip used in various arcade systems.
//! It is part of the OPM family of Yamaha sound chips.
//!
//! # Features
//! - 8 FM channels (each with 4 operators)
//! - 32 operators total
//! - LFO with AM/PM modulation
//! - Stereo output
//! - Simpler than YM2612 (no DAC, simpler control)

use super::SoundChipEmulator;
use std::f32::consts::PI;

/// A simple FM operator for YM2151
#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase: u16,
    phase_increment: u32,
    key_on: bool,
    attack_rate: u8,
    decay_rate: u8,
    sustain_level: u8,
    release_rate: u8,
    envelope_level: u8,
    total_level: u8,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self {
            phase: 0,
            phase_increment: 0,
            key_on: false,
            attack_rate: 0,
            decay_rate: 0,
            sustain_level: 0,
            release_rate: 0,
            envelope_level: 127,
            total_level: 127,
        }
    }
}

/// FM channel for YM2151
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    operators: [FmOperator; 4],
    frequency: u16,
    octave: u8,
    algorithm: u8,
    feedback: u8,
    left_enable: bool,
    right_enable: bool,
    output_level: u8,
    output_phase: u32,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            operators: [FmOperator::default(); 4],
            frequency: 0,
            octave: 0,
            algorithm: 0,
            feedback: 0,
            left_enable: true,
            right_enable: true,
            output_level: 0,
            output_phase: 0,
        }
    }
}

/// YM2151 chip emulator with 8 FM channels
pub struct YM2151 {
    /// Master clock rate in Hz
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Register cache (0x100 bytes for YM2151)
    regs: [u8; 0x100],

    /// 8 FM channels
    channels: [FmChannel; 8],

    /// LFO state
    lfo_counter: u16,
    lfo_enabled: bool,
    lfo_frequency: u8,

    /// Accumulated clock cycles for sample generation
    accumulated_cycles: f32,
}

impl YM2151 {
    /// Create a new YM2151 emulator with the default clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// Create a new YM2151 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            channels: [FmChannel::default(); 8],
            lfo_counter: 0,
            lfo_enabled: false,
            lfo_frequency: 0,
            accumulated_cycles: 0.0,
        }
    }

    /// Get output from a channel
    fn get_channel_output(&self, ch: usize) -> (f32, f32) {
        if ch >= 8 {
            return (0.0, 0.0);
        }

        let channel = &self.channels[ch];

        // Simple sine wave output based on channel frequency
        if !channel.operators[0].key_on || channel.output_level == 127 {
            return (0.0, 0.0);
        }

        // Calculate phase increment from frequency and octave
        let base_freq = 440.0 * 2_f32.powi((channel.frequency as i32 - 69) / 12);
        let freq = base_freq * 2_f32.powi(channel.octave as i32);
        let phase_f = (channel.output_phase as f32 / 1000000.0) * freq / self.sample_rate as f32 * 2.0 * PI;

        let output = phase_f.sin() * 0.5 * (1.0 - channel.output_level as f32 / 127.0);

        let left = if channel.left_enable { output } else { 0.0 };
        let right = if channel.right_enable { output } else { 0.0 };

        (left, right)
    }

    /// Get total output mixing all channels
    fn get_output(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for ch in 0..8 {
            let (ch_left, ch_right) = self.get_channel_output(ch);
            left += ch_left;
            right += ch_right;
        }

        // Normalize to prevent clipping
        let scale = 8.0; // 8 channels
        (left / scale, right / scale)
    }
}

impl SoundChipEmulator for YM2151 {
    fn name(&self) -> &'static str {
        "YM2151 (OPM)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        // Store in register cache
        self.regs[addr as usize] = data;

        // Handle register writes by address
        match addr {
            // LFO control
            0x01 => {
                self.lfo_enabled = (data & 0x80) != 0;
                self.lfo_frequency = data & 0x07;
            }
            // Tone number / frequency / octave registers
            0x28..=0x2F => {
                // Register 0x28-0x2F are frequency/octave
                let ch = (addr - 0x28) as usize;
                if ch < 8 {
                    self.channels[ch].frequency = ((data as u16 & 0x7F) << 2);
                }
            }
            // Key on/off
            0x08 => {
                let ch = (data >> 4) & 0x07;
                if ch < 8 {
                    self.channels[ch as usize].operators[0].key_on = (data & 0x80) != 0;
                }
            }
            // Volume/panning
            0x60..=0x67 => {
                let ch = (addr - 0x60) as usize;
                if ch < 8 {
                    self.channels[ch].output_level = 127 - (data >> 1);
                }
            }
            _ => {
                // Unimplemented register - ignore
            }
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        // YM2151 is a write-only synthesis chip (no readable registers)
        0xFF
    }

    fn clock(&mut self) {
        // Update LFO
        if self.lfo_enabled {
            self.lfo_counter = self.lfo_counter.wrapping_add(1);
        }

        // Update all channels
        for ch in &mut self.channels {
            if ch.operators[0].key_on {
                ch.output_phase = ch.output_phase.wrapping_add(1);
            }
            for op in &mut ch.operators {
                if op.key_on {
                    op.phase = op.phase.wrapping_add((op.phase_increment >> 16) as u16);
                }
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        // Update sample rate
        self.sample_rate = sample_rate;

        // Fill buffer with samples from all channels
        for frame in buffer.chunks_mut(2) {
            // Accumulate clock cycles
            let cycles_per_sample = self.clock_rate as f32 / sample_rate as f32;
            self.accumulated_cycles += cycles_per_sample;

            // Clock the chip for accumulated cycles
            while self.accumulated_cycles >= 1.0 {
                self.clock();
                self.accumulated_cycles -= 1.0;
            }

            // Get output for this sample
            let (left, right) = self.get_output();
            frame[0] = left;
            frame[1] = right;
        }
    }
}

impl Default for YM2151 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2151_new() {
        let chip = YM2151::new();
        assert_eq!(chip.name(), "YM2151 (OPM)");
        assert_eq!(chip.clock_rate(), 3_579_545);
        assert_eq!(chip.channels.len(), 8);
    }

    #[test]
    fn test_ym2151_channels() {
        let chip = YM2151::new();
        for ch in &chip.channels {
            assert_eq!(ch.operators.len(), 4);
        }
    }

    #[test]
    fn test_ym2151_write_frequency() {
        let mut chip = YM2151::new();
        chip.write(0x28, 0x42); // Channel 0 frequency
        assert!(chip.regs[0x28] == 0x42);
    }

    #[test]
    fn test_ym2151_soundchip_trait() {
        let mut chip = YM2151::new();

        assert_eq!(chip.name(), "YM2151 (OPM)");
        assert_eq!(chip.clock_rate(), 3_579_545);

        chip.reset();
        chip.write(0x08, 0xF0); // Key on channel 0
        chip.clock();

        let mut buffer = [0.0f32; 2];
        chip.generate_samples(&mut buffer, 44100);

        // Output should not be all zeros with key on
        assert!(buffer[0].abs() > 0.0 || buffer[1].abs() > 0.0);
    }
}
