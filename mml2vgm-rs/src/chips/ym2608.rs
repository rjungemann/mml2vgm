//! YM2608 (OPNA) sound chip emulation — 6 FM + 3 SSG + ADPCM-A/B

use super::SoundChipEmulator;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, Default)]
struct SsgChannel {
    frequency: u16,
    volume: u8,
    phase: u32,
}

#[derive(Debug, Clone, Copy)]
struct FmChannel {
    frequency: u16,
    octave: u8,
    output_level: u8,
    key_on: bool,
    output_phase: u32,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self { frequency: 0, octave: 0, output_level: 0, key_on: false, output_phase: 0 }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct AdpcmAChannel {
    key_on: bool,
    level: u8,
    pan_left: bool,
    pan_right: bool,
    position: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct DeltaT {
    active: bool,
    pan_left: bool,
    pan_right: bool,
    start_addr: u32,
    end_addr: u32,
    delta_n: u16,
    level: u8,
    position: u32,
    frac: u32,
    adpcm_step_index: i32,
    adpcm_predictor: i32,
    high_nibble: bool,
}

pub struct YM2608 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 0x400],
    fm_channels: [FmChannel; 6],
    ssg_channels: [SsgChannel; 3],
    adpcm_a_channels: [AdpcmAChannel; 6],
    adpcm_a_master_vol: u8,
    adpcm_a_rom: Vec<u8>,
    adpcm_b: DeltaT,
    adpcm_b_rom: Vec<u8>,
    accumulated_cycles: f32,
}

impl YM2608 {
    pub fn new() -> Self { Self::with_clock_rate(7_987_200) }

    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x400],
            fm_channels: [FmChannel::default(); 6],
            ssg_channels: [SsgChannel::default(); 3],
            adpcm_a_channels: [AdpcmAChannel::default(); 6],
            adpcm_a_master_vol: 63,
            adpcm_a_rom: Vec::new(),
            adpcm_b: DeltaT::default(),
            adpcm_b_rom: Vec::new(),
            accumulated_cycles: 0.0,
        }
    }

    fn get_fm_output(&self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for ch in &self.fm_channels {
            if ch.key_on && ch.frequency > 0 {
                let shift = 21u32.saturating_sub(ch.octave.min(7) as u32);
                let denom = 144.0 * (1u64 << shift) as f32;
                let freq_hz = (ch.frequency as f32 * self.clock_rate as f32) / denom;
                let phase = (ch.output_phase as f32 * freq_hz * 2.0 * PI) / self.clock_rate as f32;
                let sample = phase.sin() * (1.0 - ch.output_level as f32 / 127.0) * 0.15;
                left += sample;
                right += sample;
            }
        }
        (left, right)
    }

    fn get_ssg_output(&self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for ch in &self.ssg_channels {
            if ch.frequency > 0 && ch.volume > 0 {
                let half = ch.frequency as u32;
                let square = if (ch.phase / half.max(1)) % 2 == 0 { 1.0f32 } else { -1.0f32 };
                let sample = square * (ch.volume as f32 / 15.0) * 0.1;
                left += sample;
                right += sample;
            }
        }
        (left, right)
    }

    fn get_adpcm_a_output(&self) -> (f32, f32) {
        if self.adpcm_a_rom.is_empty() { return (0.0, 0.0); }
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        let master_vol = self.adpcm_a_master_vol as f32 / 63.0;
        for ch in &self.adpcm_a_channels {
            if !ch.key_on { continue; }
            let pos = ch.position as usize;
            if pos >= self.adpcm_a_rom.len() { continue; }
            let raw = self.adpcm_a_rom[pos] as i8;
            let sample = (raw as f32 / 128.0) * (ch.level as f32 / 31.0) * master_vol * 0.15;
            if ch.pan_left  { left  += sample; }
            if ch.pan_right { right += sample; }
        }
        (left, right)
    }

    const ADPCM_STEP_TABLE: [i32; 49] = [
        16, 17, 19, 21, 23, 25, 28, 31, 34, 37,
        41, 45, 50, 55, 60, 66, 73, 80, 88, 97,
        107, 118, 130, 143, 157, 173, 190, 209, 230, 253,
        279, 307, 337, 371, 408, 449, 494, 544, 598, 658,
        724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552,
    ];
    const ADPCM_INDEX_TABLE: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

    fn adpcm_b_decode_nibble(&mut self, nibble: u8) -> f32 {
        let step = Self::ADPCM_STEP_TABLE[self.adpcm_b.adpcm_step_index as usize];
        let mut delta = step >> 3;
        if (nibble & 4) != 0 { delta += step; }
        if (nibble & 2) != 0 { delta += step >> 1; }
        if (nibble & 1) != 0 { delta += step >> 2; }
        if (nibble & 8) != 0 {
            self.adpcm_b.adpcm_predictor = (self.adpcm_b.adpcm_predictor - delta).clamp(-32768, 32767);
        } else {
            self.adpcm_b.adpcm_predictor = (self.adpcm_b.adpcm_predictor + delta).clamp(-32768, 32767);
        }
        let idx_delta = Self::ADPCM_INDEX_TABLE[(nibble & 7) as usize];
        self.adpcm_b.adpcm_step_index = (self.adpcm_b.adpcm_step_index + idx_delta).clamp(0, 48);
        self.adpcm_b.adpcm_predictor as f32 / 32768.0
    }

    fn get_adpcm_b_sample(&mut self) -> f32 {
        if !self.adpcm_b.active || self.adpcm_b_rom.is_empty() { return 0.0; }
        let step = if self.adpcm_b.delta_n == 0 { 0x100u32 } else { self.adpcm_b.delta_n as u32 };
        self.adpcm_b.frac += step;
        let mut sample = 0.0f32;
        while self.adpcm_b.frac >= 0x100 {
            self.adpcm_b.frac -= 0x100;
            let end = (self.adpcm_b.end_addr * 32).min(self.adpcm_b_rom.len() as u32);
            if self.adpcm_b.position >= end {
                self.adpcm_b.active = false;
                return 0.0;
            }
            let byte_pos = self.adpcm_b.position as usize;
            if byte_pos >= self.adpcm_b_rom.len() {
                self.adpcm_b.active = false;
                return 0.0;
            }
            let byte = self.adpcm_b_rom[byte_pos];
            let nibble = if self.adpcm_b.high_nibble {
                self.adpcm_b.high_nibble = false;
                (byte >> 4) & 0x0F
            } else {
                self.adpcm_b.high_nibble = true;
                self.adpcm_b.position += 1;
                byte & 0x0F
            };
            sample = self.adpcm_b_decode_nibble(nibble);
        }
        sample * (self.adpcm_b.level as f32 / 255.0) * 0.5
    }

    fn apply_register(&mut self, port: u8, addr: u8, data: u8) {
        if port == 0 {
            match addr {
                0x28 => {
                    let ch_sel = (data & 0x03) as usize;
                    let port_bit = ((data >> 2) & 0x01) as usize;
                    let ch = ch_sel + port_bit * 3;
                    if ch < 6 { self.fm_channels[ch].key_on = (data >> 4) != 0; }
                }
                0xA0 | 0xA1 | 0xA2 => {
                    let ch = (addr - 0xA0) as usize;
                    let hi = self.regs[(0xA4 + (addr - 0xA0)) as usize];
                    let block = (hi >> 3) & 0x07;
                    let fnumber = ((hi & 0x07) as u16) << 8 | data as u16;
                    if ch < 6 { self.fm_channels[ch].frequency = fnumber; self.fm_channels[ch].octave = block; }
                }
                0xA4 | 0xA5 | 0xA6 => {
                    let ch = (addr - 0xA4) as usize;
                    let block = (data >> 3) & 0x07;
                    let lo = self.regs[(0xA0 + (addr - 0xA4)) as usize];
                    let fnumber = ((data & 0x07) as u16) << 8 | lo as u16;
                    if ch < 6 { self.fm_channels[ch].frequency = fnumber; self.fm_channels[ch].octave = block; }
                }
                0x40..=0x4E => {
                    let ch = (addr - 0x40) as usize % 3;
                    if ch < 6 { self.fm_channels[ch].output_level = data & 0x7F; }
                }
                0x00 | 0x02 | 0x04 => {
                    let ch = (addr / 2) as usize;
                    if ch < 3 { self.ssg_channels[ch].frequency = (self.ssg_channels[ch].frequency & 0xFF00) | data as u16; }
                }
                0x01 | 0x03 | 0x05 => {
                    let ch = (addr / 2) as usize;
                    if ch < 3 { self.ssg_channels[ch].frequency = (self.ssg_channels[ch].frequency & 0x00FF) | (((data & 0x0F) as u16) << 8); }
                }
                0x08 | 0x09 | 0x0A => {
                    let ch = (addr - 0x08) as usize;
                    if ch < 3 { self.ssg_channels[ch].volume = data & 0x1F; }
                }
                0x10 => {
                    let key_on = (data & 0x80) == 0;
                    for (i, ch) in self.adpcm_a_channels.iter_mut().enumerate() {
                        if (data >> i) & 1 == 1 {
                            ch.key_on = key_on;
                            if key_on { ch.position = 0; }
                        }
                    }
                }
                0x11 => { self.adpcm_a_master_vol = data & 0x3F; }
                0x18..=0x1D => {
                    let ch = (addr - 0x18) as usize;
                    if ch < 6 {
                        self.adpcm_a_channels[ch].level = data & 0x1F;
                        self.adpcm_a_channels[ch].pan_left  = (data & 0x80) != 0;
                        self.adpcm_a_channels[ch].pan_right = (data & 0x40) != 0;
                    }
                }
                _ => {}
            }
        } else {
            match addr {
                0x00 => {
                    if (data & 0x01) != 0 {
                        self.adpcm_b.active = true;
                        self.adpcm_b.position = self.adpcm_b.start_addr * 32;
                        self.adpcm_b.frac = 0;
                        self.adpcm_b.adpcm_step_index = 0;
                        self.adpcm_b.adpcm_predictor = 0;
                        self.adpcm_b.high_nibble = true;
                    } else if (data & 0x02) != 0 {
                        self.adpcm_b.active = false;
                    }
                }
                0x01 => { self.adpcm_b.pan_left = (data & 0x80) != 0; self.adpcm_b.pan_right = (data & 0x40) != 0; }
                0x02 => { self.adpcm_b.start_addr = (self.adpcm_b.start_addr & 0xFF00) | data as u32; }
                0x03 => { self.adpcm_b.start_addr = (self.adpcm_b.start_addr & 0x00FF) | ((data as u32) << 8); }
                0x04 => { self.adpcm_b.end_addr = (self.adpcm_b.end_addr & 0xFF00) | data as u32; }
                0x05 => { self.adpcm_b.end_addr = (self.adpcm_b.end_addr & 0x00FF) | ((data as u32) << 8); }
                0x08 => { self.adpcm_b.delta_n = (self.adpcm_b.delta_n & 0xFF00) | data as u16; }
                0x09 => { self.adpcm_b.delta_n = (self.adpcm_b.delta_n & 0x00FF) | ((data as u16) << 8); }
                0x0A => { self.adpcm_b.level = data; }
                0x28 => { self.apply_register(0, addr, data); }
                0xA0 | 0xA1 | 0xA2 => {
                    let ch = (addr - 0xA0) as usize + 3;
                    let hi_idx = 0x100 + (0xA4 + (addr - 0xA0)) as usize;
                    let hi = if hi_idx < self.regs.len() { self.regs[hi_idx] } else { 0 };
                    let block = (hi >> 3) & 0x07;
                    let fnumber = ((hi & 0x07) as u16) << 8 | data as u16;
                    if ch < 6 { self.fm_channels[ch].frequency = fnumber; self.fm_channels[ch].octave = block; }
                }
                0xA4 | 0xA5 | 0xA6 => {
                    let ch = (addr - 0xA4) as usize + 3;
                    let block = (data >> 3) & 0x07;
                    let lo_idx = 0x100 + (0xA0 + (addr - 0xA4)) as usize;
                    let lo = if lo_idx < self.regs.len() { self.regs[lo_idx] } else { 0 };
                    let fnumber = ((data & 0x07) as u16) << 8 | lo as u16;
                    if ch < 6 { self.fm_channels[ch].frequency = fnumber; self.fm_channels[ch].octave = block; }
                }
                _ => {}
            }
        }
    }
}

impl SoundChipEmulator for YM2608 {
    fn name(&self) -> &'static str { "YM2608 (OPNA)" }
    fn clock_rate(&self) -> u32 { self.clock_rate }
    fn reset(&mut self) { *self = Self::with_clock_rate(self.clock_rate); }

    fn write(&mut self, addr: u8, data: u8) {
        if (addr as usize) < 0x200 { self.regs[addr as usize] = data; }
        self.apply_register(0, addr, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let base = if port == 0 { 0usize } else { 0x100usize };
        let idx = base + addr as usize;
        if idx < self.regs.len() { self.regs[idx] = data; }
        self.apply_register(port, addr, data);
    }

    fn read(&self, _addr: u8) -> u8 { 0xFF }

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        match block_type {
            0x81 => { self.adpcm_b_rom = data.to_vec(); }
            0x82 => { self.adpcm_a_rom = data.to_vec(); }
            _ => {}
        }
    }

    fn clock(&mut self) {
        for ch in &mut self.fm_channels {
            if ch.key_on { ch.output_phase = ch.output_phase.wrapping_add(1); }
        }
        for ch in &mut self.ssg_channels {
            ch.phase = ch.phase.wrapping_add(1);
        }
        for ch in &mut self.adpcm_a_channels {
            if ch.key_on { ch.position = ch.position.wrapping_add(1); }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;
        for frame in buffer.chunks_mut(2) {
            let cycles_per_sample = self.clock_rate as f32 / sample_rate as f32;
            self.accumulated_cycles += cycles_per_sample;
            while self.accumulated_cycles >= 1.0 {
                self.clock();
                self.accumulated_cycles -= 1.0;
            }
            let (fm_left, fm_right) = self.get_fm_output();
            let (ssg_left, ssg_right) = self.get_ssg_output();
            let (adpcm_a_left, adpcm_a_right) = self.get_adpcm_a_output();
            let adpcm_b_sample = self.get_adpcm_b_sample();
            let adpcm_b_left  = if self.adpcm_b.pan_left  { adpcm_b_sample } else { 0.0 };
            let adpcm_b_right = if self.adpcm_b.pan_right { adpcm_b_sample } else { 0.0 };
            frame[0] = (fm_left + ssg_left + adpcm_a_left + adpcm_b_left).clamp(-1.0, 1.0);
            frame[1] = (fm_right + ssg_right + adpcm_a_right + adpcm_b_right).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YM2608 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2608_new() {
        let chip = YM2608::new();
        assert_eq!(chip.name(), "YM2608 (OPNA)");
        assert_eq!(chip.clock_rate(), 7_987_200);
        assert_eq!(chip.fm_channels.len(), 6);
        assert_eq!(chip.ssg_channels.len(), 3);
    }

    #[test]
    fn test_ym2608_write_fm() {
        let mut chip = YM2608::new();
        chip.write(0x28, 0x42);
        assert_eq!(chip.regs[0x28], 0x42);
    }

    #[test]
    fn test_ym2608_write_ssg() {
        let mut chip = YM2608::new();
        chip.write(0x0E, 0x10);
        chip.write(0x18, 0x0F);
        assert_eq!(chip.regs[0x0E], 0x10);
        assert_eq!(chip.regs[0x18], 0x0F);
    }

    #[test]
    fn test_ym2608_soundchip_trait() {
        let mut chip = YM2608::new();
        chip.reset();
        chip.write(0xA0, 0x0E);
        chip.write(0xA4, 0x24);
        chip.write(0x28, 0xF0);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert!(buffer[0].abs() > 0.0 || buffer[1].abs() > 0.0);
    }

    #[test]
    fn test_ym2608_adpcm_b_load() {
        let mut chip = YM2608::new();
        chip.load_pcm_data(0x81, &vec![0xAAu8; 256]);
        assert_eq!(chip.adpcm_b_rom.len(), 256);
    }

    #[test]
    fn test_ym2608_adpcm_a_load() {
        let mut chip = YM2608::new();
        chip.load_pcm_data(0x82, &vec![0x55u8; 512]);
        assert_eq!(chip.adpcm_a_rom.len(), 512);
    }
}
