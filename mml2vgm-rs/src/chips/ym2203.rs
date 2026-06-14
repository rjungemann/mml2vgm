//! YM2203 (OPN) sound chip emulation
//!
//! The YM2203 provides 3 FM channels (4 operators each) and 3 SSG channels.
//!
//! OPN register map:
//! - 0x00-0x0F: SSG registers (AY8910-compatible)
//!   - 0x00-0x05: Tone period lo/hi per channel
//!   - 0x06: Noise period
//!   - 0x07: Mixer (tone/noise enable)
//!   - 0x08-0x0A: Channel volume (0-15)
//! - 0x28: Key on/off: bits[1:0]=ch, bits[7:4]=operator select
//! - 0x30-0x3F: Operator DT/MULT — ch=bits[1:0], op=bits[3:2]
//! - 0x40-0x4F: Operator Total Level
//! - 0x50-0x5F: Operator KS/AR
//! - 0x60-0x6F: Operator AM/DR
//! - 0x70-0x7F: Operator SR
//! - 0x80-0x8F: Operator SL/RR
//! - 0xA0-0xA2: Channel F-number lo (ch 0-2)
//! - 0xA4-0xA6: Channel Block/F-number hi (ch 0-2)
//! - 0xB0-0xB2: Channel Algorithm/Feedback
//! - 0xB4-0xB6: Channel L/R/AM sensitivity

use super::SoundChipEmulator;
use std::f32::consts::PI;

const FREQ_MULT: [f32; 16] = [
    0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 10.0, 12.0, 12.0, 15.0, 15.0,
];

/// OPN operator address offset → (channel, operator_index)
/// Layout: ch = offset % 4 (0-2 valid, 3 unused), op = offset / 4
fn opn_ch_op(offset: u8) -> Option<(usize, usize)> {
    let ch = (offset % 4) as usize;
    let op = (offset / 4) as usize;
    if ch > 2 || op > 3 {
        return None;
    }
    Some((ch, op))
}

/// FM operator
#[derive(Debug, Clone, Copy)]
struct FmOperator {
    phase_acc: f32,
    total_level: u8,
    mult: u8,
    key_on: bool,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self {
            phase_acc: 0.0,
            total_level: 0,
            mult: 1,
            key_on: false,
        }
    }
}

/// FM channel (OPN 4-operator)
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
            f_num: 0,
            block: 0,
            algorithm: 0,
            feedback: 0,
            left_enable: true,
            right_enable: true,
        }
    }
}

/// SSG channel (AY8910-compatible square wave)
#[derive(Debug, Clone, Copy, Default)]
struct SsgChannel {
    period: u16,
    volume: u8,
    #[allow(dead_code)]
    tone_enable: bool,
    phase_acc: f32,
}

/// YM2203 chip emulator
pub struct YM2203 {
    clock_rate: u32,
    sample_rate: u32,
    regs: [u8; 0x100],
    fm_channels: [FmChannel; 3],
    ssg_channels: [SsgChannel; 3],
    ssg_mixer: u8,
    #[allow(dead_code)]
    accumulated_cycles: f32,
    op_feedback: [f32; 3],
}

impl YM2203 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(3_993_600)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            regs: [0; 0x100],
            fm_channels: [FmChannel::default(); 3],
            ssg_channels: [SsgChannel::default(); 3],
            ssg_mixer: 0xFF,
            accumulated_cycles: 0.0,
            op_feedback: [0.0; 3],
        }
    }

    fn channel_freq_hz(&self, ch: usize) -> f32 {
        let f_num = self.fm_channels[ch].f_num as f32;
        let block = self.fm_channels[ch].block as i32;
        f_num * 2_f32.powi(block - 1) * self.clock_rate as f32 / (144.0 * (1u32 << 19) as f32)
    }

    fn advance_fm_phases(&mut self, sample_rate: u32) {
        for ch in 0..3 {
            let base_freq = self.channel_freq_hz(ch);
            for op in 0..4 {
                if !self.fm_channels[ch].operators[op].key_on {
                    continue;
                }
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
        if !self.fm_channels[ch].operators[op].key_on {
            return 0.0;
        }
        let tl = 1.0 - self.fm_channels[ch].operators[op].total_level as f32 / 127.0;
        (self.fm_channels[ch].operators[op].phase_acc + mod_in).sin() * tl
    }

    fn get_fm_output(&mut self, ch: usize) -> (f32, f32) {
        let any_on = self.fm_channels[ch].operators.iter().any(|op| op.key_on);
        if !any_on {
            return (0.0, 0.0);
        }

        let fb = self.op_feedback[ch] * (self.fm_channels[ch].feedback as f32 / 7.0) * 0.25;
        let m1 = self.op_out(ch, 0, fb);
        self.op_feedback[ch] = m1;

        let output = match self.fm_channels[ch].algorithm {
            0 => {
                // M1→M2→C1→C2
                let m2 = self.op_out(ch, 1, m1 * PI);
                let c1 = self.op_out(ch, 2, m2 * PI);
                self.op_out(ch, 3, c1 * PI)
            }
            1 => {
                let m2 = self.op_out(ch, 1, 0.0);
                let c1 = self.op_out(ch, 2, (m1 + m2) * PI * 0.5);
                self.op_out(ch, 3, c1 * PI)
            }
            2 => {
                let m2 = self.op_out(ch, 1, 0.0);
                let c1 = self.op_out(ch, 2, m2 * PI);
                self.op_out(ch, 3, (m1 + c1) * PI * 0.5)
            }
            3 => {
                let m2 = self.op_out(ch, 1, m1 * PI);
                let c1 = self.op_out(ch, 2, m2 * PI);
                self.op_out(ch, 3, (m2 + c1) * PI * 0.5)
            }
            4 => {
                let m2 = self.op_out(ch, 1, m1 * PI);
                let c1 = self.op_out(ch, 2, 0.0);
                (m2 + self.op_out(ch, 3, c1 * PI)) * 0.5
            }
            5 => {
                let m2 = self.op_out(ch, 1, m1 * PI);
                let c1 = self.op_out(ch, 2, m1 * PI);
                let c2 = self.op_out(ch, 3, m1 * PI);
                (m2 + c1 + c2) / 3.0
            }
            6 => {
                let m2 = self.op_out(ch, 1, m1 * PI);
                let c1 = self.op_out(ch, 2, 0.0);
                let c2 = self.op_out(ch, 3, 0.0);
                (m2 + c1 + c2) / 3.0
            }
            _ => {
                let m2 = self.op_out(ch, 1, 0.0);
                let c1 = self.op_out(ch, 2, 0.0);
                let c2 = self.op_out(ch, 3, 0.0);
                (m1 + m2 + c1 + c2) * 0.25
            }
        };

        let sample = (output * 0.2).clamp(-1.0, 1.0);
        let left = if self.fm_channels[ch].left_enable {
            sample
        } else {
            0.0
        };
        let right = if self.fm_channels[ch].right_enable {
            sample
        } else {
            0.0
        };
        (left, right)
    }

    fn advance_ssg_phases(&mut self, sample_rate: u32) {
        for ch in 0..3 {
            if self.ssg_channels[ch].period == 0 {
                continue;
            }
            let freq = self.clock_rate as f32 / (8.0 * self.ssg_channels[ch].period as f32);
            self.ssg_channels[ch].phase_acc += freq / sample_rate as f32;
            if self.ssg_channels[ch].phase_acc >= 1.0 {
                self.ssg_channels[ch].phase_acc -= 1.0;
            }
        }
    }

    fn get_ssg_output(&self, ch: usize) -> f32 {
        let ssg = &self.ssg_channels[ch];
        let tone_active = (self.ssg_mixer >> ch) & 0x01 == 0; // active when bit is 0
        if !tone_active || ssg.volume == 0 || ssg.period == 0 {
            return 0.0;
        }
        let square = if ssg.phase_acc < 0.5 { 1.0f32 } else { -1.0f32 };
        square * (ssg.volume as f32 / 15.0) * 0.08
    }
}

impl SoundChipEmulator for YM2203 {
    fn name(&self) -> &'static str {
        "YM2203 (OPN)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        if (addr as usize) < self.regs.len() {
            self.regs[addr as usize] = data;
        }

        match addr {
            // SSG tone period: lo = bits[7:0], hi = bits[11:8] (4 bits)
            0x00 => {
                self.ssg_channels[0].period = (self.ssg_channels[0].period & 0xF00) | data as u16
            }
            0x01 => {
                self.ssg_channels[0].period =
                    (self.ssg_channels[0].period & 0x0FF) | ((data as u16 & 0x0F) << 8)
            }
            0x02 => {
                self.ssg_channels[1].period = (self.ssg_channels[1].period & 0xF00) | data as u16
            }
            0x03 => {
                self.ssg_channels[1].period =
                    (self.ssg_channels[1].period & 0x0FF) | ((data as u16 & 0x0F) << 8)
            }
            0x04 => {
                self.ssg_channels[2].period = (self.ssg_channels[2].period & 0xF00) | data as u16
            }
            0x05 => {
                self.ssg_channels[2].period =
                    (self.ssg_channels[2].period & 0x0FF) | ((data as u16 & 0x0F) << 8)
            }
            0x07 => self.ssg_mixer = data,
            0x08 => self.ssg_channels[0].volume = data & 0x0F,
            0x09 => self.ssg_channels[1].volume = data & 0x0F,
            0x0A => self.ssg_channels[2].volume = data & 0x0F,

            // FM key on/off
            0x28 => {
                let ch = (data & 0x03) as usize;
                if ch < 3 {
                    let prev_any = self.fm_channels[ch].operators.iter().any(|op| op.key_on);
                    self.fm_channels[ch].operators[0].key_on = (data & 0x10) != 0; // M1
                    self.fm_channels[ch].operators[1].key_on = (data & 0x20) != 0; // M2
                    self.fm_channels[ch].operators[2].key_on = (data & 0x40) != 0; // C1
                    self.fm_channels[ch].operators[3].key_on = (data & 0x80) != 0; // C2
                    let new_any = self.fm_channels[ch].operators.iter().any(|op| op.key_on);
                    if !prev_any && new_any {
                        for op in &mut self.fm_channels[ch].operators {
                            op.phase_acc = 0.0;
                        }
                        self.op_feedback[ch] = 0.0;
                    }
                }
            }

            // FM operator registers (offset from base)
            0x30..=0x3F => {
                if let Some((ch, op)) = opn_ch_op(addr - 0x30) {
                    self.fm_channels[ch].operators[op].mult = data & 0x0F;
                }
            }
            0x40..=0x4F => {
                if let Some((ch, op)) = opn_ch_op(addr - 0x40) {
                    self.fm_channels[ch].operators[op].total_level = data & 0x7F;
                }
            }
            0x50..=0x8F => {} // AR/DR/SR/SL/RR stored in cache only

            // FM channel frequency
            0xA0..=0xA2 => {
                let ch = (addr - 0xA0) as usize;
                self.fm_channels[ch].f_num = (self.fm_channels[ch].f_num & 0x700) | data as u16;
            }
            0xA4..=0xA6 => {
                let ch = (addr - 0xA4) as usize;
                self.fm_channels[ch].block = (data >> 3) & 0x07;
                self.fm_channels[ch].f_num =
                    (self.fm_channels[ch].f_num & 0x0FF) | (((data as u16) & 0x07) << 8);
            }

            // FM channel algorithm/feedback
            0xB0..=0xB2 => {
                let ch = (addr - 0xB0) as usize;
                self.fm_channels[ch].algorithm = data & 0x07;
                self.fm_channels[ch].feedback = (data >> 3) & 0x07;
            }
            // FM channel L/R
            0xB4..=0xB6 => {
                let ch = (addr - 0xB4) as usize;
                self.fm_channels[ch].left_enable = (data & 0x80) != 0;
                self.fm_channels[ch].right_enable = (data & 0x40) != 0;
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        self.sample_rate = sample_rate;

        for frame in buffer.chunks_mut(2) {
            self.advance_fm_phases(sample_rate);
            self.advance_ssg_phases(sample_rate);

            let mut left = 0.0f32;
            let mut right = 0.0f32;
            for ch in 0..3 {
                let (l, r) = self.get_fm_output(ch);
                left += l;
                right += r;
            }
            for ch in 0..3 {
                let s = self.get_ssg_output(ch);
                left += s;
                right += s;
            }
            frame[0] = (left / 3.0).clamp(-1.0, 1.0);
            frame[1] = (right / 3.0).clamp(-1.0, 1.0);
        }
    }
}

impl Default for YM2203 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2203_new() {
        let chip = YM2203::new();
        assert_eq!(chip.name(), "YM2203 (OPN)");
        assert_eq!(chip.clock_rate(), 3_993_600);
    }

    #[test]
    fn test_ym2203_channels() {
        let chip = YM2203::new();
        assert_eq!(chip.fm_channels.len(), 3);
        assert_eq!(chip.ssg_channels.len(), 3);
    }

    #[test]
    fn test_ym2203_opn_ch_op_mapping() {
        assert_eq!(opn_ch_op(0), Some((0, 0))); // ch0 op0 (M1)
        assert_eq!(opn_ch_op(1), Some((1, 0))); // ch1 op0 (M1)
        assert_eq!(opn_ch_op(2), Some((2, 0))); // ch2 op0 (M1)
        assert_eq!(opn_ch_op(3), None); // unused
        assert_eq!(opn_ch_op(4), Some((0, 1))); // ch0 op1 (M2)
        assert_eq!(opn_ch_op(12), Some((0, 3))); // ch0 op3 (C2)
    }

    #[test]
    fn test_ym2203_write_fm_freq_and_keyon() {
        let mut chip = YM2203::new();
        chip.write(0xA0, 0x57); // ch0 f_num lo
        chip.write(0xA4, 0x21); // ch0 block=4, f_num hi=1 → f_num = 0x157
        assert_eq!(chip.fm_channels[0].f_num, 0x157);
        assert_eq!(chip.fm_channels[0].block, 4);

        chip.write(0x28, 0xF0); // key on all ops ch0 (0xF0 = 0b1111_0000, ch=0)
        assert!(chip.fm_channels[0].operators[0].key_on); // M1
        assert!(chip.fm_channels[0].operators[3].key_on); // C2
    }

    #[test]
    fn test_ym2203_write_operator_tl() {
        let mut chip = YM2203::new();
        chip.write(0x40, 0x20); // offset 0 → ch0 op0 TL=32
        assert_eq!(chip.fm_channels[0].operators[0].total_level, 32);
        chip.write(0x4C, 0x10); // offset 0x0C=12 → ch0 op3 TL=16
        assert_eq!(chip.fm_channels[0].operators[3].total_level, 16);
    }

    #[test]
    fn test_ym2203_generate_samples_fm_active() {
        let mut chip = YM2203::new();
        chip.write(0xB0, 0x07); // ch0 AL=7 (additive)
        chip.write(0xA0, 0x57);
        chip.write(0xA4, 0x21);
        chip.write(0x28, 0xF0); // key on all ops ch0
        let mut buffer = [0.0f32; 8];
        chip.generate_samples(&mut buffer, 44100);
        assert!(
            buffer.iter().any(|&s| s != 0.0),
            "FM channel must produce output"
        );
    }

    #[test]
    fn test_ym2203_ssg_write() {
        let mut chip = YM2203::new();
        chip.write(0x00, 0xBE); // ch0 period lo
        chip.write(0x01, 0x01); // ch0 period hi
        chip.write(0x07, 0xF8); // mixer: ch0 tone enabled (bit 0 = 0)
        chip.write(0x08, 0x0F); // ch0 volume = 15
        assert_eq!(chip.ssg_channels[0].period, 0x1BE);
        assert_eq!(chip.ssg_channels[0].volume, 15);
    }

    #[test]
    fn test_ym2203_soundchip_trait() {
        let mut chip = YM2203::new();
        assert_eq!(chip.name(), "YM2203 (OPN)");
        chip.reset();
        chip.write(0xA0, 0x57);
        chip.write(0xA4, 0x21);
        chip.write(0x28, 0xF0);
        chip.clock();
        let mut buffer = [0.0f32; 4];
        chip.generate_samples(&mut buffer, 44100);
        assert_eq!(buffer.len(), 4);
    }
}
