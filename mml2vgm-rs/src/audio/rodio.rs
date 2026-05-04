//! Rodio audio backend
//!
//! Higher-level audio output using the rodio library.
//! Rodio provides a simpler interface than CPAL for common use cases.

use super::backend::AudioBackend;
use crate::{MmlError, MmlResult};

/// Rodio audio backend for convenient playback
///
/// This is a stub implementation. Full integration requires
/// designing a proper WAV/PCM source that implements the Source trait.
pub struct RodioBackend {
    sample_rate: u32,
    channels: u16,
    is_playing: bool,
    position: u64,
}

impl RodioBackend {
    /// Create a new Rodio backend instance
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            is_playing: false,
            position: 0,
        }
    }
}

impl AudioBackend for RodioBackend {
    fn init(&mut self, sample_rate: u32, channels: u16) -> MmlResult<()> {
        self.sample_rate = sample_rate;
        self.channels = channels;
        Ok(())
    }

    fn start(&mut self) -> MmlResult<()> {
        self.is_playing = true;
        Ok(())
    }

    fn stop(&mut self) -> MmlResult<()> {
        self.is_playing = false;
        Ok(())
    }

    fn write_samples(&mut self, samples: &[f32]) -> MmlResult<()> {
        self.position += samples.len() as u64;
        Ok(())
    }

    fn is_playing(&self) -> bool {
        self.is_playing
    }

    fn position(&self) -> u64 {
        self.position
    }
}

impl Default for RodioBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rodio_backend_creation() {
        let backend = RodioBackend::new();
        assert!(!backend.is_playing());
        assert_eq!(backend.position(), 0);
    }

    #[test]
    fn test_rodio_backend_defaults() {
        let backend = RodioBackend::default();
        assert!(!backend.is_playing());
    }
}
