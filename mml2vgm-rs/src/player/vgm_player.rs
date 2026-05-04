//! VGM file player
//!
//! Handles playback of VGM (Video Game Music) format files.
//! Parses VGM headers, initializes sound chips, and processes commands.

use crate::audio::AudioBackend;
use crate::chips::SoundChipEmulator;
use crate::{MmlError, MmlResult, SoundChip, VgmHeader};
use std::io::Cursor;

/// VGM file player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

/// VGM file player
pub struct VgmPlayer {
    header: Option<VgmHeader>,
    commands: Vec<(u32, Vec<u8>)>, // (time in samples, command bytes)
    chips: Vec<(SoundChip, Box<dyn SoundChipEmulator>)>,
    current_sample: u64,
    current_command: usize,
    sample_rate: u32,
    state: PlayerState,
    audio_backend: Option<Box<dyn AudioBackend>>,
}

impl VgmPlayer {
    /// Create a new VGM player
    pub fn new() -> Self {
        Self {
            header: None,
            commands: Vec::new(),
            chips: Vec::new(),
            current_sample: 0,
            current_command: 0,
            sample_rate: 44100,
            state: PlayerState::Stopped,
            audio_backend: None,
        }
    }

    /// Load a VGM file from bytes
    pub fn load(&mut self, data: &[u8]) -> MmlResult<()> {
        if data.len() < 64 {
            return Err(MmlError::InvalidVgmHeader(
                "VGM file too small".to_string(),
            ));
        }

        // Parse header
        let header = Self::parse_header(data)?;
        self.header = Some(header);
        self.sample_rate = header.rate;

        // Parse commands
        self.commands = Self::parse_commands(data, &header)?;

        // Reset playback position
        self.current_sample = 0;
        self.current_command = 0;

        Ok(())
    }

    /// Parse VGM header
    fn parse_header(data: &[u8]) -> MmlResult<VgmHeader> {
        let mut cursor = Cursor::new(data);
        let mut header = VgmHeader::new();

        // Check identifier
        if &data[0..4] != b"Vgm " {
            return Err(MmlError::InvalidVgmHeader(
                "Invalid VGM identifier".to_string(),
            ));
        }

        // Read header fields
        let eof_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

        if eof_offset + 4 > data.len() {
            return Err(MmlError::InvalidVgmHeader(
                "VGM file truncated".to_string(),
            ));
        }

        header.version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        header.sn76489_clock =
            u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        header.ym2413_clock =
            u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        header.gd3_offset = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        header.total_samples =
            u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        header.loop_offset = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        header.loop_samples =
            u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        header.rate = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);

        if header.rate == 0 {
            header.rate = 44100;
        }

        Ok(header)
    }

    /// Parse VGM commands from data
    fn parse_commands(data: &[u8], header: &VgmHeader) -> MmlResult<Vec<(u32, Vec<u8>)>> {
        let mut commands = Vec::new();
        let mut offset = 0x40; // Start after header
        let mut current_time: u32 = 0;

        let data_end = if header.eof_offset > 0 {
            (header.eof_offset as usize) + 4
        } else {
            data.len()
        };

        while offset < data_end && offset < data.len() {
            let cmd = data[offset];
            offset += 1;

            match cmd {
                0x50 => {
                    // SN76489 write
                    if offset + 1 <= data.len() {
                        let value = data[offset];
                        offset += 1;
                        commands.push((current_time, vec![cmd, value]));
                    }
                }
                0x52 | 0x53 => {
                    // YM2612 write (port 0 or 1)
                    if offset + 2 <= data.len() {
                        let addr = data[offset];
                        let value = data[offset + 1];
                        offset += 2;
                        commands.push((current_time, vec![cmd, addr, value]));
                    }
                }
                0x61 => {
                    // Wait n samples
                    if offset + 2 <= data.len() {
                        let samples =
                            u16::from_le_bytes([data[offset], data[offset + 1]]) as u32;
                        offset += 2;
                        current_time = current_time.saturating_add(samples);
                    }
                }
                0x62 => {
                    // Wait 735 samples
                    current_time = current_time.saturating_add(735);
                }
                0x63 => {
                    // Wait 882 samples
                    current_time = current_time.saturating_add(882);
                }
                0x66 => {
                    // End of data
                    break;
                }
                _ => {
                    // Skip unknown commands
                }
            }
        }

        Ok(commands)
    }

    /// Play the loaded VGM file
    pub fn play(&mut self) -> MmlResult<()> {
        if self.header.is_none() {
            return Err(MmlError::Compilation(
                "No VGM file loaded. Call load() first.".to_string(),
            ));
        }

        self.state = PlayerState::Playing;
        self.current_sample = 0;
        self.current_command = 0;

        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) -> MmlResult<()> {
        self.state = PlayerState::Stopped;
        self.current_sample = 0;
        self.current_command = 0;

        if let Some(backend) = &mut self.audio_backend {
            backend.stop()?;
        }

        Ok(())
    }

    /// Pause playback
    pub fn pause(&mut self) -> MmlResult<()> {
        self.state = PlayerState::Paused;
        if let Some(backend) = &mut self.audio_backend {
            backend.stop()?;
        }
        Ok(())
    }

    /// Resume playback
    pub fn resume(&mut self) -> MmlResult<()> {
        self.state = PlayerState::Playing;
        if let Some(backend) = &mut self.audio_backend {
            backend.start()?;
        }
        Ok(())
    }

    /// Check if playback is active
    pub fn is_playing(&self) -> bool {
        self.state == PlayerState::Playing
    }

    /// Get current playback position in samples
    pub fn position(&self) -> u64 {
        self.current_sample
    }

    /// Get total duration in samples
    pub fn duration(&self) -> u32 {
        self.header.as_ref().map(|h| h.total_samples).unwrap_or(0)
    }

    /// Generate the next batch of samples
    pub fn generate_samples(&mut self, buffer: &mut [f32], sample_count: usize) -> MmlResult<()> {
        if self.state != PlayerState::Playing {
            buffer.fill(0.0);
            return Ok(());
        }

        // Process commands that should occur during this sample batch
        let end_sample = self.current_sample + sample_count as u64;

        while self.current_command < self.commands.len() {
            let cmd_time = self.commands[self.current_command].0;
            if cmd_time as u64 >= end_sample {
                break;
            }

            // Process this command
            let cmd_data = self.commands[self.current_command].1.clone();
            self._execute_command(&cmd_data)?;
            self.current_command += 1;
        }

        // Generate samples from all chips
        for (_, chip) in &mut self.chips {
            chip.generate_samples(buffer, self.sample_rate);
        }

        self.current_sample = end_sample;

        Ok(())
    }

    fn _execute_command(&mut self, _cmd_data: &[u8]) -> MmlResult<()> {
        // Command execution will be implemented with full chip support
        Ok(())
    }

    /// Set the audio backend for playback
    pub fn set_audio_backend(&mut self, backend: Box<dyn AudioBackend>) {
        self.audio_backend = Some(backend);
    }
}

impl Default for VgmPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vgm_player_creation() {
        let player = VgmPlayer::new();
        assert!(!player.is_playing());
        assert_eq!(player.position(), 0);
        assert_eq!(player.state, PlayerState::Stopped);
    }

    #[test]
    fn test_vgm_player_defaults() {
        let player = VgmPlayer::default();
        assert!(!player.is_playing());
    }
}
