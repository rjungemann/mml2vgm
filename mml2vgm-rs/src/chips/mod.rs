//! Sound chip emulation module

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
pub mod ay8910;
pub mod huc6280;
pub mod ym2413;
pub mod k051649;
pub mod nes_apu;
pub mod pokey;
pub mod dmg;
pub mod vrc6;
pub mod k053260;
pub mod k054539;
pub mod qsound;

use crate::{MmlError, MmlResult};

/// Trait for all sound chips
pub trait SoundChipEmulator {
    fn name(&self) -> &'static str;
    fn clock_rate(&self) -> u32;
    fn reset(&mut self);
    fn write(&mut self, addr: u8, data: u8);
    fn read(&self, addr: u8) -> u8 { 0xFF }
    fn clock(&mut self);
    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32);
    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let _ = port;
        self.write(addr, data);
    }
    fn is_initialized(&self) -> bool { true }
    fn load_pcm_data(&mut self, _block_type: u8, _data: &[u8]) {}
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
