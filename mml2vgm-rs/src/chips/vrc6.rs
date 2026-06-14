//! VRC6 (Konami NES expansion chip) emulation
//!
//! 2 pulse channels and 1 sawtooth channel.
//! VGM opcode 0xB6: two-byte payload [addr, data].
//!
//! Register map:
//!   0x00: Pulse1 control — bits[3:0]=volume, bits[6:4]=duty(0-7), bit7=halt
//!   0x01: Pulse1 period lo
//!   0x02: Pulse1 period hi + enable(bit7)
//!   0x10: Pulse2 control (same layout)
//!   0x11: Pulse2 period lo
//!   0x12: Pulse2 period hi + enable
//!   0x20: Sawtooth accum_rate (bits[5:0])
//!   0x21: Sawtooth period lo
//!   0x22: Sawtooth period hi + enable

use super::SoundChipEmulator;

#[derive(Debug, Clone, Copy, Default)]
struct PulseChannel {
    volume: u8,
    duty: u8,
    halt: bool,
    period: u16,
    enabled: bool,
    phase_acc: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct SawtoothChannel {
    accum_rate: u8,
    period: u16,
    enabled: bool,
    phase_acc: f32,
}

/// VRC 6.
pub struct VRC6 {
    clock_rate: u32,
    pulse: [PulseChannel; 2],
    sawtooth: SawtoothChannel,
}

impl VRC6 {
    /// New.
    pub fn new() -> Self {
        Self::with_clock_rate(1_789_772)
    }

    /// With clock rate.
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            pulse: [PulseChannel::default(); 2],
            sawtooth: SawtoothChannel::default(),
        }
    }

    fn pulse_freq_hz(&self, ch: usize) -> f32 {
        let period = self.pulse[ch].period.max(1) as f32;
        self.clock_rate as f32 / (16.0 * (period + 1.0))
    }

    fn saw_freq_hz(&self) -> f32 {
        let period = self.sawtooth.period.max(1) as f32;
        self.clock_rate as f32 / (14.0 * (period + 1.0))
    }
}

impl SoundChipEmulator for VRC6 {
    fn name(&self) -> &'static str {
        "VRC6"
    }
    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        match addr {
            // Pulse 1
            0x00 => {
                self.pulse[0].volume = data & 0x0F;
                self.pulse[0].duty = (data >> 4) & 0x07;
                self.pulse[0].halt = (data & 0x80) != 0;
            }
            0x01 => {
                self.pulse[0].period = (self.pulse[0].period & 0x0F00) | data as u16;
            }
            0x02 => {
                self.pulse[0].period =
                    (self.pulse[0].period & 0x00FF) | (((data & 0x0F) as u16) << 8);
                self.pulse[0].enabled = (data & 0x80) != 0;
            }
            // Pulse 2
            0x10 => {
                self.pulse[1].volume = data & 0x0F;
                self.pulse[1].duty = (data >> 4) & 0x07;
                self.pulse[1].halt = (data & 0x80) != 0;
            }
            0x11 => {
                self.pulse[1].period = (self.pulse[1].period & 0x0F00) | data as u16;
            }
            0x12 => {
                self.pulse[1].period =
                    (self.pulse[1].period & 0x00FF) | (((data & 0x0F) as u16) << 8);
                self.pulse[1].enabled = (data & 0x80) != 0;
            }
            // Sawtooth
            0x20 => {
                self.sawtooth.accum_rate = data & 0x3F;
            }
            0x21 => {
                self.sawtooth.period = (self.sawtooth.period & 0x0F00) | data as u16;
            }
            0x22 => {
                self.sawtooth.period =
                    (self.sawtooth.period & 0x00FF) | (((data & 0x0F) as u16) << 8);
                self.sawtooth.enabled = (data & 0x80) != 0;
            }
            _ => {}
        }
    }

    fn read(&self, _addr: u8) -> u8 {
        0xFF
    }
    fn clock(&mut self) {}

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for frame in buffer.chunks_mut(2) {
            let mut out = 0.0f32;

            // Pulse channels
            for ch in 0..2 {
                if !self.pulse[ch].enabled || self.pulse[ch].halt {
                    continue;
                }
                let freq = self.pulse_freq_hz(ch);
                let phase_inc = freq / sample_rate as f32;
                self.pulse[ch].phase_acc += phase_inc;
                if self.pulse[ch].phase_acc >= 1.0 {
                    self.pulse[ch].phase_acc -= 1.0;
                }
                let vol = self.pulse[ch].volume as f32 / 15.0;
                // duty 7 = always high; threshold = duty/8 clamped to 0.875
                let threshold = if self.pulse[ch].duty == 7 {
                    0.875f32
                } else {
                    self.pulse[ch].duty as f32 / 8.0
                };
                let sample = if self.pulse[ch].phase_acc < threshold {
                    vol
                } else {
                    -vol
                };
                out += sample * 0.25;
            }

            // Sawtooth channel
            if self.sawtooth.enabled {
                let freq = self.saw_freq_hz();
                let phase_inc = freq / sample_rate as f32;
                self.sawtooth.phase_acc += phase_inc;
                if self.sawtooth.phase_acc >= 1.0 {
                    self.sawtooth.phase_acc -= 1.0;
                }
                let amp = self.sawtooth.accum_rate as f32 / 63.0;
                let sample = (2.0 * self.sawtooth.phase_acc - 1.0) * amp;
                out += sample * 0.25;
            }

            let mixed = out.clamp(-1.0, 1.0);
            if frame.len() >= 2 {
                frame[0] = mixed;
                frame[1] = mixed;
            }
        }
    }
}

impl Default for VRC6 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vrc6_new() {
        let chip = VRC6::new();
        assert_eq!(chip.name(), "VRC6");
        assert_eq!(chip.clock_rate(), 1_789_772);
    }

    #[test]
    fn test_vrc6_reset() {
        let mut chip = VRC6::new();
        chip.write(0x02, 0x80); // enable pulse 1
        chip.reset();
        assert!(!chip.pulse[0].enabled);
        assert!(!chip.sawtooth.enabled);
    }

    #[test]
    fn test_vrc6_write() {
        let mut chip = VRC6::new();
        chip.write(0x00, 0x3F); // duty=3, volume=15
        assert_eq!(chip.pulse[0].duty, 3);
        assert_eq!(chip.pulse[0].volume, 15);
        chip.write(0x01, 0xAB); // period lo
        chip.write(0x02, 0x85); // period hi=5, enable
        assert_eq!(chip.pulse[0].period, 0x5AB);
        assert!(chip.pulse[0].enabled);
    }

    #[test]
    fn test_vrc6_generate_samples_active() {
        let mut chip = VRC6::new();
        // Set up pulse 1
        chip.write(0x00, 0x3F); // duty=3, volume=15
        chip.write(0x01, 0x80); // period lo
        chip.write(0x02, 0x80); // enable
        let mut buf = [0.0f32; 8];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().any(|&s| s != 0.0),
            "active VRC6 pulse must produce output"
        );
    }

    #[test]
    fn test_vrc6_soundchip_trait() {
        let mut chip = VRC6::new();
        assert!(chip.is_initialized());
        assert_eq!(chip.read(0x00), 0xFF);
        chip.clock(); // must not panic
                      // Sawtooth channel
        chip.write(0x20, 0x20); // accum_rate=32
        chip.write(0x21, 0x50); // period lo
        chip.write(0x22, 0x80); // enable
        let mut buf = [0.0f32; 8];
        chip.generate_samples(&mut buf, 44100);
        assert!(
            buf.iter().any(|&s| s != 0.0),
            "active VRC6 sawtooth must produce output"
        );
    }
}
