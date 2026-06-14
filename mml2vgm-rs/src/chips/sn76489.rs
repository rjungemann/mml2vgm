//! SN76489 (DCSG) Programmable Sound Generator emulation
//!
//! The SN76489 is a simple PSG (Programmable Sound Generator) chip used in
//! many 8-bit and 16-bit systems including the Sega Master System and
//! Sega Mega Drive/Genesis (as part of the sound system).
//!
//! # Features
//! - 4 sound channels (3 square wave tone generators + 1 noise generator)
//! - 10-bit tone frequency dividers
//! - 4-bit volume attenuation (16 levels)
//! - Noise generator with 2 modes (periodic or white)
//! - Simple register interface
//!
//! # Register Map
//! - 0x80-0x8F: Channel 0 tone (high 4 bits)
//! - 0x90-0x9F: Channel 0 volume/attenuation
//! - 0xA0-0xAF: Channel 1 tone (high 4 bits)
//! - 0xB0-0xBF: Channel 1 volume/attenuation
//! - 0xC0-0xCF: Channel 2 tone (high 4 bits)
//! - 0xD0-0xDF: Channel 2 volume/attenuation
//! - 0xE0-0xE7: Channel 3 noise mode/period
//! - 0xE8-0xEF: Channel 3 volume/attenuation
//! - 0xF0-0xFF: Noise control (period, mode)

use super::SoundChipEmulator;

/// Noise generator mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseMode {
    /// Periodic noise (repeats every N samples)
    Periodic,
    /// White noise (random)
    White,
}

/// A single PSG channel (tone or noise)
#[derive(Debug, Clone, Copy)]
pub struct PsgChannel {
    /// Tone frequency divider (10-bit value)
    pub tone_divider: u16,
    /// Volume attenuation (0-15, where 0 = loudest, 15 = silent)
    pub volume: u8,
    /// Current position in tone divider
    pub tone_counter: u16,
    /// Current output state (true = high, false = low)
    pub output_state: bool,
}

impl Default for PsgChannel {
    fn default() -> Self {
        Self {
            tone_divider: 0,
            volume: 15, // Silent by default
            tone_counter: 0,
            output_state: false,
        }
    }
}

/// SN76489 chip emulator
pub struct SN76489 {
    /// Master clock rate in Hz (NTSC: 3,579,545 Hz)
    clock_rate: u32,

    /// Sample rate for output
    sample_rate: u32,

    /// Clock divider for sample rate conversion
    /// This is the number of chip clocks per audio sample
    clock_divider: f64,

    /// Accumulated clock cycles
    accumulated_cycles: f64,

    /// Tone channels (0-2)
    channels: [PsgChannel; 3],

    /// Noise channel (3)
    noise_channel: PsgChannel,

    /// Noise generator shift register
    noise_shift_register: u16,

    /// Noise generator feedback mask
    noise_feedback: u16,

    /// Noise generator mode
    noise_mode: NoiseMode,

    /// Noise generator period (for periodic mode)
    noise_period: u8,

    /// Last written register
    last_register: u8,

    /// Stereo panning for each channel (left, right)
    /// Each channel can be panned to left, right, or both
    stereo_pan: [StereoPan; 4],

    /// Output buffer for sample generation
    #[allow(dead_code)]
    output_buffer: Vec<f32>,
}

/// Stereo panning mode for a channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StereoPan {
    /// Output to left channel only
    Left,
    /// Output to right channel only
    Right,
    /// Output to both channels
    #[default]
    Center,
}

impl SN76489 {
    /// Create a new SN76489 emulator with the default NTSC clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(3_579_545)
    }

    /// Create a new SN76489 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        let sample_rate: u32 = 44100;
        let clock_divider = clock_rate as f64 / sample_rate as f64;
        Self {
            clock_rate,
            sample_rate,
            clock_divider,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 3],
            noise_channel: Default::default(),
            noise_shift_register: 0x8000, // Initialize with bit 15 set
            noise_feedback: 0x8000,
            noise_mode: NoiseMode::Periodic,
            noise_period: 0,
            last_register: 0,
            stereo_pan: [Default::default(); 4],
            output_buffer: vec![0.0; 1024],
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }

    /// Get the current output sample
    pub fn get_output(&self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Sum outputs from all channels
        for (i, channel) in self.channels.iter().enumerate() {
            let output = self.get_channel_output(channel);
            match self.stereo_pan[i] {
                StereoPan::Left => left += output,
                StereoPan::Right => right += output,
                StereoPan::Center => {
                    left += output;
                    right += output;
                }
            }
        }

        // Add noise channel
        let noise_output = self.get_channel_output(&self.noise_channel);
        match self.stereo_pan[3] {
            StereoPan::Left => left += noise_output,
            StereoPan::Right => right += noise_output,
            StereoPan::Center => {
                left += noise_output;
                right += noise_output;
            }
        }

        // Normalize output (divide by number of channels)
        // This is a simple approach; actual volume mixing would be more sophisticated
        let norm_factor = 4.0; // 4 channels max
        (left / norm_factor, right / norm_factor)
    }

    /// Get output from a single channel
    fn get_channel_output(&self, channel: &PsgChannel) -> f32 {
        if channel.volume == 15 {
            return 0.0; // Silent
        }

        // Volume attenuation: 0 = max volume (1.0), 15 = silent (0.0)
        // The attenuation is approximately 2dB per step
        let attenuation = 1.0 - (channel.volume as f32 / 16.0);
        let volume = attenuation * 0.75; // Scale to reasonable level

        if channel.output_state {
            volume
        } else {
            -volume
        }
    }

    /// Update the noise generator
    fn update_noise(&mut self) {
        // For now, simple noise implementation
        // The actual SN76489 uses a 15-bit LFSR

        // Check if noise should toggle
        if self.noise_channel.tone_counter >= self.noise_channel.tone_divider {
            self.noise_channel.tone_counter = 0;

            // Toggle noise output based on LFSR
            // In periodic mode, use the noise_period
            // In white mode, use random pattern
            match self.noise_mode {
                NoiseMode::Periodic => {
                    if self.noise_period > 0 {
                        // Simple toggle for now
                        self.noise_channel.output_state = !self.noise_channel.output_state;
                    }
                }
                NoiseMode::White => {
                    // Use LFSR
                    let bit = (self.noise_shift_register & 1) == 1;
                    self.noise_shift_register >>= 1;
                    if bit {
                        self.noise_shift_register |= self.noise_feedback;
                    }
                    self.noise_channel.output_state = (self.noise_shift_register & 1) == 1;
                }
            }
        } else {
            self.noise_channel.tone_counter += 1;
        }
    }

    /// Set stereo panning for a channel
    pub fn set_stereo_pan(&mut self, channel: usize, pan: StereoPan) {
        if channel < 4 {
            self.stereo_pan[channel] = pan;
        }
    }
}

impl SoundChipEmulator for SN76489 {
    fn name(&self) -> &'static str {
        "SN76489 (DCSG)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, _data: u8) {
        // SN76489 uses a single-byte serial interface.
        // `addr` carries the SN76489 byte directly (as VGM opcode 0x50 passes it).
        //
        // Latch byte (bit 7 = 1):  1 CC T DDDD
        //   CC   = channel (00=ch0, 01=ch1, 10=ch2, 11=noise)
        //   T    = type (0=tone/freq, 1=attenuation/volume)
        //   DDDD = low 4 bits of data
        //
        // Data byte (bit 7 = 0):  0 X HHHHHH
        //   HHHHHH = high 6 bits of frequency for the last-latched channel
        let byte = addr;

        if (byte & 0x80) != 0 {
            // Latch byte
            let channel = ((byte >> 5) & 0x03) as usize;
            let is_volume = (byte & 0x10) != 0;
            let data4 = byte & 0x0F;
            self.last_register = byte; // remember for subsequent data byte

            if is_volume {
                if channel < 3 {
                    self.channels[channel].volume = data4;
                } else {
                    self.noise_channel.volume = data4;
                }
            } else if channel < 3 {
                // Update low 4 bits of tone divider, preserve high bits
                self.channels[channel].tone_divider =
                    (self.channels[channel].tone_divider & 0x3F0) | data4 as u16;
            } else {
                // Noise control: bit 2 = white/periodic, bits 1:0 = period select
                self.noise_mode = if (data4 & 0x04) != 0 {
                    NoiseMode::White
                } else {
                    NoiseMode::Periodic
                };
                self.noise_period = data4 & 0x03;
                self.noise_channel.tone_divider = match self.noise_period {
                    0 => 16,
                    1 => 32,
                    2 => 64,
                    _ => self.channels[2].tone_divider, // follow ch2
                };
                self.noise_feedback = if (data4 & 0x04) != 0 { 0x8000 } else { 0x4000 };
            }
        } else {
            // Data byte: high 6 bits of frequency for the last-latched tone channel
            let last = self.last_register;
            let channel = ((last >> 5) & 0x03) as usize;
            let is_volume = (last & 0x10) != 0;
            if !is_volume && channel < 3 {
                let data6 = (byte & 0x3F) as u16;
                // Preserve low 4 bits, set high 6 bits
                self.channels[channel].tone_divider =
                    (data6 << 4) | (self.channels[channel].tone_divider & 0x0F);
            }
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        // SN76489 doesn't support reading registers
        // Return 0xFF as per specification
        0xFF
    }

    fn clock(&mut self) {
        // Advance all channel counters
        for channel in &mut self.channels {
            if channel.tone_counter >= channel.tone_divider {
                channel.tone_counter = 0;
                channel.output_state = !channel.output_state;
            } else {
                channel.tone_counter += 1;
            }
        }

        // Update noise generator
        self.update_noise();
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        // Set sample rate if changed
        if sample_rate != self.sample_rate {
            self.set_sample_rate(sample_rate);
        }

        // Generate samples for each frame
        for frame in buffer.chunks_mut(2) {
            // Accumulate clock cycles for this sample
            self.accumulated_cycles += self.clock_divider;

            // Clock the chip for the accumulated cycles
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

impl Default for SN76489 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sn76489_new() {
        let chip = SN76489::new();
        assert_eq!(chip.name(), "SN76489 (DCSG)");
        assert_eq!(chip.clock_rate(), 3_579_545);
    }

    #[test]
    fn test_sn76489_reset() {
        let mut chip = SN76489::new();
        // Latch byte: ch0 tone, data=0 → 1_00_0_0000 = 0x80
        chip.write(0x80, 0);
        chip.reset();
        assert_eq!(chip.channels[0].tone_divider, 0);
    }

    #[test]
    fn test_sn76489_write_tone() {
        let mut chip = SN76489::new();
        // Tone divider target: 0x012 = low4=0x2, high6=0x1
        // Latch byte: ch0 tone, low4=2 → 1_00_0_0010 = 0x82
        chip.write(0x82, 0);
        // Data byte: high6=1 → 0_X_000001 = 0x01
        chip.write(0x01, 0);
        assert_eq!(chip.channels[0].tone_divider, 0x012);
    }

    #[test]
    fn test_sn76489_write_volume() {
        let mut chip = SN76489::new();
        // Latch byte: ch0 volume, data=0x0A → 1_00_1_1010 = 0x9A
        chip.write(0x9A, 0);
        assert_eq!(chip.channels[0].volume, 0x0A);
    }

    #[test]
    fn test_sn76489_clock() {
        let mut chip = SN76489::new();
        // With tone_divider=1, the counter will reach it on first clock
        // The counter starts at 0, so: 0 >= 1? No, counter becomes 1
        // On second clock: 1 >= 1? Yes, toggle and reset counter
        chip.channels[0].tone_divider = 1;
        chip.channels[0].volume = 0; // Max volume
        chip.channels[0].output_state = false;

        // Clock once - counter goes from 0 to 1, no toggle yet
        chip.clock();
        assert_eq!(chip.channels[0].tone_counter, 1);
        assert!(!chip.channels[0].output_state);

        // Clock again - counter is 1 >= 1, so toggle
        chip.clock();
        assert!(chip.channels[0].output_state);

        // Clock again - counter was reset to 0, goes to 1, no toggle
        chip.clock();
        assert!(chip.channels[0].output_state);

        // Clock again - counter is 1 >= 1, toggle back
        chip.clock();
        assert!(!chip.channels[0].output_state);
    }

    #[test]
    fn test_sn76489_soundchip_trait() {
        let mut chip = SN76489::new();

        // Verify trait methods work
        assert_eq!(chip.name(), "SN76489 (DCSG)");
        assert_eq!(chip.clock_rate(), 3_579_545);

        chip.reset();
        chip.write(0x82, 0); // ch0 tone latch, low4=2
        chip.clock();

        let mut buffer = [0.0f32; 2];
        chip.generate_samples(&mut buffer, 44100);
    }
}
