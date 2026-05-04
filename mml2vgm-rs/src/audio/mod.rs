//! Audio playback module
//!
//! This module provides audio output backends for VGM file playback.
//! It includes multiple implementations for cross-platform compatibility:
//! - CPAL for low-level cross-platform audio
//! - Rodio for convenient high-level audio playback
//! - SDL2 as an alternative backend

pub mod backend;
pub mod cpal;
pub mod rodio;
pub mod sdl2;

pub use backend::{AudioBackend, AudioError};
pub use cpal::CpalBackend;
pub use rodio::RodioBackend;
pub use sdl2::Sdl2Backend;
