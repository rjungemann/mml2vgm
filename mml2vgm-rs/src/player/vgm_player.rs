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
        if header.version >= 0x151 && data.len() >= 0x3C {
            header.segapcm_clock = u32::from_le_bytes([data[0x38], data[0x39], data[0x3A], data[0x3B]]);
        }
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
                0x51 | 0x52 | 0x53 => {
                    if offset + 2 <= data.len() {
                        let a = data[offset]; let v = data[offset+1]; offset += 2;
                        commands.push((current_time, vec![cmd, a, v]));
                    }
                }
                0x54..=0x5F => {
                    if offset + 2 <= data.len() {
                        let a = data[offset]; let v = data[offset+1]; offset += 2;
                        commands.push((current_time, vec![cmd, a, v]));
                    }
                }
                // AY8910 and 0xB0-0xBF range (2-byte writes: addr, data)
                0xA0 | 0xB0..=0xBF => {
                    if offset + 2 <= data.len() {
                        let a = data[offset]; let v = data[offset+1]; offset += 2;
                        commands.push((current_time, vec![cmd, a, v]));
                    }
                }
                // 0xD0-0xD6 range (3-byte writes: port/chip, addr, data)
                0xD0..=0xD6 => {
                    if offset + 3 <= data.len() {
                        let p = data[offset]; let a = data[offset+1]; let v = data[offset+2]; offset += 3;
                        commands.push((current_time, vec![cmd, p, a, v]));
                    }
                }
                // C352 write (3-byte: addr_hi, addr_lo, data)
                0xE1 => {
                    if offset + 3 <= data.len() {
                        let ah = data[offset]; let al = data[offset+1]; let v = data[offset+2]; offset += 3;
                        commands.push((current_time, vec![cmd, ah, al, v]));
                    }
                }
                // PCM seek (4-byte u32 offset, no chip effect — just advance)
                0xE0 => {
                    if offset + 4 <= data.len() { offset += 4; }
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
                // QSound write (3-byte: data_hi, addr, data_lo)
                0xC4 => {
                    if offset + 3 <= data.len() {
                        let dh = data[offset]; let a = data[offset+1]; let dl = data[offset+2]; offset += 3;
                        commands.push((current_time, vec![cmd, dh, a, dl]));
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
                0x50 | 0x51 | 0x52 | 0x53 | 0x54..=0x5F | 0xA0 | 0xB0..=0xBF => { offset += 2; }
                0xC0 | 0xC1 | 0xC4 | 0xD0..=0xD6 | 0xE1 => { offset += 3; }
                0xE0 => { if offset + 4 <= data.len() { offset += 4; } }
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
                        emu.write(cmd_data[1], 0); break;
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
            0x51 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2413 { emu.write(cmd_data[1], cmd_data[2]); break; }
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
            0x58 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2610B {
                        emu.write_port(0, cmd_data[1], cmd_data[2]); break;
                    }
                }
            }
            0x59 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM2610B {
                        emu.write_port(1, cmd_data[1], cmd_data[2]); break;
                    }
                }
            }
            0x5A if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM3812 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x5B if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YM3526 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x5C if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::Y8950 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x5E if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YMF262 { emu.write_port(0, cmd_data[1], cmd_data[2]); break; }
                }
            }
            0x5F if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::YMF262 { emu.write_port(1, cmd_data[1], cmd_data[2]); break; }
                }
            }
            // AY8910: 0xA0 aa dd — top bit of aa selects chip instance (we ignore, route to AY8910)
            0xA0 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::AY8910 {
                        emu.write_port(cmd_data[1] >> 7, cmd_data[1] & 0x7F, cmd_data[2]); break;
                    }
                }
            }
            // HuC6280: 0xB9 aa dd
            0xB9 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::HuC6280 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            // NES APU: 0xB4 aa dd
            0xB4 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::NES { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            // DMG (Game Boy APU): 0xB3 aa dd
            0xB3 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::DMG { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            // POKEY: 0xBB aa dd
            0xBB if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::POKEY { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            // VRC6: 0xB6 aa dd
            0xB6 if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::VRC6 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            // K053260: 0xBA aa dd
            0xBA if cmd_data.len() >= 3 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::K053260 { emu.write(cmd_data[1], cmd_data[2]); break; }
                }
            }
            // K054539: 0xD3 pp aa dd
            0xD3 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::K054539 {
                        emu.write_port(cmd_data[1], cmd_data[2], cmd_data[3]); break;
                    }
                }
            }
            // K051649 (SCC): 0xD2 pp aa dd
            0xD2 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::K051649 {
                        emu.write_port(cmd_data[1], cmd_data[2], cmd_data[3]); break;
                    }
                }
            }
            0xD4 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::C140 { emu.write_port(cmd_data[1], cmd_data[2], cmd_data[3]); break; }
                }
            }
            0xE1 if cmd_data.len() >= 4 => {
                for (chip, emu) in &mut self.chips {
                    if *chip == SoundChip::C352 { emu.write_port(cmd_data[1], cmd_data[2], cmd_data[3]); break; }
                }
            }
            0xC0 if cmd_data.len() >= 4 => {
                // addr_hi = cmd_data[2], addr_lo = cmd_data[1] (little-endian 16-bit offset)
                let has_segapcm = self.header.as_ref().map_or(false, |h| h.segapcm_clock > 0);
                if has_segapcm {
                    for (chip, emu) in &mut self.chips {
                        if *chip == SoundChip::SegaPCM {
                            emu.write_port(cmd_data[2], cmd_data[1], cmd_data[3]); break;
                        }
                    }
                } else {
                    for (chip, emu) in &mut self.chips {
                        if *chip == SoundChip::RF5C164 {
                            emu.write_port(cmd_data[2], cmd_data[1], cmd_data[3]); break;
                        }
                    }
                }
            }
            0xC1 if cmd_data.len() >= 4 => {
                // chip 2: second SegaPCM or RF5C164 (rare); same address encoding
                let has_segapcm = self.header.as_ref().map_or(false, |h| h.segapcm_clock > 0);
                if has_segapcm {
                    for (chip, emu) in &mut self.chips {
                        if *chip == SoundChip::SegaPCM {
                            emu.write_port(cmd_data[2], cmd_data[1], cmd_data[3]); break;
                        }
                    }
                } else {
                    for (chip, emu) in &mut self.chips {
                        if *chip == SoundChip::RF5C164 {
                            emu.write_port(cmd_data[2], cmd_data[1], cmd_data[3]); break;
                        }
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
        let mut has_ym2610b = false;
        let mut has_ym3812 = false;
        let mut has_ym3526 = false;
        let mut has_y8950 = false;
        let mut has_ymf262 = false;
        let mut has_c140 = false;
        let mut has_c352 = false;
        let mut has_c0_opcode = false;
        let mut has_ym2413 = false;
        let mut has_ay8910 = false;
        let mut has_huc6280 = false;
        let mut has_nes = false;
        let mut has_dmg = false;
        let mut has_pokey = false;
        let mut has_k051649 = false;
        let mut has_vrc6 = false;
        let mut has_k053260 = false;
        let mut has_k054539 = false;

        for (_, cmd) in &self.commands {
            match cmd.first().copied() {
                Some(0x50) => has_sn76489 = true,
                Some(0x51) => has_ym2413 = true,
                Some(0x52 | 0x53) => has_ym2612 = true,
                Some(0x54) => has_ym2151 = true,
                Some(0x55) => has_ym2203 = true,
                Some(0x56 | 0x57) => has_ym2608 = true,
                Some(0x58 | 0x59) => has_ym2610b = true,
                Some(0x5A) => has_ym3812 = true,
                Some(0x5B) => has_ym3526 = true,
                Some(0x5C) => has_y8950 = true,
                Some(0x5E | 0x5F) => has_ymf262 = true,
                Some(0xA0) => has_ay8910 = true,
                Some(0xB3) => has_dmg = true,
                Some(0xB4) => has_nes = true,
                Some(0xB9) => has_huc6280 = true,
                Some(0xBB) => has_pokey = true,
                Some(0xC0 | 0xC1) => has_c0_opcode = true,
                Some(0xB6) => has_vrc6 = true,
                Some(0xBA) => has_k053260 = true,
                Some(0xD2) => has_k051649 = true,
                Some(0xD3) => has_k054539 = true,
                Some(0xD4) => has_c140 = true,
                Some(0xE1) => has_c352 = true,
                _ => {}
            }
        }

        // 0xC0/0xC1 routes to SegaPCM when the header declares a SegaPCM clock,
        // otherwise it is an RF5C164 write.
        let has_segapcm_clock = self.header.as_ref().map_or(false, |h| h.segapcm_clock > 0);

        if has_sn76489 { self.chips.push((SoundChip::SN76489, Box::new(crate::chips::sn76489::SN76489::new()))); }
        if has_ym2612 { self.chips.push((SoundChip::YM2612, Box::new(crate::chips::ym2612::YM2612::new()))); }
        if has_ym2151 { self.chips.push((SoundChip::YM2151, Box::new(crate::chips::ym2151::YM2151::new()))); }
        if has_ym2203 { self.chips.push((SoundChip::YM2203, Box::new(crate::chips::ym2203::YM2203::new()))); }
        if has_ym2608 { self.chips.push((SoundChip::YM2608, Box::new(crate::chips::ym2608::YM2608::new()))); }
        if has_ym2610b { self.chips.push((SoundChip::YM2610B, Box::new(crate::chips::ym2608::YM2608::new()))); }
        if has_ym3812 { self.chips.push((SoundChip::YM3812, Box::new(crate::chips::ym3812::YM3812::new()))); }
        if has_ym3526 { self.chips.push((SoundChip::YM3526, Box::new(crate::chips::ym3526::YM3526::new()))); }
        if has_y8950 { self.chips.push((SoundChip::Y8950, Box::new(crate::chips::y8950::Y8950::new()))); }
        if has_ymf262 { self.chips.push((SoundChip::YMF262, Box::new(crate::chips::ymf262::YMF262::new()))); }
        if has_c140 { self.chips.push((SoundChip::C140, Box::new(crate::chips::c140::C140::new()))); }
        if has_c352 { self.chips.push((SoundChip::C352, Box::new(crate::chips::c352::C352::new()))); }
        if has_ym2413 { self.chips.push((SoundChip::YM2413, Box::new(crate::chips::ym2413::YM2413::new()))); }
        if has_ay8910 { self.chips.push((SoundChip::AY8910, Box::new(crate::chips::ay8910::AY8910::new()))); }
        if has_huc6280 { self.chips.push((SoundChip::HuC6280, Box::new(crate::chips::huc6280::HuC6280::new()))); }
        if has_nes { self.chips.push((SoundChip::NES, Box::new(crate::chips::nes_apu::NesApu::new()))); }
        if has_dmg { self.chips.push((SoundChip::DMG, Box::new(crate::chips::dmg::Dmg::new()))); }
        if has_pokey { self.chips.push((SoundChip::POKEY, Box::new(crate::chips::pokey::Pokey::new()))); }
        if has_k051649 { self.chips.push((SoundChip::K051649, Box::new(crate::chips::k051649::K051649::new()))); }
        if has_vrc6 { self.chips.push((SoundChip::VRC6, Box::new(crate::chips::vrc6::VRC6::new()))); }
        if has_k053260 { self.chips.push((SoundChip::K053260, Box::new(crate::chips::k053260::K053260::new()))); }
        if has_k054539 { self.chips.push((SoundChip::K054539, Box::new(crate::chips::k054539::K054539::new()))); }
        if has_c0_opcode {
            if has_segapcm_clock {
                self.chips.push((SoundChip::SegaPCM, Box::new(crate::chips::segapcm::SegaPCM::new())));
            } else {
                self.chips.push((SoundChip::RF5C164, Box::new(crate::chips::rf5c164::RF5C164::new())));
            }
        }

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

    fn minimal_vgm(extra_commands: &[u8]) -> Vec<u8> {
        let mut data = vec![0u8; 0x80];
        data[0..4].copy_from_slice(b"Vgm ");
        let eof = (data.len() as u32 - 4 + extra_commands.len() as u32 + 1).to_le_bytes();
        data[4..8].copy_from_slice(&eof);
        data[8..12].copy_from_slice(&0x171u32.to_le_bytes()); // version 1.71
        data[36..40].copy_from_slice(&44100u32.to_le_bytes()); // rate
        data.extend_from_slice(extra_commands);
        data.push(0x66); // EOF command
        data
    }

    #[test]
    fn test_segapcm_opcode_detection() {
        // 0x58 aa dd: YM2610 port 0 write — must NOT be detected as SegaPCM
        let data = minimal_vgm(&[0x58, 0x30, 0xFF]);
        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();

        assert!(player.chips.iter().any(|(c, _)| *c == SoundChip::YM2610B),
            "0x58 opcode should select YM2610B chip");
        assert!(!player.chips.iter().any(|(c, _)| *c == SoundChip::SegaPCM),
            "0x58 opcode must not select SegaPCM");
    }

    #[test]
    fn test_c0_dispatches_to_segapcm_when_clock_set() {
        // 0xC0 with segapcm_clock > 0 in header → SegaPCM, not RF5C164
        let mut data = minimal_vgm(&[0xC0, 0x00, 0x10, 0xFF]);
        // Write segapcm_clock at header offset 0x38
        let clock = 4000000u32.to_le_bytes();
        data[0x38..0x3C].copy_from_slice(&clock);

        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();

        assert!(player.chips.iter().any(|(c, _)| *c == SoundChip::SegaPCM),
            "0xC0 with segapcm_clock should select SegaPCM");
        assert!(!player.chips.iter().any(|(c, _)| *c == SoundChip::RF5C164),
            "0xC0 with segapcm_clock must not select RF5C164");
    }

    #[test]
    fn test_c0_dispatches_to_rf5c164_when_no_clock() {
        // 0xC0 with segapcm_clock == 0 → RF5C164
        let data = minimal_vgm(&[0xC0, 0x00, 0x10, 0xFF]);

        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();

        assert!(player.chips.iter().any(|(c, _)| *c == SoundChip::RF5C164),
            "0xC0 without segapcm_clock should select RF5C164");
        assert!(!player.chips.iter().any(|(c, _)| *c == SoundChip::SegaPCM),
            "0xC0 without segapcm_clock must not select SegaPCM");
    }

    #[test]
    fn test_ym2413_opcode_detection() {
        let data = minimal_vgm(&[0x51, 0x30, 0x07]);
        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();
        assert!(player.chips.iter().any(|(c, _)| *c == SoundChip::YM2413),
            "0x51 should select YM2413");
    }

    #[test]
    fn test_opl_opcode_detection() {
        let cases: &[(&[u8], SoundChip)] = &[
            (&[0x5A, 0x20, 0x01], SoundChip::YM3812),
            (&[0x5B, 0x20, 0x01], SoundChip::YM3526),
            (&[0x5C, 0x20, 0x01], SoundChip::Y8950),
            (&[0x5E, 0x20, 0x01], SoundChip::YMF262),
            (&[0x5F, 0x20, 0x01], SoundChip::YMF262),
        ];
        for (cmd, expected_chip) in cases {
            let data = minimal_vgm(cmd);
            let mut player = VgmPlayer::new();
            player.load(&data).unwrap();
            player.init_chips_from_header();
            assert!(
                player.chips.iter().any(|(c, _)| c == expected_chip),
                "opcode 0x{:02X} should select {:?}", cmd[0], expected_chip
            );
        }
    }

    #[test]
    fn test_c140_opcode_detection() {
        let data = minimal_vgm(&[0xD4, 0x00, 0x10, 0x40]);
        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();
        assert!(player.chips.iter().any(|(c, _)| *c == SoundChip::C140),
            "0xD4 should select C140");
    }

    #[test]
    fn test_c352_opcode_detection() {
        let data = minimal_vgm(&[0xE1, 0x00, 0x10, 0x40]);
        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();
        assert!(player.chips.iter().any(|(c, _)| *c == SoundChip::C352),
            "0xE1 should select C352");
    }

    #[test]
    fn test_ymf262_port1_writes_bank1_channel() {
        let data = minimal_vgm(&[0x5E, 0x10, 0x20, 0x5F, 0x10, 0x20]);
        let mut player = VgmPlayer::new();
        player.load(&data).unwrap();
        player.init_chips_from_header();
        player.play().unwrap();
        // Execute the two commands; if write_port is wired they won't panic.
        let mut buf = [0.0f32; 2];
        player.generate_samples(&mut buf, 2).unwrap();
    }

    #[test]
    fn test_unknown_opcodes_do_not_stall_parser() {
        // Mix of known and unknown opcodes — parser must not loop or panic
        let data = minimal_vgm(&[
            0xA0, 0x07, 0x3F,  // AY8910 write (2-byte)
            0xB4, 0x00, 0x0F,  // NES APU write (2-byte)
            0xD3, 0x00, 0x01, 0x80, // K054539 write (3-byte)
            0xE0, 0x00, 0x00, 0x00, 0x00, // PCM seek (4-byte, skip)
        ]);
        let mut player = VgmPlayer::new();
        player.load(&data).unwrap(); // must not panic or stall
    }
}
