//! CPAL audio backend
//!
//! Cross-platform audio output using the cpal (Cross-Platform Audio Library).
//! Supports Windows, macOS, and Linux with various audio APIs.

use super::backend::AudioBackend;
use crate::{MmlError, MmlResult};

/// CPAL audio backend for cross-platform playback
///
/// This is a stub implementation as CPAL's callback architecture
/// doesn't easily integrate with the Send trait requirement.
/// Use RodioBackend for actual playback instead.
pub struct CpalBackend {
    sample_rate: u32,
    channels: u16,
    is_playing: bool,
    position: u64,
}

impl CpalBackend {
    /// Create a new CPAL backend instance
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            is_playing: false,
            position: 0,
        }
    }
}

impl AudioBackend for CpalBackend {
    fn init(&mut self, sample_rate: u32, channels: u16) -> MmlResult<()> {
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.position = 0;
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

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpal_backend_creation() {
        let backend = CpalBackend::new();
        assert!(!backend.is_playing());
        assert_eq!(backend.position(), 0);
    }

    #[test]
    fn test_cpal_backend_defaults() {
        let backend = CpalBackend::default();
        assert!(!backend.is_playing());
    }
}
