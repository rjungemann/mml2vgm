//! QSound (Capcom DL-1425) 16-voice PCM chip emulation.
//!
//! VGM opcode 0xC4: three-byte payload [data_hi, addr, data_lo].
//! The full 16-bit word `(data_hi << 8) | data_lo` is written to DSP register `addr`.
//!
//! Register map (one 16-bit word per entry):
//!   Voice n (0..15) occupies addresses n*8 .. n*8+7:
//!     n*8+0  start_addr  — ROM word address to play from (key-on resets position here)
//!     n*8+1  step        — pitch step (0x0100 = 1× native speed)
//!     n*8+2  loop_addr   — word address to jump back to on loop
//!     n*8+3  end_addr    — word address at which playback ends / loops
//!     n*8+4  key_on_ctl  — bit15=start, bit0=loop-enable; writing bit15 resets position
//!     n*8+5  volume      — linear, 0=silent, 0x0FFF=max
//!     n*8+6  pan_left    — left  channel mix (0..0x1F, 0x10=full, 0=mute)
//!     n*8+7  pan_right   — right channel mix (0..0x1F, 0x10=full, 0=mute)
//!
//! PCM ROM: 16-bit signed samples loaded via VGM data block type 0x88.
//! ROM size: up to 8 MB (4 M × 16-bit words), byte 0/1 of each word are lo/hi.
//!
//! Native sample rate: ~24038 Hz (4 MHz clock / 166.4 cycles/sample).
//! The `generate_samples` method resamples to the requested output rate.

use super::SoundChipEmulator;

const NUM_VOICES: usize = 16;
const ROM_SIZE: usize = 8 * 1024 * 1024; // 8 MB = 4 M × 16-bit words
const NATIVE_RATE: f64 = 24038.0;
// step 0x0100 ≈ 1 native sample per output step at native rate
const STEP_UNITY: f64 = 0x0100 as f64;

#[derive(Debug, Clone, Default)]
struct QSoundVoice {
    start_addr: u32,
    loop_addr: u32,
    end_addr: u32,
    /// Fixed-point position: integer part = ROM word index; fraction = 0..0xFFFF
    position: u32,
    step: u16,
    volume: u16,
    pan_left: u8,
    pan_right: u8,
    key_on: bool,
    loop_en: bool,
}

pub struct QSound {
    clock_rate: u32,
    voices: [QSoundVoice; NUM_VOICES],
    rom: Vec<u8>,
    /// Accumulator for the resampling step (output → native sample phase)
    resample_acc: f64,
}

impl QSound {
    pub fn new() -> Self {
        Self::with_clock_rate(4_000_000)
    }

    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            voices: std::array::from_fn(|_| QSoundVoice::default()),
            rom: vec![0u8; ROM_SIZE],
            resample_acc: 0.0,
        }
    }

    /// Read a 16-bit signed sample from the ROM at word address `word_addr`.
    #[allow(dead_code)]
    fn read_sample(&self, word_addr: u32) -> i16 {
        Self::read_sample_static(word_addr, &self.rom)
    }

    /// Advance one native-rate tick and return (left, right) output sample pair.
    ///
    /// Position is Q24.8 fixed-point (24-bit integer word address, 8-bit fraction).
    /// step=0x0100 → advance 1 ROM word per native tick (unity speed).
    fn clock_native(&mut self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for v in &mut self.voices {
            if !v.key_on { continue; }

            // Integer word address = upper 24 bits (Q24.8 → shift right by 8)
            let word_addr = v.position >> 8;

            // Check end condition
            if v.end_addr > 0 && word_addr >= v.end_addr as u32 {
                if v.loop_en {
                    let frac = v.position & 0xFF;
                    v.position = ((v.loop_addr as u32) << 8) | frac;
                } else {
                    v.key_on = false;
                    continue;
                }
            }

            let raw = Self::read_sample_static(word_addr, &self.rom);
            let sample = raw as f32 / 32768.0;
            let vol = v.volume.min(0x0FFF) as f32 / 0x0FFF_u32 as f32;
            let s = sample * vol;

            let l_gain = (v.pan_left as f32 / 16.0_f32).min(1.0);
            let r_gain = (v.pan_right as f32 / 16.0_f32).min(1.0);
            left  += s * l_gain;
            right += s * r_gain;

            // step is Q8.8: 0x0100 = 1.0 word per tick
            v.position = v.position.wrapping_add(v.step as u32);
        }

        (left, right)
    }

    fn read_sample_static(word_addr: u32, rom: &[u8]) -> i16 {
        let byte_off = (word_addr as usize) * 2;
        if byte_off + 1 < rom.len() {
            i16::from_le_bytes([rom[byte_off], rom[byte_off + 1]])
        } else {
            0
        }
    }

    fn write_reg(&mut self, addr: u8, data: u16) {
        let addr = addr as usize;
        if addr < NUM_VOICES * 8 {
            let ch = addr / 8;
            let reg = addr % 8;
            let v = &mut self.voices[ch];
            match reg {
                0 => {
                    v.start_addr = data as u32;
                    // Writing start_addr resets playback position (Q24.8)
                    v.position = (data as u32) << 8;
                }
                1 => v.step = data,
                2 => v.loop_addr = data as u32,
                3 => v.end_addr = data as u32,
                4 => {
                    // bit15 = key-on trigger; bit0 = loop enable
                    v.loop_en = (data & 0x0001) != 0;
                    if (data & 0x8000) != 0 {
                        v.position = v.start_addr << 8;
                        v.key_on = true;
                    } else if data == 0 {
                        v.key_on = false;
                    }
                }
                5 => v.volume = data & 0x0FFF,
                6 => v.pan_left  = (data & 0x1F) as u8,
                7 => v.pan_right = (data & 0x1F) as u8,
                _ => {}
            }
        }
        // Addresses 0x80..0x8F: alternate per-voice pan shorthand (ignored here)
    }
}

impl SoundChipEmulator for QSound {
    fn name(&self) -> &'static str { "QSound" }
    fn clock_rate(&self) -> u32 { self.clock_rate }

    fn reset(&mut self) {
        self.voices = std::array::from_fn(|_| QSoundVoice::default());
        self.resample_acc = 0.0;
    }

    fn write(&mut self, addr: u8, data: u8) {
        // Single-byte write: treat as low byte with 0 high byte
        self.write_reg(addr, data as u16);
    }

    // VGM 0xC4: port=data_hi, addr=addr, data=data_lo → 16-bit write
    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        let word = ((port as u16) << 8) | (data as u16);
        self.write_reg(addr, word);
    }

    fn clock(&mut self) {}

    fn load_pcm_data(&mut self, block_type: u8, data: &[u8]) {
        if block_type == 0x88 {
            let copy_len = data.len().min(self.rom.len());
            self.rom[..copy_len].copy_from_slice(&data[..copy_len]);
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        let out_rate = sample_rate as f64;
        // How many native ticks advance per output sample
        let ticks_per_sample = NATIVE_RATE / out_rate;

        for frame in buffer.chunks_mut(2) {
            self.resample_acc += ticks_per_sample;
            let mut acc_left = 0.0f32;
            let mut acc_right = 0.0f32;
            let ticks = self.resample_acc as u32;
            for _ in 0..ticks {
                let (l, r) = self.clock_native();
                acc_left  += l;
                acc_right += r;
            }
            if ticks > 0 {
                acc_left  /= ticks as f32;
                acc_right /= ticks as f32;
            }
            self.resample_acc -= ticks as f64;

            let scale = 1.0 / NUM_VOICES as f32;
            if frame.len() >= 2 {
                frame[0] = (acc_left  * scale).clamp(-1.0, 1.0);
                frame[1] = (acc_right * scale).clamp(-1.0, 1.0);
            }
        }
    }
}

impl Default for QSound {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qsound_new() {
        let chip = QSound::new();
        assert_eq!(chip.name(), "QSound");
        assert_eq!(chip.clock_rate(), 4_000_000);
        assert!(chip.is_initialized());
    }

    #[test]
    fn test_qsound_silence_when_no_voices_active() {
        let mut chip = QSound::new();
        let mut buf = [0.0f32; 16];
        chip.generate_samples(&mut buf, 44100);
        assert!(buf.iter().all(|&s| s == 0.0), "idle chip must produce silence");
    }

    #[test]
    fn test_qsound_load_pcm_data() {
        let mut chip = QSound::new();
        let data: Vec<u8> = (0u8..=255).collect();
        chip.load_pcm_data(0x88, &data);
        assert_eq!(chip.rom[0], 0);
        assert_eq!(chip.rom[255], 255);
        // Wrong block type must not overwrite
        let zeros = vec![0u8; 256];
        chip.load_pcm_data(0x00, &zeros);
        assert_eq!(chip.rom[255], 255, "wrong block type must not overwrite ROM");
    }

    #[test]
    fn test_qsound_key_on_produces_sound() {
        let mut chip = QSound::new();

        // Fill ROM words 0..127 with a constant non-zero value (0x4000 = +0.5 FS)
        for i in 0..128usize {
            let val: i16 = 0x4000;
            let idx = i * 2;
            chip.rom[idx]     = val as u8;
            chip.rom[idx + 1] = (val >> 8) as u8;
        }

        // Voice 0: start=0, end=128, step=0x0100 (1 word/tick), volume=0x0FFF, full pan, loop
        chip.write_port(0x00, 0x00, 0x00); // start_addr = 0
        chip.write_port(0x01, 0x01, 0x00); // step = 0x0100
        chip.write_port(0x00, 0x02, 0x00); // loop_addr = 0
        chip.write_port(0x00, 0x03, 0x80); // end_addr = 128
        chip.write_port(0x0F, 0x05, 0xFF); // volume = 0x0FFF
        chip.write_port(0x00, 0x06, 0x10); // pan_left = 0x10 (full)
        chip.write_port(0x00, 0x07, 0x10); // pan_right = 0x10 (full)
        chip.write_port(0x80, 0x04, 0x01); // key-on + loop

        let mut buf = vec![0.0f32; 256];
        chip.generate_samples(&mut buf, 44100);

        let any_nonzero = buf.iter().any(|&s| s != 0.0);
        assert!(any_nonzero, "key-on voice with ROM data must produce non-silent output");
    }

    #[test]
    fn test_qsound_reset_silences() {
        let mut chip = QSound::new();
        chip.voices[0].key_on = true;
        chip.voices[0].volume = 0x0FFF;
        chip.reset();
        assert!(!chip.voices[0].key_on);
        assert_eq!(chip.voices[0].volume, 0);
    }

    #[test]
    fn test_qsound_write_port_decodes_16bit() {
        let mut chip = QSound::new();
        // Write 0xABCD to voice 0 step register (addr 1)
        chip.write_port(0xAB, 0x01, 0xCD);
        assert_eq!(chip.voices[0].step, 0xABCD);
    }
}
