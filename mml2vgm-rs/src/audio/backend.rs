//! Audio backend abstraction
//!
//! Provides a trait-based abstraction for audio output across different backends.

use crate::MmlResult;
use std::fmt;

/// Audio backend error types
#[derive(Debug, Clone)]
pub enum AudioError {
    /// Device not available or not found
    DeviceNotFound(String),
    /// Stream initialization failed
    StreamError(String),
    /// Buffer underrun or overrun
    BufferError(String),
    /// Unsupported sample rate or channel count
    UnsupportedFormat(String),
    /// Generic backend error
    BackendError(String),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::DeviceNotFound(msg) => write!(f, "Audio device not found: {}", msg),
            AudioError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            AudioError::BufferError(msg) => write!(f, "Buffer error: {}", msg),
            AudioError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            AudioError::BackendError(msg) => write!(f, "Backend error: {}", msg),
        }
    }
}

impl std::error::Error for AudioError {}

/// Audio backend trait for playback
///
/// This trait defines the interface that any audio backend must implement.
/// Backends are responsible for:
/// - Initializing audio devices
/// - Managing playback state
/// - Sending audio samples to the device
pub trait AudioBackend: Send {
    /// Initialize the audio backend with specified parameters
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz (typically 44100 or 48000)
    /// * `channels` - Number of audio channels (1 for mono, 2 for stereo)
    fn init(&mut self, sample_rate: u32, channels: u16) -> MmlResult<()>;

    /// Start audio playback
    fn start(&mut self) -> MmlResult<()>;

    /// Stop audio playback
    fn stop(&mut self) -> MmlResult<()>;

    /// Write audio samples to the backend
    ///
    /// For stereo output, samples should be interleaved (L, R, L, R, ...)
    fn write_samples(&mut self, samples: &[f32]) -> MmlResult<()>;

    /// Check if playback is currently active
    fn is_playing(&self) -> bool;

    /// Get the current playback position in samples
    fn position(&self) -> u64 {
        0
    }

    /// Flush any buffered samples
    fn flush(&mut self) -> MmlResult<()> {
        Ok(())
    }
}
