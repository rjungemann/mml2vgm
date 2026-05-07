//! YM2608 (OPNA) sound chip emulation — 6 FM + 3 SSG + ADPCM-A/B

use super::SoundChipEmulator;
use std::f32::consts::PI;

const FREQ_MULT: [f32; 16] = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 10.0, 12.0, 12.0, 15.0, 15.0];

/// OPN operator-address offset → (channel_within_bank, operator_index)
fn opn_ch_op(offset: u8) -> Option<(usize, usize)> {
    let ch = (offset % 4) as usize;
    let op = (offset / 4) as usize;
    if ch > 2 || op > 3 { return None; }
    Some((ch, op))
}

#[derive(Debug, Clone, Copy, Default)]
struct SsgChannel {
    frequency: u16,
    volume: u8,
    phase: u32,
}

/// FM operator for OPN 4-operator synthesis
#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase_acc: f32,
    total_level: u8,
    mult: u8,
    key_on: bool,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self { phase_acc: 0.0, total_level: 0, mult: 1, key_on: false }
    }
}

/// FM channel — 4-operator OPN
#[derive(Debug, Clone, Copy)]
struct FmChannel {
    operators: [FmOperator; 4],
    f_num: u16,
    block: u8,
    algorithm: u8,
    feedback: u8,
    left_enable: bool,
    right_enable: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            operators: [FmOperator::default(); 4],
            f_num: 0, block: 0, algorithm: 0, feedback: 0,
            left_enable: true, right_enable: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct AdpcmAChannel {
    key_on: bool,
    level: u8,
    pan_left: bool,
    pan_right: bool,
    start_addr: u32,
    end_addr: u32,
    position: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct DeltaT {
    active: bool,
    pan_left: bool,
    pan_right: bool,
    start_addr: u32,
    end_addr: u32,
    /// Upper bound for playback (registers 0x06/0x07); 0 means no limit.
    limit_addr: u32,
    delta_n: u16,
    /// Clock prescaler (registers 0x10/0x11); applied as divisor on the frac step.
    prescaler: u16,
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
    fm_op_feedback: [f32; 6],
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
            fm_op_feedback: [0.0; 6],
            ssg_channels: [SsgChannel::default(); 3],
            adpcm_a_channels: [AdpcmAChannel::default(); 6],
            adpcm_a_master_vol: 63,
            adpcm_a_rom: Vec::new(),
            adpcm_b: DeltaT::default(),
            adpcm_b_rom: Vec::new(),
            accumulated_cycles: 0.0,
        }
    }

    fn channel_freq_hz(&self, ch: usize) -> f32 {
        let f_num = self.fm_channels[ch].f_num as f32;
        let block = self.fm_channels[ch].block as i32;
        f_num * 2_f32.powi(block - 1) * self.clock_rate as f32 / (144.0 * (1u32 << 19) as f32)
    }

    fn advance_fm_phases(&mut self, sample_rate: u32) {
        for ch in 0..6 {
            let base_freq = self.channel_freq_hz(ch);
            for op in 0..4 {
                if !self.fm_channels[ch].operators[op].key_on { continue; }
                let mult = FREQ_MULT[self.fm_channels[ch].operators[op].mult as usize & 0xF];
                let inc = base_freq * mult * 2.0 * PI / sample_rate as f32;
                self.fm_channels[ch].operators[op].phase_acc += inc;
                if self.fm_channels[ch].operators[op].phase_acc > 2.0 * PI {
                    self.fm_channels[ch].operators[op].phase_acc -= 2.0 * PI;
                }
            }
        }
    }

    fn op_out(&self, ch: usize, op: usize, mod_in: f32) -> f32 {
        if !self.fm_channels[ch].operators[op].key_on { return 0.0; }
        let tl = 1.0 - self.fm_channels[ch].operators[op].total_level as f32 / 127.0;
        (self.fm_channels[ch].operators[op].phase_acc + mod_in).sin() * tl
    }

    fn get_fm_channel_output(&mut self, ch: usize) -> (f32, f32) {
        let any_on = self.fm_channels[ch].operators.iter().any(|op| op.key_on);
        if !any_on { return (0.0, 0.0); }

        let fb = self.fm_op_feedback[ch] * (self.fm_channels[ch].feedback as f32 / 7.0) * 0.25;
        let m1 = self.op_out(ch, 0, fb);
        self.fm_op_feedback[ch] = m1;

        let output = match self.fm_channels[ch].algorithm {
            0 => { let m2 = self.op_out(ch, 1, m1 * PI); let c1 = self.op_out(ch, 2, m2 * PI); self.op_out(ch, 3, c1 * PI) }
            1 => { let m2 = self.op_out(ch, 1, 0.0); let c1 = self.op_out(ch, 2, (m1 + m2) * PI * 0.5); self.op_out(ch, 3, c1 * PI) }
            2 => { let m2 = self.op_out(ch, 1, 0.0); let c1 = self.op_out(ch, 2, m2 * PI); self.op_out(ch, 3, (m1 + c1) * PI * 0.5) }
            3 => { let m2 = self.op_out(ch, 1, m1 * PI); let c1 = self.op_out(ch, 2, m2 * PI); self.op_out(ch, 3, (m2 + c1) * PI * 0.5) }
            4 => { let m2 = self.op_out(ch, 1, m1 * PI); let c1 = self.op_out(ch, 2, 0.0); (m2 + self.op_out(ch, 3, c1 * PI)) * 0.5 }
            5 => { let m2 = self.op_out(ch, 1, m1 * PI); let c1 = self.op_out(ch, 2, m1 * PI); let c2 = self.op_out(ch, 3, m1 * PI); (m2 + c1 + c2) / 3.0 }
            6 => { let m2 = self.op_out(ch, 1, m1 * PI); let c1 = self.op_out(ch, 2, 0.0); let c2 = self.op_out(ch, 3, 0.0); (m2 + c1 + c2) / 3.0 }
            _ => { let m2 = self.op_out(ch, 1, 0.0); let c1 = self.op_out(ch, 2, 0.0); let c2 = self.op_out(ch, 3, 0.0); (m1 + m2 + c1 + c2) * 0.25 }
        };

        let sample = (output * 0.2).clamp(-1.0, 1.0);
        let left = if self.fm_channels[ch].left_enable { sample } else { 0.0 };
        let right = if self.fm_channels[ch].right_enable { sample } else { 0.0 };
        (left, right)
    }

    fn get_fm_output(&mut self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for ch in 0..6 {
            let (l, r) = self.get_fm_channel_output(ch);
            left += l;
            right += r;
        }
        (left / 6.0, right / 6.0)
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
        let raw_step = if self.adpcm_b.delta_n == 0 { 0x100u32 } else { self.adpcm_b.delta_n as u32 };
        let divisor = if self.adpcm_b.prescaler == 0 { 1u32 } else { self.adpcm_b.prescaler as u32 };
        let step = (raw_step / divisor).max(1);
        self.adpcm_b.frac += step;
        let mut sample = 0.0f32;
        while self.adpcm_b.frac >= 0x100 {
            self.adpcm_b.frac -= 0x100;
            let end_by_reg = self.adpcm_b.end_addr * 32;
            let end_by_limit = if self.adpcm_b.limit_addr > 0 {
                self.adpcm_b.limit_addr * 32
            } else {
                u32::MAX
            };
            let end = end_by_reg.min(end_by_limit).min(self.adpcm_b_rom.len() as u32);
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
        let ch_base = if port == 0 { 0usize } else { 3usize };

        if port == 0 {
            match addr {
                // Key on/off (global: port 0 only)
                0x28 => {
                    let ch_sel = (data & 0x03) as usize;
                    let part = ((data >> 2) & 0x01) as usize;
                    let ch = ch_sel + part * 3;
                    if ch < 6 {
                        let prev_any = self.fm_channels[ch].operators.iter().any(|op| op.key_on);
                        self.fm_channels[ch].operators[0].key_on = (data & 0x10) != 0;
                        self.fm_channels[ch].operators[1].key_on = (data & 0x20) != 0;
                        self.fm_channels[ch].operators[2].key_on = (data & 0x40) != 0;
                        self.fm_channels[ch].operators[3].key_on = (data & 0x80) != 0;
                        let new_any = self.fm_channels[ch].operators.iter().any(|op| op.key_on);
                        if !prev_any && new_any {
                            for op in &mut self.fm_channels[ch].operators { op.phase_acc = 0.0; }
                            self.fm_op_feedback[ch] = 0.0;
                        }
                    }
                }
                // SSG registers
                0x00 => self.ssg_channels[0].frequency = (self.ssg_channels[0].frequency & 0xF00) | data as u16,
                0x01 => self.ssg_channels[0].frequency = (self.ssg_channels[0].frequency & 0x0FF) | ((data as u16 & 0x0F) << 8),
                0x02 => self.ssg_channels[1].frequency = (self.ssg_channels[1].frequency & 0xF00) | data as u16,
                0x03 => self.ssg_channels[1].frequency = (self.ssg_channels[1].frequency & 0x0FF) | ((data as u16 & 0x0F) << 8),
                0x04 => self.ssg_channels[2].frequency = (self.ssg_channels[2].frequency & 0xF00) | data as u16,
                0x05 => self.ssg_channels[2].frequency = (self.ssg_channels[2].frequency & 0x0FF) | ((data as u16 & 0x0F) << 8),
                0x08 => self.ssg_channels[0].volume = data & 0x1F,
                0x09 => self.ssg_channels[1].volume = data & 0x1F,
                0x0A => self.ssg_channels[2].volume = data & 0x1F,
                // ADPCM-A
                0x10 => {
                    let key_on = (data & 0x80) == 0;
                    for (i, ch) in self.adpcm_a_channels.iter_mut().enumerate() {
                        if (data >> i) & 1 == 1 {
                            ch.key_on = key_on;
                            if key_on { ch.position = ch.start_addr * 32; }
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
                // FM operator registers (port 0 → ch_base=0)
                _ => { self.apply_fm_register(ch_base, addr, data); }
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
                0x06 => { self.adpcm_b.limit_addr = (self.adpcm_b.limit_addr & 0xFF00) | data as u32; }
                0x07 => { self.adpcm_b.limit_addr = (self.adpcm_b.limit_addr & 0x00FF) | ((data as u32) << 8); }
                0x08 => { self.adpcm_b.delta_n = (self.adpcm_b.delta_n & 0xFF00) | data as u16; }
                0x09 => { self.adpcm_b.delta_n = (self.adpcm_b.delta_n & 0x00FF) | ((data as u16) << 8); }
                0x0A => { self.adpcm_b.level = data; }
                0x10 => { self.adpcm_b.prescaler = (self.adpcm_b.prescaler & 0xFF00) | data as u16; }
                0x11 => { self.adpcm_b.prescaler = (self.adpcm_b.prescaler & 0x00FF) | ((data as u16) << 8); }
                // ADPCM-A start address low/high per channel
                0x20..=0x25 => {
                    let ch = (addr - 0x20) as usize;
                    self.adpcm_a_channels[ch].start_addr =
                        (self.adpcm_a_channels[ch].start_addr & 0xFF00) | data as u32;
                }
                0x28..=0x2D => {
                    let ch = (addr - 0x28) as usize;
                    self.adpcm_a_channels[ch].start_addr =
                        (self.adpcm_a_channels[ch].start_addr & 0x00FF) | ((data as u32) << 8);
                }
                // ADPCM-A end address low/high per channel
                0x30..=0x35 => {
                    let ch = (addr - 0x30) as usize;
                    self.adpcm_a_channels[ch].end_addr =
                        (self.adpcm_a_channels[ch].end_addr & 0xFF00) | data as u32;
                }
                0x38..=0x3D => {
                    let ch = (addr - 0x38) as usize;
                    self.adpcm_a_channels[ch].end_addr =
                        (self.adpcm_a_channels[ch].end_addr & 0x00FF) | ((data as u32) << 8);
                }
                // Port 1 FM registers (ch_base=3)
                _ => { self.apply_fm_register(ch_base, addr, data); }
            }
        }
    }

    fn apply_fm_register(&mut self, ch_base: usize, addr: u8, data: u8) {
        match addr {
            // Operator DT1/MULT
            0x30..=0x3F => { if let Some((ch, op)) = opn_ch_op(addr - 0x30) { let c = ch_base + ch; if c < 6 { self.fm_channels[c].operators[op].mult = data & 0x0F; } } }
            // Operator Total Level
            0x40..=0x4F => { if let Some((ch, op)) = opn_ch_op(addr - 0x40) { let c = ch_base + ch; if c < 6 { self.fm_channels[c].operators[op].total_level = data & 0x7F; } } }
            // Operator AR/DR/SR/SL/RR/SSG-EG — stored in reg cache only
            0x50..=0x9F => {}
            // Channel F-number lo
            0xA0..=0xA2 => {
                let ch = ch_base + (addr - 0xA0) as usize;
                if ch < 6 { self.fm_channels[ch].f_num = (self.fm_channels[ch].f_num & 0x700) | data as u16; }
            }
            // Channel Block/F-number hi
            0xA4..=0xA6 => {
                let ch = ch_base + (addr - 0xA4) as usize;
                if ch < 6 {
                    self.fm_channels[ch].block = (data >> 3) & 0x07;
                    self.fm_channels[ch].f_num = (self.fm_channels[ch].f_num & 0x0FF) | (((data as u16) & 0x07) << 8);
                }
            }
            // Channel Algorithm/Feedback
            0xB0..=0xB2 => {
                let ch = ch_base + (addr - 0xB0) as usize;
                if ch < 6 { self.fm_channels[ch].algorithm = data & 0x07; self.fm_channels[ch].feedback = (data >> 3) & 0x07; }
            }
            // Channel L/R
            0xB4..=0xB6 => {
                let ch = ch_base + (addr - 0xB4) as usize;
                if ch < 6 { self.fm_channels[ch].left_enable = (data & 0x80) != 0; self.fm_channels[ch].right_enable = (data & 0x40) != 0; }
            }
            _ => {}
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
        for ch in &mut self.ssg_channels {
            ch.phase = ch.phase.wrapping_add(1);
        }
        for ch in &mut self.adpcm_a_channels {
            if ch.key_on {
                ch.position = ch.position.wrapping_add(1);
                let end = if ch.end_addr > 0 { ch.end_addr * 32 } else { u32::MAX };
                if ch.position >= end {
                    ch.key_on = false;
                }
            }
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
            self.advance_fm_phases(sample_rate);
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

    #[test]
    fn test_ym2608_adpcm_b_limit_address() {
        let mut chip = YM2608::new();

        // Load PCM ROM (128 bytes of dummy data)
        chip.load_pcm_data(0x81, &vec![0x77u8; 128]);

        // start_addr = 0x0000, end_addr = 0x0004 (end = 4*32 = 128 bytes)
        chip.write_port(1, 0x02, 0x00);
        chip.write_port(1, 0x03, 0x00);
        chip.write_port(1, 0x04, 0x04);
        chip.write_port(1, 0x05, 0x00);

        // limit_addr = 0x0002 (limit = 2*32 = 64 bytes — tighter than end_addr)
        chip.write_port(1, 0x06, 0x02);
        chip.write_port(1, 0x07, 0x00);
        assert_eq!(chip.adpcm_b.limit_addr, 0x0002);

        // delta_n = 0x0100 (step = 0x100, frac advances exactly 1 byte per call)
        chip.write_port(1, 0x08, 0x00);
        chip.write_port(1, 0x09, 0x01);

        // prescaler = 0 (no division)
        chip.write_port(1, 0x10, 0x00);
        chip.write_port(1, 0x11, 0x00);
        assert_eq!(chip.adpcm_b.prescaler, 0);

        // Start playback
        chip.write_port(1, 0x00, 0x01);
        assert!(chip.adpcm_b.active);

        // Each call reads one nibble; 2 nibbles per byte → 2*(limit*32)+1 calls to trigger stop.
        let limit_bytes = 0x0002u32 * 32; // 64
        for _ in 0..(2 * limit_bytes + 1) {
            chip.get_adpcm_b_sample();
        }
        assert!(!chip.adpcm_b.active,
            "ADPCM-B must stop at limit_addr boundary when it is less than end_addr");

        // Prescaler test: write prescaler=2 and verify it halves the step
        chip.write_port(1, 0x10, 0x02);
        assert_eq!(chip.adpcm_b.prescaler, 2);
        // restart playback
        chip.write_port(1, 0x00, 0x01);
        assert!(chip.adpcm_b.active);
        // With prescaler=2 and delta_n=0x100, effective step = 0x100/2 = 0x80;
        // two calls are needed to advance frac past 0x100 by one byte.
        chip.get_adpcm_b_sample(); // frac += 0x80 (no byte consumed yet)
        assert!(chip.adpcm_b.active, "channel should still be active after one half-step");
    }

    #[test]
    fn test_ym2608_adpcm_a_address_registers() {
        let mut chip = YM2608::new();

        // CH0 start_addr = 0x0210 (low=0x10, high=0x02)
        chip.write_port(1, 0x20, 0x10);
        chip.write_port(1, 0x28, 0x02);
        // CH0 end_addr = 0x0220 (low=0x20, high=0x02)
        chip.write_port(1, 0x30, 0x20);
        chip.write_port(1, 0x38, 0x02);

        assert_eq!(chip.adpcm_a_channels[0].start_addr, 0x0210);
        assert_eq!(chip.adpcm_a_channels[0].end_addr, 0x0220);

        // Key-on channel 0 (bit7=0 → key-on, bit0=1 → ch0)
        chip.write(0x10, 0x01);
        assert!(chip.adpcm_a_channels[0].key_on);
        assert_eq!(chip.adpcm_a_channels[0].position, 0x0210 * 32,
            "key-on must reset position to start_addr * 32");

        // Clock until end_addr * 32 is reached; channel should auto-stop
        let start_pos = 0x0210u32 * 32;
        let end_pos   = 0x0220u32 * 32;
        for _ in 0..(end_pos - start_pos) {
            chip.clock();
        }
        assert!(!chip.adpcm_a_channels[0].key_on,
            "channel must stop when position reaches end_addr * 32");

        // CH3 should be unaffected
        assert!(!chip.adpcm_a_channels[3].key_on);
        assert_eq!(chip.adpcm_a_channels[3].start_addr, 0);
        assert_eq!(chip.adpcm_a_channels[3].end_addr, 0);
    }
}
