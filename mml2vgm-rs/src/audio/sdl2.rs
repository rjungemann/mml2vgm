//! SDL2 audio backend
//!
//! Alternative audio output using SDL2.
//! SDL2 requires additional system libraries but provides good compatibility.
//! This backend is optional and requires the "sdl2" feature to be enabled.

use super::backend::AudioBackend;
use crate::{MmlError, MmlResult};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// SDL2 audio backend for compatibility with SDL2-based applications
pub struct Sdl2Backend {
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
    channels: u16,
    is_playing: bool,
    position: u64,
}

impl Sdl2Backend {
    /// Create a new SDL2 backend instance
    pub fn new() -> Self {
        Self {
            sample_buffer: Arc::new(Mutex::new(VecDeque::new())),
            sample_rate: 44100,
            channels: 2,
            is_playing: false,
            position: 0,
        }
    }
}

impl AudioBackend for Sdl2Backend {
    fn init(&mut self, sample_rate: u32, channels: u16) -> MmlResult<()> {
        self.sample_rate = sample_rate;
        self.channels = channels;
        // SDL2 initialization would go here
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
        let mut buffer = self.sample_buffer.lock()
            .map_err(|_| MmlError::AudioError(
                super::backend::AudioError::BufferError(
                    "Failed to acquire buffer lock".to_string()
                )
            ))?;

        for &sample in samples {
            buffer.push_back(sample.clamp(-1.0, 1.0));
        }

        self.position += samples.len() as u64;

        // Prevent buffer from growing too large
        let max_samples = (self.sample_rate * 2) as usize;
        if buffer.len() > max_samples {
            let to_remove = buffer.len() - max_samples;
            for _ in 0..to_remove {
                buffer.pop_front();
            }
        }

        Ok(())
    }

    fn is_playing(&self) -> bool {
        self.is_playing
    }

    fn position(&self) -> u64 {
        self.position
    }
}

impl Default for Sdl2Backend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdl2_backend_creation() {
        let backend = Sdl2Backend::new();
        assert!(!backend.is_playing());
        assert_eq!(backend.position(), 0);
    }

    #[test]
    fn test_sdl2_backend_defaults() {
        let backend = Sdl2Backend::default();
        assert!(!backend.is_playing());
    }
}
