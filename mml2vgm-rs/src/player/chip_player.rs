//! Real-time chip emulation player
//!
//! Handles real-time emulation and playback of sound chips.
//! Allows writing directly to chip registers and generating audio samples.

use crate::audio::AudioBackend;
use crate::chips::SoundChipEmulator;
use crate::{MmlError, MmlResult, SoundChip};
use std::collections::HashMap;

/// Chip player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipPlayerState {
    /// Playback is stopped
    Stopped,
    /// Playback is in progress
    Playing,
    /// Playback is paused
    Paused,
}

/// Chip player for real-time emulation
pub struct ChipPlayer {
    chips: HashMap<SoundChip, Box<dyn SoundChipEmulator>>,
    /// Per-chip linear gain applied during mixing. Defaults to 1.0 for any
    /// chip without an explicit entry. 0.0 mutes the chip (its emulator is
    /// still clocked so envelopes/timers stay coherent if the gain comes back).
    chip_gains: HashMap<SoundChip, f32>,
    /// Scratch buffer reused across mixing iterations to avoid per-call
    /// allocation. Same length as `sample_buffer` (stereo-interleaved).
    chip_scratch: Vec<f32>,
    sample_rate: u32,
    state: ChipPlayerState,
    audio_backend: Option<Box<dyn AudioBackend>>,
    sample_buffer: Vec<f32>,
    position: u64,
}

impl ChipPlayer {
    /// Create a new chip player
    pub fn new() -> Self {
        Self {
            chips: HashMap::new(),
            chip_gains: HashMap::new(),
            chip_scratch: Vec::with_capacity(4096 * 2),
            sample_rate: 44100,
            state: ChipPlayerState::Stopped,
            audio_backend: None,
            sample_buffer: Vec::with_capacity(4096 * 2), // Stereo samples
            position: 0,
        }
    }

    /// Add a sound chip to the player
    pub fn add_chip(&mut self, chip: SoundChip) -> MmlResult<()> {
        if self.chips.contains_key(&chip) {
            return Err(MmlError::UnsupportedChip(format!(
                "Chip {:?} already added",
                chip
            )));
        }

        // Create appropriate chip emulator based on type
        let emulator: Box<dyn SoundChipEmulator> = match chip {
            SoundChip::YM2612 => Box::new(crate::chips::ym2612::YM2612::new()),
            SoundChip::SN76489 => Box::new(crate::chips::sn76489::SN76489::new()),
            SoundChip::YM2151 => Box::new(crate::chips::ym2151::YM2151::new()),
            SoundChip::YM2608 => Box::new(crate::chips::ym2608::YM2608::new()),
            SoundChip::RF5C164 => Box::new(crate::chips::rf5c164::RF5C164::new()),
            SoundChip::YM2203 => Box::new(crate::chips::ym2203::YM2203::new()),
            SoundChip::YM3526 => Box::new(crate::chips::ym3526::YM3526::new()),
            SoundChip::Y8950 => Box::new(crate::chips::y8950::Y8950::new()),
            SoundChip::YM3812 => Box::new(crate::chips::ym3812::YM3812::new()),
            SoundChip::YMF262 => Box::new(crate::chips::ymf262::YMF262::new()),
            SoundChip::SegaPCM => Box::new(crate::chips::segapcm::SegaPCM::new()),
            SoundChip::C140 => Box::new(crate::chips::c140::C140::new()),
            SoundChip::C352 => Box::new(crate::chips::c352::C352::new()),
            SoundChip::AY8910 => Box::new(crate::chips::ay8910::AY8910::new()),
            SoundChip::HuC6280 => Box::new(crate::chips::huc6280::HuC6280::new()),
            SoundChip::YM2413 => Box::new(crate::chips::ym2413::YM2413::new()),
            SoundChip::K051649 => Box::new(crate::chips::k051649::K051649::new()),
            SoundChip::NES => Box::new(crate::chips::nes_apu::NesApu::new()),
            SoundChip::POKEY => Box::new(crate::chips::pokey::Pokey::new()),
            SoundChip::DMG => Box::new(crate::chips::dmg::Dmg::new()),
            SoundChip::VRC6 => Box::new(crate::chips::vrc6::VRC6::new()),
            SoundChip::K053260 => Box::new(crate::chips::k053260::K053260::new()),
            SoundChip::K054539 => Box::new(crate::chips::k054539::K054539::new()),
            SoundChip::QSound => Box::new(crate::chips::qsound::QSound::new()),
            // Variant/extended chips reuse compatible base emulators
            SoundChip::YM2610B => Box::new(crate::chips::ym2608::YM2608::new()),
            SoundChip::YM2609 => Box::new(crate::chips::ym2608::YM2608::new()),
            SoundChip::SN76489X2 => Box::new(crate::chips::sn76489::SN76489::new()),
            SoundChip::YM2612X => Box::new(crate::chips::ym2612::YM2612::new()),
            SoundChip::YM2612X2 => Box::new(crate::chips::ym2612::YM2612::new()),
            SoundChip::YMF271 => Box::new(crate::chips::ymf271::YMF271::new()),
            SoundChip::MIDI => Box::new(crate::chips::SilentChip::new("MIDI", 0)),
            SoundChip::CONDUCTOR => Box::new(crate::chips::SilentChip::new("CONDUCTOR", 0)),
        };

        self.chips.insert(chip, emulator);
        Ok(())
    }

    /// Remove a chip from the player
    pub fn remove_chip(&mut self, chip: SoundChip) -> MmlResult<()> {
        self.chips
            .remove(&chip)
            .ok_or_else(|| MmlError::UnsupportedChip(format!("Chip {:?} not found", chip)))?;
        Ok(())
    }

    /// Get the number of active chips
    pub fn chip_count(&self) -> usize {
        self.chips.len()
    }

    /// Write to a chip register
    pub fn write_register(&mut self, chip: SoundChip, addr: u8, data: u8) -> MmlResult<()> {
        if let Some(emulator) = self.chips.get_mut(&chip) {
            emulator.write(addr, data);
            Ok(())
        } else {
            Err(MmlError::UnsupportedChip(format!(
                "Chip {:?} not found",
                chip
            )))
        }
    }

    /// Read from a chip register (if supported)
    pub fn read_register(&self, chip: SoundChip, addr: u8) -> MmlResult<u8> {
        if let Some(emulator) = self.chips.get(&chip) {
            Ok(emulator.read(addr))
        } else {
            Err(MmlError::UnsupportedChip(format!(
                "Chip {:?} not found",
                chip
            )))
        }
    }

    /// Reset a chip
    pub fn reset_chip(&mut self, chip: SoundChip) -> MmlResult<()> {
        if let Some(emulator) = self.chips.get_mut(&chip) {
            emulator.reset();
            Ok(())
        } else {
            Err(MmlError::UnsupportedChip(format!(
                "Chip {:?} not found",
                chip
            )))
        }
    }

    /// Reset all chips
    pub fn reset_all(&mut self) {
        for emulator in self.chips.values_mut() {
            emulator.reset();
        }
    }

    /// Start playback
    pub fn play(&mut self) -> MmlResult<()> {
        if self.chips.is_empty() {
            return Err(MmlError::ChipInitFailed(
                "No chips added. Call add_chip() first.".to_string(),
            ));
        }

        self.state = ChipPlayerState::Playing;

        if let Some(backend) = &mut self.audio_backend {
            backend.start()?;
        }

        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) -> MmlResult<()> {
        self.state = ChipPlayerState::Stopped;
        self.position = 0;

        if let Some(backend) = &mut self.audio_backend {
            backend.stop()?;
        }

        Ok(())
    }

    /// Pause playback
    pub fn pause(&mut self) -> MmlResult<()> {
        self.state = ChipPlayerState::Paused;

        if let Some(backend) = &mut self.audio_backend {
            backend.stop()?;
        }

        Ok(())
    }

    /// Resume playback
    pub fn resume(&mut self) -> MmlResult<()> {
        self.state = ChipPlayerState::Playing;

        if let Some(backend) = &mut self.audio_backend {
            backend.start()?;
        }

        Ok(())
    }

    /// Check if playback is active
    pub fn is_playing(&self) -> bool {
        self.state == ChipPlayerState::Playing
    }

    /// Get current playback position in samples
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Get the current player state
    pub fn state(&self) -> ChipPlayerState {
        self.state
    }

    /// Set the linear gain applied to a chip's output during mixing.
    /// `gain = 1.0` is unity, `0.0` mutes. Values are clamped to a sane
    /// range so a runaway slider can't blow speakers.
    pub fn set_chip_gain(&mut self, chip: SoundChip, gain: f32) {
        self.chip_gains.insert(chip, gain.clamp(0.0, 4.0));
    }

    /// Get the linear gain for a chip. Returns 1.0 if no explicit gain was
    /// set (the default).
    pub fn get_chip_gain(&self, chip: SoundChip) -> f32 {
        self.chip_gains.get(&chip).copied().unwrap_or(1.0)
    }

    /// Generate the next batch of samples
    pub fn generate_samples(&mut self, sample_count: usize) -> MmlResult<Vec<f32>> {
        // When stopped, still generate samples so WASM callers (which manage
        // their own state externally) receive chip output immediately after
        // register writes without needing an explicit play() call.
        // CLI usage sets state to Playing before calling generate_samples.
        if self.state == ChipPlayerState::Paused {
            return Ok(vec![0.0; sample_count * 2]); // Return silence when paused
        }

        let total = sample_count * 2; // stereo interleaved
        self.sample_buffer.clear();
        self.sample_buffer.resize(total, 0.0);

        if self.chips.is_empty() {
            self.position += sample_count as u64;
            return Ok(self.sample_buffer.clone());
        }

        // Each chip's `generate_samples` *overwrites* the buffer it's given
        // (rather than accumulating), so summing chips requires rendering
        // each into a private scratch buffer and mixing it in. The scratch
        // buffer is reused across iterations to avoid per-call allocation.
        if self.chip_scratch.len() < total {
            self.chip_scratch.resize(total, 0.0);
        }
        let scratch = &mut self.chip_scratch[..total];

        for (chip_type, emulator) in &mut self.chips {
            let gain = self.chip_gains.get(chip_type).copied().unwrap_or(1.0);

            // Zero the scratch buffer so chip writes start from silence.
            for v in scratch.iter_mut() {
                *v = 0.0;
            }
            emulator.generate_samples(scratch, self.sample_rate);

            // Skip the mix-in when muted, but keep the emulator clocked above
            // so envelopes/timers don't desync when the gain comes back.
            if gain == 0.0 {
                continue;
            }
            if gain == 1.0 {
                for (dst, src) in self.sample_buffer.iter_mut().zip(scratch.iter()) {
                    *dst += *src;
                }
            } else {
                for (dst, src) in self.sample_buffer.iter_mut().zip(scratch.iter()) {
                    *dst += *src * gain;
                }
            }
        }

        // Clamp to prevent clipping when multiple chips overlap loudly.
        for sample in &mut self.sample_buffer {
            *sample = sample.clamp(-1.0, 1.0);
        }

        self.position += sample_count as u64;

        if let Some(backend) = &mut self.audio_backend {
            backend.write_samples(&self.sample_buffer)?;
        }

        Ok(self.sample_buffer.clone())
    }

    /// Set the audio backend for playback
    pub fn set_audio_backend(&mut self, backend: Box<dyn AudioBackend>) {
        self.audio_backend = Some(backend);
    }

    /// Set the sample rate
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }
}

impl Default for ChipPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip_player_creation() {
        let player = ChipPlayer::new();
        assert!(!player.is_playing());
        assert_eq!(player.position(), 0);
        assert_eq!(player.state, ChipPlayerState::Stopped);
        assert_eq!(player.chip_count(), 0);
    }

    #[test]
    fn test_chip_player_defaults() {
        let player = ChipPlayer::default();
        assert!(!player.is_playing());
    }

    #[test]
    fn test_chip_player_add_chip() {
        let mut player = ChipPlayer::new();
        assert!(player.add_chip(SoundChip::SN76489).is_ok());
        assert_eq!(player.chip_count(), 1);
        // Adding same chip again should fail
        assert!(player.add_chip(SoundChip::SN76489).is_err());
    }

    #[test]
    fn test_chip_player_remove_chip() {
        let mut player = ChipPlayer::new();
        player.add_chip(SoundChip::SN76489).unwrap();
        assert!(player.remove_chip(SoundChip::SN76489).is_ok());
        assert_eq!(player.chip_count(), 0);
    }

    // ── Batch D4: variant / alias chips ─────────────────────────────────────

    #[test]
    fn test_batch_d4_variant_chips_add_successfully() {
        let variants = [
            SoundChip::YM2610B,
            SoundChip::YM2609,
            SoundChip::SN76489X2,
            SoundChip::YM2612X,
            SoundChip::YM2612X2,
        ];
        for chip in variants {
            let mut player = ChipPlayer::new();
            assert!(
                player.add_chip(chip).is_ok(),
                "add_chip failed for {:?}",
                chip
            );
            assert_eq!(player.chip_count(), 1);
        }
    }

    #[test]
    fn test_batch_d4_declared_chips_produce_silence() {
        let declared = [SoundChip::YMF271, SoundChip::MIDI, SoundChip::CONDUCTOR];
        for chip in declared {
            let mut player = ChipPlayer::new();
            assert!(
                player.add_chip(chip).is_ok(),
                "add_chip failed for {:?}",
                chip
            );
            // Generate a small buffer and verify it is silent
            let buf = player
                .generate_samples(32)
                .expect("generate_samples failed");
            assert!(
                buf.iter().all(|&s| s == 0.0),
                "silent chip {:?} produced non-zero samples",
                chip
            );
        }
    }
}
