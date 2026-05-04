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
            return Err(MmlError::UnsupportedChip(
                format!("Chip {:?} already added", chip),
            ));
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
            _ => {
                return Err(MmlError::UnsupportedChip(
                    format!("Chip {:?} not yet implemented", chip),
                ))
            }
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
            Err(MmlError::UnsupportedChip(format!("Chip {:?} not found", chip)))
        }
    }

    /// Read from a chip register (if supported)
    pub fn read_register(&self, chip: SoundChip, addr: u8) -> MmlResult<u8> {
        if let Some(emulator) = self.chips.get(&chip) {
            Ok(emulator.read(addr))
        } else {
            Err(MmlError::UnsupportedChip(format!("Chip {:?} not found", chip)))
        }
    }

    /// Reset a chip
    pub fn reset_chip(&mut self, chip: SoundChip) -> MmlResult<()> {
        if let Some(emulator) = self.chips.get_mut(&chip) {
            emulator.reset();
            Ok(())
        } else {
            Err(MmlError::UnsupportedChip(format!("Chip {:?} not found", chip)))
        }
    }

    /// Reset all chips
    pub fn reset_all(&mut self) {
        for (_, emulator) in &mut self.chips {
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

    /// Generate the next batch of samples
    pub fn generate_samples(&mut self, sample_count: usize) -> MmlResult<Vec<f32>> {
        if self.state != ChipPlayerState::Playing {
            return Ok(vec![0.0; sample_count * 2]); // Return silence for stereo
        }

        self.sample_buffer.clear();
        self.sample_buffer.resize(sample_count * 2, 0.0); // Stereo samples

        // Collect mutable references to all chip emulators
        let mut chip_refs: Vec<&mut dyn SoundChipEmulator> = self
            .chips
            .values_mut()
            .map(|b| b.as_mut() as &mut dyn SoundChipEmulator)
            .collect();

        if chip_refs.is_empty() {
            self.position += sample_count as u64;
            return Ok(self.sample_buffer.clone());
        }

        // Generate and mix samples from all chips
        for chip in &mut chip_refs {
            chip.generate_samples(&mut self.sample_buffer, self.sample_rate);
        }

        // Clamp values to prevent clipping
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
}
