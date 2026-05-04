//! Player module for VGM/XGM/ZGM playback
//!
//! This module provides implementations for playing VGM files and real-time chip emulation.
//! It includes:
//! - VgmPlayer for playing pre-recorded VGM files
//! - ChipPlayer for real-time sound chip emulation

pub mod vgm_player;
pub mod chip_player;

pub use vgm_player::{VgmPlayer, PlayerState};
pub use chip_player::{ChipPlayer, ChipPlayerState};
