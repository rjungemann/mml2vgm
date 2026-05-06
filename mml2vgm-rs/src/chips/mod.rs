//! Sound chip emulation module
//!
//! This module contains emulation implementations for various sound chips.
//! It provides a unified interface through the `SoundChip` trait for writing
//! to chip registers and generating audio samples.
//!
//! # Architecture
//!
//! Each sound chip has:
//! - A struct representing its internal state
//! - Implementation of the `SoundChip` trait
//! - Register-level emulation
//! - Sample generation

pub mod ym2612;
pub mod sn76489;
pub mod ym2151;
pub mod ym2608;
pub mod rf5c164;
pub mod ym2203;
pub mod ym3526;
pub mod y8950;
pub mod ym3812;
pub mod ymf262;
pub mod segapcm;
pub mod c140;
pub mod c352;

use crate::{MmlError, MmlResult};

/// Trait for all sound chips
///
/// This trait provides a common interface for all supported sound chips,
/// allowing the player to work with any chip through the same interface.
pub trait SoundChipEmulator {
    /// Get the chip name
    fn name(&self) -> &'static str;

    /// Get the default clock rate in Hz
    fn clock_rate(&self) -> u32;

    /// Reset the chip to power-on state
    fn reset(&mut self);

    /// Write to a chip register
    fn write(&mut self, addr: u8, data: u8);

    /// Read from a chip register (returns 0xFF if not supported)
    fn read(&self, addr: u8) -> u8 {
        0xFF
    }

    /// Advance the chip state by one clock cycle
    fn clock(&mut self);

    /// Generate audio samples
    /// 
    /// Fills the provided buffer with interleaved stereo samples.
    /// Each sample should be in the range [-1.0, 1.0].
    /// The number of samples generated is buffer.len() / 2 (stereo).
    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32);

    /// Write to a chip register with explicit port selection.
    /// Port 0 = first register bank, Port 1 = second register bank (e.g. YM2608 ch4-6).
    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let _ = port;
        self.write(addr, data);
    }

    /// Check if the chip has been initialized
    fn is_initialized(&self) -> bool {
        true
    }
}

/// Clock the chip for a specific number of cycles
pub fn clock_chip(chip: &mut dyn SoundChipEmulator, cycles: u32) {
    for _ in 0..cycles {
        chip.clock();
    }
}

/// Generate samples from multiple chips and mix them
pub fn generate_mixed_samples(
    chips: &mut [&mut dyn SoundChipEmulator],
    buffer: &mut [f32],
    sample_rate: u32,
) {
    // Create temporary buffer for each chip
    let mut temp_buffers: Vec<Vec<f32>> = chips
        .iter()
        .map(|_| vec![0.0; buffer.len()])
        .collect();

    // Generate samples from each chip
    for (i, chip) in chips.iter_mut().enumerate() {
        chip.generate_samples(&mut temp_buffers[i], sample_rate);
    }

    // Mix all buffers
    for i in 0..buffer.len() {
        let mut mixed = 0.0f32;
        for temp_buf in &temp_buffers {
            mixed += temp_buf[i];
        }
        // Clamp to prevent overflow
        buffer[i] = mixed.clamp(-1.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_chip() {
        // This test verifies the clock_chip function works
        // We'll need a mock chip for this
        // For now, just verify the function compiles
    }
}
