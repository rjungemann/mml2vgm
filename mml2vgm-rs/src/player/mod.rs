//! Player module for VGM/XGM/ZGM playback
//!
//! This module provides implementations for playing VGM files and real-time chip emulation.
//! It includes:
//! - VgmPlayer for playing pre-recorded VGM files
//! - ChipPlayer for real-time sound chip emulation

pub mod chip_player;
pub mod vgm_player;

pub use chip_player::{ChipPlayer, ChipPlayerState};
pub use vgm_player::{PlayerState, VgmPlayer};
