//! YMF271 (OPX) sound chip emulator.
//!
//! ## Architecture
//! The YMF271 is Yamaha's "OPX" chip used in Taito F3 System arcade hardware.
//! - 48 FM slots arranged as 12 groups × 4 slots per group
//! - 12 PCM channels with ADPCM playback
//! - Clock: 16,934,400 Hz
//!
//! ## Phase 1 implementation
//! This Rust module stores all register state and accepts VGM register writes.
//! Audio generation requires the libvgm C core (ymf271.c from MAME/libvgm);
//! until that FFI wrapper is wired in, `generate_samples` outputs silence.
//!
//! ## Register layout (4-port FM banks)
//! | VGM port | Physical offsets | Content |
//! |----------|-----------------|---------|
//! | 0 | 0x00/0x01 | Slot bank 0: AR, D1R, DT, KF, KS |
//! | 1 | 0x02/0x03 | Slot bank 1: D2R, RR, DL, SSGEG |
//! | 2 | 0x04/0x05 | Slot bank 2: TL, ML, WF, FB, LFO, AMS, PMS |
//! | 3 | 0x06/0x07 | Slot bank 3: ACC, CON |
//! | 4 | 0x08/0x09 | PCM channel registers |
//! | 6 | 0x0C/0x0D | Group timer / key-on / FNUM / BLOCK |
//!
//! ## References
//! - `../libvgm/emu/cores/ymf271.c` — MAME port, BSD-3-Clause
//! - docs/design/YMF271_OPL4_Implementation.md

use super::SoundChipEmulator;

const NUM_SLOTS: usize = 48;
const NUM_GROUPS: usize = 12;
const NUM_BANKS: usize = 4;
const CLOCK: u32 = 16_934_400;

/// Per-slot FM register state (banks 0–3).
#[derive(Debug, Clone, Default)]
struct SlotRegs {
    /// Bank 0: AR[4:0], D1R[4:0], DT[2:0], KF[5:0], KS[1:0]
    bank0: u8,
    /// Bank 1: D2R[4:0], RR[3:0], DL[3:0], SSGEG[3:0]
    bank1: u8,
    /// Bank 2: TL[6:0], ML[3:0], WF[2:0], FB[2:0], LFO[1:0], AMS[1:0], PMS[2:0]
    bank2: u8,
    /// Bank 3: ACC[2:0], CON[2:0]
    bank3: u8,
}

/// Per-group timer/frequency register state.
#[derive(Debug, Clone, Default)]
struct GroupRegs {
    /// FNUM low 8 bits (address 0x00 + group)
    fnum_lo: u8,
    /// BLOCK[2:0] | FNUM[8] | KEYON (address 0x10 + group)
    fnum_hi: u8,
}

/// PCM channel register state.
#[derive(Debug, Clone, Default)]
struct PcmChannelRegs {
    regs: [u8; 8],
}

/// YMF271 (OPX) chip emulator.
pub struct YMF271 {
    slot_regs: [SlotRegs; NUM_SLOTS],
    group_regs: [GroupRegs; NUM_GROUPS],
    pcm_regs: [PcmChannelRegs; 12],
    rom: Vec<u8>,
}

impl YMF271 {
    pub fn new() -> Self {
        Self {
            slot_regs: std::array::from_fn(|_| SlotRegs::default()),
            group_regs: std::array::from_fn(|_| GroupRegs::default()),
            pcm_regs: std::array::from_fn(|_| PcmChannelRegs::default()),
            rom: Vec::new(),
        }
    }

    /// Map VGM port → write to the appropriate register store.
    ///
    /// VGM command 0xD1 [port, reg, data] calls this function.
    /// The libvgm player calls `ymf271_w(state, port*2, reg)` then
    /// `ymf271_w(state, port*2+1, data)`, which this mirrors at a higher level.
    fn write_port(&mut self, port: u8, reg: u8, data: u8) {
        match port {
            0 => {
                // FM bank 0 — slots 0-23 at reg 0x00-0x17; slots 24-47 would use regs 0x00-0x17 at port 0+4
                let slot = reg as usize;
                if slot < NUM_SLOTS {
                    self.slot_regs[slot].bank0 = data;
                }
            }
            1 => {
                let slot = reg as usize;
                if slot < NUM_SLOTS {
                    self.slot_regs[slot].bank1 = data;
                }
            }
            2 => {
                let slot = reg as usize;
                if slot < NUM_SLOTS {
                    self.slot_regs[slot].bank2 = data;
                }
            }
            3 => {
                let slot = reg as usize;
                if slot < NUM_SLOTS {
                    self.slot_regs[slot].bank3 = data;
                }
            }
            4 => {
                // PCM channel registers
                let ch = (reg & 0x0F) as usize;
                let param = (reg >> 4) as usize;
                if ch < 12 && param < 8 {
                    self.pcm_regs[ch].regs[param] = data;
                }
            }
            6 => {
                // Group timer / frequency / key-on
                let group_idx = (reg & 0x0F) as usize;
                if group_idx < NUM_GROUPS {
                    if reg < 0x10 {
                        self.group_regs[group_idx].fnum_lo = data;
                    } else {
                        self.group_regs[group_idx].fnum_hi = data;
                    }
                }
            }
            _ => {}
        }
    }
}

impl SoundChipEmulator for YMF271 {
    fn name(&self) -> &'static str { "YMF271" }
    fn clock_rate(&self) -> u32 { CLOCK }

    fn reset(&mut self) {
        for s in &mut self.slot_regs { *s = SlotRegs::default(); }
        for g in &mut self.group_regs { *g = GroupRegs::default(); }
        for p in &mut self.pcm_regs { *p = PcmChannelRegs::default(); }
    }

    fn write(&mut self, addr: u8, data: u8) {
        // Single-port write (addr = register address, port defaults to 0)
        self.write_port(0, addr, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        self.write_port(port, addr, data);
    }

    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], _sample_rate: u32) {
        // Audio generation requires the libvgm C core (ymf271.c).
        // Phase 1 stub: produce silence until the FFI wrapper is added.
        buffer.fill(0.0);
    }

    fn load_pcm_data(&mut self, _block_type: u8, data: &[u8]) {
        self.rom.extend_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::SoundChipEmulator;

    #[test]
    fn test_ymf271_register_write() {
        let mut chip = YMF271::new();
        // Bank 2, slot 0: write TL
        chip.write_port(2, 0x00, 0x3F);
        assert_eq!(chip.slot_regs[0].bank2, 0x3F);
    }

    #[test]
    fn test_ymf271_group_fnum_write() {
        let mut chip = YMF271::new();
        // Group 0 FNUM low
        chip.write_port(6, 0x00, 0xAB);
        assert_eq!(chip.group_regs[0].fnum_lo, 0xAB);
        // Group 0 FNUM high + key-on
        chip.write_port(6, 0x10, 0x83); // bit7=keyon, block=0, fnum_hi=3
        assert_eq!(chip.group_regs[0].fnum_hi, 0x83);
    }

    #[test]
    fn test_ymf271_generates_silence() {
        let mut chip = YMF271::new();
        let mut buf = vec![0.1f32; 64];
        chip.generate_samples(&mut buf, 44100);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_ymf271_reset() {
        let mut chip = YMF271::new();
        chip.write_port(0, 0x00, 0xFF);
        chip.reset();
        assert_eq!(chip.slot_regs[0].bank0, 0);
    }
}
