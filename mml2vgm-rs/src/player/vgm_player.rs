//! VGM file player

use crate::audio::AudioBackend;
use crate::chips::SoundChipEmulator;
use crate::{MmlError, MmlResult, SoundChip, VgmHeader};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

pub struct VgmPlayer {
    header: Option<VgmHeader>,
    commands: Vec<(u32, Vec<u8>)>,
    data_blocks: Vec<(u8, Vec<u8>)>,
    chips: Vec<(SoundChip, Box<dyn SoundChipEmulator>)>,
    current_sample: u64,
    current_command: usize,
    sample_rate: u32,
    state: PlayerState,
    audio_backend: Option<Box<dyn AudioBackend>>,
}

impl VgmPlayer {
    pub fn new() -> Self {
        Self {
            header: None,
            commands: Vec::new(),
            data_blocks: Vec::new(),
            chips: Vec::new(),
            current_sample: 0,
            current_command: 0,
            sample_rate: 44100,
            state: PlayerState::Stopped,
            audio_backend: None,
        }
    }

    pub fn load(&mut self, data: &[u8]) -> MmlResult<()> {
        if data.len() < 64 {
            return Err(MmlError::InvalidVgmHeader("VGM file too small".to_string()));
        }
        let header = Self::parse_header(data)?;
        self.header = Some(header);
        self.sample_rate = header.rate;
        self.commands = Self::parse_commands(data, &header)?;
        self.data_blocks = Self::parse_data_blocks(data, &header);
        self.current_sample = 0;
        self.current_command = 0;
        Ok(())
    }

    fn parse_header(data: &[u8]) -> MmlResult<VgmHeader> {
        let mut _cursor = Cursor::new(data);
        let mut header = VgmHeader::new();
        if &data[0..4] != b"Vgm " {
            return Err(MmlError::InvalidVgmHeader("Invalid VGM identifier".to_string()));
        }
        let eof_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        if eof_offset + 4 > data.len() {
            return Err(MmlError::InvalidVgmHeader("VGM file truncated".to_string()));
        }
        header.version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        header.eof_offset = eof_offset as u32;
        header.sn76489_clock = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        header.ym2413_clock = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        header.gd3_offset = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        header.total_samples = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        header.loop_offset = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        header.loop_samples = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        header.rate = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        if header.rate == 0 { header.rate = 44100; }
        Ok(header)
    }

    fn parse_commands(data: &[u8], header: &VgmHeader) -> MmlResult<Vec<(u32, Vec<u8>)>> {
        let mut commands = Vec::new();
        let mut offset = 0x40;
        let mut current_time: u32 = 0;
        let data_end = if header.eof_offset > 0 { (header.eof_offset as usize) + 4 } else { data.len() };

        while offset < data_end && offset < data.len() {
            let cmd = data[offset];
            offset += 1;
            match cmd {
                0x50 => {
                    if offset + 1 <= data.len() {
                        let v = data[offset]; offset += 1;
                        commands.push((current_time, vec![cmd, v]));
                    }
                }
                0x52 | 0x53 => {
                    if offset + 2 <= data.len() {
                        let a = data[offset]; let v = data[offset+1]; offset += 2;
                        commands.push((current_time, vec![cmd, a, v]));
                    }
                }
                0x54..=0x57 => {
                    if offset + 2 <= data.len() {
                        let a = data[offset]; let v = data[offset+1]; offset += 2;
                        commands.push((current_time, vec![cmd, a, v]));
                    }
                }
                0x58 | 0x59 => {
                    if offset + 3 <= data.len() {
                        let a = data[offset]; let b = data[offset+1]; let v = data[offset+2]; offset += 3;
                        commands.push((current_time, vec![cmd, a, b, v]));
                    }
                }
                0x5A..=0x5F => {
                    if offset + 2 <= data.len() {
                        let a = data[offset]; let v = data[offset+1]; offset += 2;
                        commands.push((current_time, vec![cmd, a, v]));
                    }
                }
                0x61 => {
                    if offset + 2 <= data.len() {
                        let samples = u16::from_le_bytes([data[offset], data[offset+1]]) as u32;
                        offset += 2;
                        current_time = current_time.saturating_add(samples);
                    }
                }
                0x62 => { current_time = current_time.saturating_add(735); }
                0x63 => { current_time = current_time.saturating_add(882); }
                0x66 => break,
                0x67 => {
                    // Data block: skip compat(1) + type(1) + size(4) + data
                    if offset + 2 <= data.len() {
                        offset += 1; // compat byte 0x66
                        offset += 1; // block_type
                        if offset + 4 <= data.len() {
                            let size = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
                            offset += 4;
                            offset = offset.saturating_add(size).min(data.len());
                        }
                    }
                }
                0x70..=0x7F => { current_time = current_time.saturating_add((cmd & 0x0F) as u32 + 1); }
                0x80..=0x8F => { current_time = current_time.saturating_add((cmd & 0x0F) as u32); }
                0xC0 | 0xC1 => {
                    if offset + 3 <= data.len() {
                        let a = data[offset]; let b = data[offset+1]; let v = data[offset+2]; offset += 3;
                        commands.push((current_time, vec![cmd, a, b, v]));
                    }
                }
                _ => {}
            }
        }
        Ok(commands)
    }

    fn parse_data_blocks(data: &[u8], header: &VgmHeader) -> Vec<(u8, Vec<u8>)> {
        let mut blocks = Vec::new();
        let mut offset = 0x40;
        let data_end = if header.eof_offset > 0 { (header.eof_offset as usize) + 4 } else { data.len() };

        while offset < data_end && offset < data.len() {
            let cmd = data[offset];
            offset += 1;
            match cmd {
                0x50 | 0x52 | 0x53 | 0x54..=0x5F => { offset += 2; }
                0x58 | 0x59 | 0xC0 | 0xC1 => { offset += 3; }
                0x61 => { offset += 2; }
                0x62 | 0x63 => {}
                0x70..=0x7F | 0x80..=0x8F => {}
                0x66 => break,
                0x67 => {
                    if offset + 2 <= data.len() {
                        offset += 1; // compat byte
                        let block_type = data[offset]; offset += 1;
                        if offset + 4 <= data.len() {
                            let size = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
                            offset += 4;
                            let end = (offset + size).min(data.len());
                            blocks.push((block_type, data[offset..end].to_vec()));
                            offset = end;
                        }
                    }
                }
                _ => {}
            }
        }
        blocks
    }

    pub fn play(&mut self) -> MmlResult<()> {
        if self.header.is_none() {
            return Err(MmlError::Compilation("No VGM file loaded. Call load() first.".to_string()));
        }
        self.state = PlayerState::Playing;
        self.current_sample = 0;
        self.current_command = 0;
        for (block_type, data) in &self.data_blocks.clone() {
            for (_, chip) in &mut self.chips {
                chip.load_pcm_data(*block_type, data);
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> MmlResult<()> {
        self.state = PlayerState::Stopped;
        self.current_sample = 0;
        self.current_command = 0;
        if let Some(backend) = &mut self.audio_backend { backend.stop()?; }
        Ok(())
    }

    pub fn pause(&mut self) -> MmlResult<()> {
        self.state = PlayerState::Paused;
        if let Some(backend) = &mut self.audio_backend { backend.stop()?; }
        Ok(())
    }

    pub fn resume(&mut self) -> MmlResult<()> {
        self.state = PlayerState::Playing;
        if let Some(backend) = &mut self.audio_backend { backend.start()?; }
        Ok(())
    }

    pub fn is_playing(&self) -> bool { self.state == PlayerState::Playing }
    pub fn position(&self) -> u64 { self.current_sample }
    pub fn duration(&self) -> u32 { self.header.as_ref().map(|h| h.total_samples).unwrap_or(0) }
    pub fn state(&self) -> PlayerState { self.state }
    pub fn header(&self) -> Option<&VgmHeader> { self.header.as_ref() }

    pub fn generate_samples(&mut self, buffer: &mut [f32], sample_count: usize) -> MmlResult<()> {
        if self.state != PlayerState::Playing {
            buffer.fill(0.0);
            return Ok(());
        }
        let end_sample = self.current_sample + sample_count as u64;
        while self.current_command < self.commands.len() {
            let cmd_time = self.commands[self.current_command].0;
            if cmd_time as u64 >= end_sample { break; }
            let cmd_data = self.commands[self.current_command].1.clone();
            self._execute_command(&cmd_data)?;
            self.current_command += 1;
        }
        buffer.fill(0.0);
        for (_, chip) in &mut self.chips {
            chip.generate_samples(buffer, self.sample_rate);
        }
        self.current_sample = end_sample;
        Ok(())
    }

    fn _execute_command(&mut self, cmd_data: &[u8]) -> MmlResult<()> {
        if cmd_data.is_empty() { return Ok(()); }
        match cmd_data[0] {
            0x50 if cmd_data.len() >= 2 => {
                for (chip, emu) in &mut self.chips {
                    if matches!(chip, SoundChip::SN76489 | SoundChip::SN76489X2) {
                        emu.write(0, cmd_data[1]); break;
                    }
                }
            }
            0x52 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if matches!(chip, SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2) {
                        emu.write_port(0, cmd_data[1], cmd_data[2]); break;
                    }
                }
            }
            0x53 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if matches!(chip, SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2) {
                        emu.write_port(1, cmd_data[1], cmd_data[2]); break;
                    }
                }
            }
            0x54 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2151 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x55 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2203 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x56 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2608 { emu.write_port(0, cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x57 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2608 { emu.write_port(1, cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x58 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::SegaPCM {
                        let off = u16::from_le_bytes([cmd_data[1], cmd_data[2]]);
                        emu.write(off as u8, cmd_data[3]); break;
                    }
                }
            }
            0x59 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::SegaPCM {
                        let off = u16::from_le_bytes([cmd_data[1], cmd_data[2]]);
                        emu.write_port(1, off as u8, cmd_data[3]); break;
                    }
                }
            }
            0xC0 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::RF5C164 {
                        let off = u16::from_le_bytes([cmd_data[1], cmd_data[2]]);
                        emu.write(off as u8, cmd_data[3]); break;
                    }
                }
            }
            0xC1 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::RF5C164 {
                        let off = u16::from_le_bytes([cmd_data[1], cmd_data[2]]);
                        emu.write_port(1, off as u8, cmd_data[3]); break;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn init_chips_from_header(&mut self) {
        if !self.chips.is_empty() { return; }
        let mut has_sn76489 = false;
        let mut has_ym2612 = false;
        let mut has_ym2151 = false;
        let mut has_ym2203 = false;
        let mut has_ym2608 = false;
        let mut has_rf5c164 = false;
        let mut has_segapcm = false;

        for (_, cmd) in &self.commands {
            match cmd.first().copied() {
                Some(0x50) => has_sn76489 = true,
                Some(0x52 | 0x53) => has_ym2612 = true,
                Some(0x54) => has_ym2151 = true,
                Some(0x55) => has_ym2203 = true,
                Some(0x56 | 0x57) => has_ym2608 = true,
                Some(0xC0 | 0xC1) => has_rf5c164 = true,
                Some(0x58 | 0x59) => has_segapcm = true,
                _ => {}
            }
        }

        if has_sn76489 { self.chips.push((SoundChip::SN76489, Box::new(crate::chips::sn76489::SN76489::new()))); }
        if has_ym2612 { self.chips.push((SoundChip::YM2612, Box::new(crate::chips::ym2612::YM2612::new()))); }
        if has_ym2151 { self.chips.push((SoundChip::YM2151, Box::new(crate::chips::ym2151::YM2151::new()))); }
        if has_ym2203 { self.chips.push((SoundChip::YM2203, Box::new(crate::chips::ym2203::YM2203::new()))); }
        if has_ym2608 { self.chips.push((SoundChip::YM2608, Box::new(crate::chips::ym2608::YM2608::new()))); }
        if has_rf5c164 { self.chips.push((SoundChip::RF5C164, Box::new(crate::chips::rf5c164::RF5C164::new()))); }
        if has_segapcm { self.chips.push((SoundChip::SegaPCM, Box::new(crate::chips::segapcm::SegaPCM::new()))); }

        if self.chips.is_empty() {
            self.chips.push((SoundChip::YM2608, Box::new(crate::chips::ym2608::YM2608::new())));
        }
    }

    pub fn render_to_pcm(&mut self, sample_rate: u32) -> MmlResult<Vec<f32>> {
        self.sample_rate = sample_rate;
        self.play()?;
        let total = self.duration() as usize;
        if total == 0 { return Ok(Vec::new()); }
        let mut all_samples = Vec::with_capacity(total * 2);
        const CHUNK: usize = 1024;
        let mut buf = vec![0.0f32; CHUNK * 2];
        while (self.current_sample as usize) < total {
            let remaining = total - self.current_sample as usize;
            let n = CHUNK.min(remaining);
            self.generate_samples(&mut buf[..n * 2], n)?;
            all_samples.extend_from_slice(&buf[..n * 2]);
        }
        Ok(all_samples)
    }

    pub fn set_audio_backend(&mut self, backend: Box<dyn AudioBackend>) {
        self.audio_backend = Some(backend);
    }
}

impl Default for VgmPlayer {
    fn default() -> Self { Self::new() }
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
