//! YM2612 (OPN2) sound chip emulation
//!
//! The YM2612 is a 6-channel FM synthesis chip used in the Sega Mega Drive/Genesis.
//! It is part of the OPN2 family of Yamaha sound chips.
//!
//! # Features
//! - 6 FM channels (each with 4 operators)
//! - 24 operators total
//! - 8-bit DAC for PCM playback (not implemented yet)
//! - LFO (Low Frequency Oscillator)
//! - Two timers
//! - Stereo output
//!
//! # Register Map
//! - 0x00-0x22: Test registers (usually not used)
//! - 0x20-0x2F: LFO
//! - 0x30-0x3F: Timer control
//! - 0x40-0x5F: Channel 0-2 operators
//! - 0x60-0x7F: Channel 3-5 operators
//! - 0x80-0x8F: Channel key on/off
//! - 0x90-0x9F: Channel frequency (low)
//! - 0xA0-0xA8: Channel frequency (high) / octave
//! - 0xA4-0xA6: Timer A
//! - 0xA8-0xAA: Timer B
//! - 0xB0-0xB6: Channel algorithm/feedback
//! - 0xB8-0xBF: Not used
//! - 0xC0-0xCF: Not used
//! - 0xD0-0xDF: Not used
//! - 0xE0-0xFF: Not used

use super::SoundChipEmulator;

/// FM operator
#[derive(Debug, Clone, Copy)]
pub struct FmOperator {
    /// Detune (0-7)
    pub detune: u8,
    /// Multiple (0-15)
    pub multiple: u8,
    /// Total Level (0-127)
    pub total_level: u8,
    /// Rate Scaling / Key Scaling Rate (0-3)
    pub rate_scaling: u8,
    /// Attack Rate (0-31)
    pub attack_rate: u8,
    /// Decay Rate (0-31)
    pub decay_rate: u8,
    /// Sustain Rate (0-31)
    pub sustain_rate: u8,
    /// Release Rate (0-15)
    pub release_rate: u8,
    /// Sustain Level (0-15)
    pub sustain_level: u8,
    /// Waveform (0-3: sine, half-sine, absolute-sine, square)
    pub waveform: u8,
    /// SSG-EG (Software-controlled Envelope Generator) parameters
    pub ssg_eg: u8,
    /// 32-bit phase accumulator; upper 12 bits (31:20) are the 4096-step sine index
    pub phase: u32,
    /// Per-clock-cycle phase increment: f_num * 2^(11+block) / 144 * ML
    pub phase_increment: u32,
    /// Current envelope state
    pub envelope_state: EnvelopeState,
    /// Current envelope level (0-127)
    pub envelope_level: u8,
    /// Key is on
    pub key_on: bool,
}

impl Default for FmOperator {
    fn default() -> Self {
        Self {
            detune: 0,
            multiple: 1,
            total_level: 127,
            rate_scaling: 0,
            attack_rate: 31,
            decay_rate: 0,
            sustain_rate: 0,
            release_rate: 15,
            sustain_level: 15,
            waveform: 0,
            ssg_eg: 0,
            phase: 0,
            phase_increment: 0,
            envelope_state: EnvelopeState::Off,
            envelope_level: 127,
            key_on: false,
        }
    }
}

/// Envelope generator state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// FM channel
#[derive(Debug, Clone, Copy)]
pub struct FmChannel {
    /// 4 operators for this channel
    pub operators: [FmOperator; 4],
    /// Channel frequency (F-Number: 10 bits)
    pub frequency: u16,
    /// Channel octave (0-7)
    pub octave: u8,
    /// Channel key is on
    pub key_on: bool,
    /// Algorithm (0-7, determines how operators are connected)
    pub algorithm: u8,
    /// Feedback (0-7, amount of operator 3 output fed back to operator 1)
    pub feedback: u8,
    /// Output level (0-127)
    pub output_level: u8,
    /// Left output enabled
    pub left_enable: bool,
    /// Right output enabled
    pub right_enable: bool,
}

impl Default for FmChannel {
    fn default() -> Self {
        Self {
            operators: [Default::default(); 4],
            frequency: 0,
            octave: 0,
            key_on: false,
            algorithm: 0,
            feedback: 0,
            output_level: 0,
            left_enable: true,
            right_enable: true,
        }
    }
}

/// LFO (Low Frequency Oscillator) state
#[derive(Debug, Clone, Copy)]
pub struct Lfo {
    /// LFO enable
    pub enabled: bool,
    /// LFO frequency (0-7)
    pub frequency: u8,
    /// Current LFO phase
    pub phase: u8,
    /// LFO phase increment
    pub phase_increment: u8,
    /// LFO amplitude modulation depth
    pub am_depth: u8,
    /// LFO phase modulation depth
    pub pm_depth: u8,
    /// LFO waveform (0-3: triangle, square, sawtooth, random)
    pub waveform: u8,
}

impl Default for Lfo {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: 0,
            phase: 0,
            phase_increment: 0,
            am_depth: 0,
            pm_depth: 0,
            waveform: 0,
        }
    }
}

/// Timer state
#[derive(Debug, Clone, Copy)]
pub struct Timer {
    /// Timer enable
    pub enabled: bool,
    /// Timer value (10 bits)
    pub value: u16,
    /// Timer preset
    pub preset: u16,
    /// Timer expired flag
    pub expired: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            enabled: false,
            value: 0,
            preset: 0,
            expired: false,
        }
    }
}

/// YM2612 chip emulator
pub struct YM2612 {
    /// Master clock rate in Hz (NTSC: 7,670,453 Hz)
    clock_rate: u32,
    
    /// Sample rate for output
    sample_rate: u32,
    
    /// Clock divider for sample rate conversion
    clock_divider: f64,
    
    /// Accumulated clock cycles
    accumulated_cycles: f64,
    
    /// All 6 FM channels
    pub channels: [FmChannel; 6],
    
    /// LFO
    pub lfo: Lfo,
    
    /// Timers
    pub timer_a: Timer,
    pub timer_b: Timer,
    
    /// Register cache (0x00-0xFF)
    regs: [u8; 0x100],
    
    /// DAC state (not fully implemented)
    dac_enabled: bool,
    dac_sample: u8,
}

impl YM2612 {
    /// Create a new YM2612 emulator with the default NTSC clock rate
    pub fn new() -> Self {
        Self::with_clock_rate(7_670_453)
    }

    /// Create a new YM2612 emulator with a custom clock rate
    pub fn with_clock_rate(clock_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate: 44100,
            clock_divider: clock_rate as f64 / 44100.0,
            accumulated_cycles: 0.0,
            channels: [Default::default(); 6],
            lfo: Default::default(),
            timer_a: Default::default(),
            timer_b: Default::default(),
            regs: [0; 0x100],
            dac_enabled: false,
            dac_sample: 0,
        }
    }

    /// Set the sample rate for output
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.clock_divider = self.clock_rate as f64 / sample_rate as f64;
    }

    /// Get the current output sample
    pub fn get_output(&self) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Sum outputs from all channels
        for channel in &self.channels {
            let (ch_left, ch_right) = self.get_channel_output(channel);
            left += ch_left;
            right += ch_right;
        }

        // Normalize output
        let norm_factor = 6.0; // 6 channels max
        (left / norm_factor, right / norm_factor)
    }

    /// Get output from a single channel
    fn get_channel_output(&self, channel: &FmChannel) -> (f32, f32) {
        // For now, simple sine wave output from operator 1
        // Full FM synthesis implementation will come later
        
        if !channel.key_on {
            return (0.0, 0.0);
        }

        // Get output from operator 1 (simplified)
        let output = self.get_operator_output(&channel.operators[0]);

        // Apply output level (0 = max volume, 127 = silent)
        let scaled_output = output * (1.0 - channel.output_level as f32 / 127.0);

        // Apply panning
        let left = if channel.left_enable { scaled_output } else { 0.0 };
        let right = if channel.right_enable { scaled_output } else { 0.0 };

        (left, right)
    }

    /// Get output from a single operator
    fn get_operator_output(&self, op: &FmOperator) -> f32 {
        if !op.key_on {
            return 0.0;
        }

        // Extract 12-bit sine index from upper bits of 32-bit accumulator
        let phase_idx = (op.phase >> 20) & 0xFFF;
        let phase_f = phase_idx as f32 / 4096.0 * 2.0 * std::f32::consts::PI;
        let sine = phase_f.sin();

        // Apply envelope (0=loud, 127=silent)
        let envelope = 1.0 - op.envelope_level as f32 / 127.0;

        sine * envelope * 0.5
    }

    /// Update operator phase accumulator (add phase_increment per clock cycle)
    fn update_operator_phase(&mut self, op: &mut FmOperator) {
        op.phase = op.phase.wrapping_add(op.phase_increment);
    }

    /// Update envelope generator for an operator
    fn update_envelope(&mut self, op: &mut FmOperator) {
        match op.envelope_state {
            EnvelopeState::Off => {
                op.envelope_level = 127;
            }
            EnvelopeState::Attack => {
                // Simplified: instant attack for now
                if op.attack_rate > 0 {
                    op.envelope_level = op.envelope_level.saturating_sub(1);
                    if op.envelope_level == 0 {
                        op.envelope_state = EnvelopeState::Decay;
                    }
                }
            }
            EnvelopeState::Decay => {
                // Simplified: instant decay for now
                if op.decay_rate > 0 {
                    if op.envelope_level < op.sustain_level {
                        op.envelope_state = EnvelopeState::Sustain;
                    } else {
                        op.envelope_level = op.envelope_level.saturating_add(1);
                    }
                }
            }
            EnvelopeState::Sustain => {
                // Stay at sustain level
            }
            EnvelopeState::Release => {
                // Simplified: instant release for now
                if op.release_rate > 0 {
                    op.envelope_level = op.envelope_level.saturating_add(1);
                    if op.envelope_level >= 127 {
                        op.envelope_state = EnvelopeState::Off;
                        op.key_on = false;
                    }
                }
            }
        }
    }

    /// Update LFO
    fn update_lfo(&mut self) {
        if self.lfo.enabled && self.lfo.frequency > 0 {
            self.lfo.phase = self.lfo.phase.wrapping_add(self.lfo.phase_increment);
        }
    }

    /// Update timers
    fn update_timers(&mut self) {
        if self.timer_a.enabled {
            if self.timer_a.value == 0 {
                self.timer_a.value = self.timer_a.preset;
                self.timer_a.expired = true;
            } else {
                self.timer_a.value -= 1;
            }
        }

        if self.timer_b.enabled {
            if self.timer_b.value == 0 {
                self.timer_b.value = self.timer_b.preset;
                self.timer_b.expired = true;
            } else {
                self.timer_b.value -= 1;
            }
        }
    }

    /// Write to a register
    pub fn write_reg(&mut self, addr: u8, data: u8) {
        // Store in register cache
        self.regs[addr as usize] = data;

        // Parse the address
        // The YM2612 has two address spaces (part I and part II)
        // Part I: 0x00-0x7F, Part II: 0x80-0xFF
        let part = ((addr >> 7) & 0x01) as usize;

        // Handle register writes by full address to avoid conflicts
        match addr & 0xFF {
            // === Part I Registers (0x00-0x7F) ===

            // LFO control
            0x21 => {
                self.lfo.enabled = (data & 0x80) != 0;
                self.lfo.frequency = data & 0x07;
            }
            0x22 => {
                self.lfo.am_depth = (data >> 4) & 0x07;
                self.lfo.pm_depth = data & 0x0F;
            }

            // Timer A preset (high byte)
            0x24 => {
                self.timer_a.preset = ((data as u16 & 0x03) << 8) | (self.timer_a.preset & 0xFF);
            }
            // Timer A preset (low byte)
            0x25 => {
                self.timer_a.preset = (self.timer_a.preset & 0xFF00) | (data as u16);
            }
            // Timer B preset
            0x26 => {
                self.timer_b.preset = data as u16;
            }
            // Timer control and IRQ flags
            0x27 => {
                self.timer_a.enabled = (data & 0x80) != 0;
                self.timer_b.enabled = (data & 0x40) != 0;
                if (data & 0x80) != 0 {
                    self.timer_a.value = self.timer_a.preset;
                }
                if (data & 0x40) != 0 {
                    self.timer_b.value = self.timer_b.preset;
                }
            }

            // === Channel Frequency Low Byte ===
            // Both Part I (0x20-0x26) and Part I with part bit set (0xA0-0xA6)
            0x20 | 0xA0 => {
                self.channels[0].frequency = (self.channels[0].frequency & 0xFF00) | (data as u16);
            }
            0x21 | 0xA1 => {
                self.channels[1].frequency = (self.channels[1].frequency & 0xFF00) | (data as u16);
            }
            0x22 | 0xA2 => {
                self.channels[2].frequency = (self.channels[2].frequency & 0xFF00) | (data as u16);
            }
            0x24 | 0xA4 => {
                self.channels[3].frequency = (self.channels[3].frequency & 0xFF00) | (data as u16);
            }
            0x25 | 0xA5 => {
                self.channels[4].frequency = (self.channels[4].frequency & 0xFF00) | (data as u16);
            }
            0x26 | 0xA6 => {
                self.channels[5].frequency = (self.channels[5].frequency & 0xFF00) | (data as u16);
            }

            // === Channel Frequency High Byte & Octave (0xA8-0xAE) ===
            // These are the registers that conflicted with key on/off
            // Registers 0xA8-0xAA (channels 0-2) and 0xAC-0xAE (channels 3-5)
            0xA8 => {
                self.channels[0].frequency = (self.channels[0].frequency & 0x00FF) | ((data as u16 & 0x03) << 8);
                self.channels[0].octave = (data >> 2) & 0x07;
            }
            0xA9 => {
                self.channels[1].frequency = (self.channels[1].frequency & 0x00FF) | ((data as u16 & 0x03) << 8);
                self.channels[1].octave = (data >> 2) & 0x07;
            }
            0xAA => {
                self.channels[2].frequency = (self.channels[2].frequency & 0x00FF) | ((data as u16 & 0x03) << 8);
                self.channels[2].octave = (data >> 2) & 0x07;
            }
            0xAC => {
                self.channels[3].frequency = (self.channels[3].frequency & 0x00FF) | ((data as u16 & 0x03) << 8);
                self.channels[3].octave = (data >> 2) & 0x07;
            }
            0xAD => {
                self.channels[4].frequency = (self.channels[4].frequency & 0x00FF) | ((data as u16 & 0x03) << 8);
                self.channels[4].octave = (data >> 2) & 0x07;
            }
            0xAE => {
                self.channels[5].frequency = (self.channels[5].frequency & 0x00FF) | ((data as u16 & 0x03) << 8);
                self.channels[5].octave = (data >> 2) & 0x07;
            }

            // === Key On/Off (0x28) ===
            // Real YM2612: register 0x28 in either port encodes the target channel in data bits.
            // data[3:2] = part (0 or 1), data[1:0] = channel-within-part (0-2),
            // data[7:4] = per-operator key-on mask (0xF0 = all on, 0x00 = all off).
            0x28 => {
                let ch_in_part = (data & 0x03) as usize;
                let ch_part = ((data >> 2) & 0x01) as usize;
                let channel = ch_part * 3 + ch_in_part;
                if channel < 6 && ch_in_part < 3 {
                    let key_on = (data & 0xF0) != 0;
                    self.channels[channel].key_on = key_on;
                    for op in &mut self.channels[channel].operators {
                        op.key_on = key_on;
                        if key_on {
                            op.envelope_state = EnvelopeState::Attack;
                            op.envelope_level = 127;
                        }
                    }
                }
            }

            // === Channel Algorithm/Feedback (0xB0-0xB6) ===
            0xB0 => {
                self.channels[0].algorithm = data & 0x07;
                self.channels[0].feedback = (data >> 3) & 0x07;
            }
            0xB1 => {
                self.channels[1].algorithm = data & 0x07;
                self.channels[1].feedback = (data >> 3) & 0x07;
            }
            0xB2 => {
                self.channels[2].algorithm = data & 0x07;
                self.channels[2].feedback = (data >> 3) & 0x07;
            }
            0xB4 => {
                self.channels[3].algorithm = data & 0x07;
                self.channels[3].feedback = (data >> 3) & 0x07;
            }
            0xB5 => {
                self.channels[4].algorithm = data & 0x07;
                self.channels[4].feedback = (data >> 3) & 0x07;
            }
            0xB6 => {
                self.channels[5].algorithm = data & 0x07;
                self.channels[5].feedback = (data >> 3) & 0x07;
            }

            // Unimplemented register - ignore
            _ => {
                // Placeholder for future operator register handling
            }
        }
    }
}

impl SoundChipEmulator for YM2612 {
    fn name(&self) -> &'static str {
        "YM2612 (OPN2)"
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn reset(&mut self) {
        *self = Self::with_clock_rate(self.clock_rate);
    }

    fn write(&mut self, addr: u8, data: u8) {
        self.write_port(0, addr, data);
    }

    fn write_port(&mut self, port: u8, addr: u8, data: u8) {
        // Port 0 (VGM 0x52) → channels 0-2; port 1 (VGM 0x53) → channels 3-5.
        let ch_base = port as usize * 3;

        match addr {
            // F-Num LSB: 0xA0+n → channel n within this port
            0xA0..=0xA2 => {
                let ch = ch_base + (addr - 0xA0) as usize;
                if ch < 6 {
                    self.channels[ch].frequency = (self.channels[ch].frequency & 0xFF00) | (data as u16);
                }
            }
            // Block + F-Num MSB: 0xA4+n → channel n within this port
            // data = [_, block2, block1, block0, f9, f8, 0, 0]
            // VGM codegen: msb = (block << 3) | ((f_num >> 8) & 0x7)
            0xA4..=0xA6 => {
                let ch = ch_base + (addr - 0xA4) as usize;
                if ch < 6 {
                    self.channels[ch].octave = (data >> 3) & 0x07;
                    let f_high = (data & 0x07) as u16;
                    self.channels[ch].frequency = (f_high << 8) | (self.channels[ch].frequency & 0xFF);
                }
            }
            // Algorithm/Feedback: 0xB0+n → channel n within this port
            0xB0..=0xB2 => {
                let ch = ch_base + (addr - 0xB0) as usize;
                if ch < 6 {
                    self.channels[ch].algorithm = data & 0x07;
                    self.channels[ch].feedback = (data >> 3) & 0x07;
                }
            }
            // Stereo/LFO sensitivity: 0xB4+n → channel n within this port
            0xB4..=0xB6 => {
                let ch = ch_base + (addr - 0xB4) as usize;
                if ch < 6 {
                    self.channels[ch].left_enable = (data & 0x80) != 0;
                    self.channels[ch].right_enable = (data & 0x40) != 0;
                }
            }
            // All other registers go through the generic handler
            _ => {
                self.write_reg(addr, data);
            }
        }
    }

    fn read(&self, addr: u8) -> u8 {
        // YM2612 status register (0x00-0x03 are status)
        // For now, return 0
        0
    }

    fn clock(&mut self) {
        // Update LFO
        self.update_lfo();

        // Update timers
        self.update_timers();

        // Update all channels and operators
        for channel in &mut self.channels {
            // Compute per-clock phase increment for each operator from channel frequency/octave.
            // Formula: f_num * 2^(11+block) / 144, scaled by operator ML.
            // This gives the correct increment for the 32-bit accumulator used in phase advance.
            let f_num = channel.frequency as u64;
            let block = channel.octave.min(7) as u32;
            let base_inc = if f_num > 0 {
                (f_num << (11 + block)) / 144
            } else {
                0
            };
            for op in &mut channel.operators {
                let ml: u64 = if op.multiple == 0 { 1 } else { op.multiple as u64 * 2 };
                op.phase_increment = ((base_inc * ml) / 2) as u32;
            }

            // Update channel key state first
            if channel.key_on {
                for op in &mut channel.operators {
                    if op.envelope_state == EnvelopeState::Off {
                        op.envelope_state = EnvelopeState::Attack;
                        op.envelope_level = 127;
                    }
                }
            }

            // Update operators
            for op in &mut channel.operators {
                // Update phase accumulator
                op.phase = op.phase.wrapping_add(op.phase_increment);
                
                // Update envelope
                match op.envelope_state {
                    EnvelopeState::Off => {
                        op.envelope_level = 127;
                    }
                    EnvelopeState::Attack => {
                        if op.attack_rate > 0 {
                            op.envelope_level = op.envelope_level.saturating_sub(1);
                            if op.envelope_level == 0 {
                                op.envelope_state = EnvelopeState::Decay;
                            }
                        }
                    }
                    EnvelopeState::Decay => {
                        if op.decay_rate > 0 {
                            if op.envelope_level < op.sustain_level {
                                op.envelope_state = EnvelopeState::Sustain;
                            } else {
                                op.envelope_level = op.envelope_level.saturating_add(1);
                            }
                        }
                    }
                    EnvelopeState::Sustain => {
                        // Stay at sustain level
                    }
                    EnvelopeState::Release => {
                        if op.release_rate > 0 {
                            op.envelope_level = op.envelope_level.saturating_add(1);
                            if op.envelope_level >= 127 {
                                op.envelope_state = EnvelopeState::Off;
                                op.key_on = false;
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        // Set sample rate if changed
        if sample_rate != self.sample_rate {
            self.set_sample_rate(sample_rate);
        }

        // Generate samples for each frame
        for frame in buffer.chunks_mut(2) {
            // Accumulate clock cycles for this sample
            self.accumulated_cycles += self.clock_divider;

            // Clock the chip for the accumulated cycles
            while self.accumulated_cycles >= 1.0 {
                self.clock();
                self.accumulated_cycles -= 1.0;
            }

            // Get output for this sample
            let (left, right) = self.get_output();
            frame[0] = left;
            frame[1] = right;
        }
    }
}

impl Default for YM2612 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ym2612_new() {
        let chip = YM2612::new();
        assert_eq!(chip.name(), "YM2612 (OPN2)");
        assert_eq!(chip.clock_rate(), 7_670_453);
    }

    #[test]
    fn test_ym2612_reset() {
        let mut chip = YM2612::new();
        chip.write(0x28, 0x01); // Key on channel 0
        chip.reset();
        assert!(!chip.channels[0].key_on);
    }

    #[test]
    fn test_ym2612_write_key_on() {
        let mut chip = YM2612::new();
        chip.write(0x28, 0xF0); // Key on channel 0 (part I)
        assert!(chip.channels[0].key_on);
    }

    #[test]
    fn test_ym2612_write_frequency() {
        let mut chip = YM2612::new();

        // Port 0 (VGM 0x52): channels 0-2
        // write_port(port, 0xA0+ch, f_num_low), write_port(port, 0xA4+ch, block<<3 | f_num_high)
        // Channel 0: F-num=0x283=643, block=4
        // msb = (4<<3) | (643>>8 & 7) = 32 | 2 = 0x22
        chip.write_port(0, 0xA4, 0x22); // ch0 block+f_high: block=(0x22>>3)&7=4, f_high=0x22&7=2
        chip.write_port(0, 0xA0, 0x83); // ch0 f_num low = 0x83
        assert_eq!(chip.channels[0].frequency, 0x0283);
        assert_eq!(chip.channels[0].octave, 4);

        // Channel 1: F-num=0x2A8, block=4
        chip.write_port(0, 0xA5, 0x25); // ch1: block=(0x25>>3)&7=4, f_high=0x25&7=5
        chip.write_port(0, 0xA1, 0xA8); // ch1 f_num low = 0xA8
        assert_eq!(chip.channels[1].frequency, (5 << 8) | 0xA8);
        assert_eq!(chip.channels[1].octave, 4);

        // Port 1 (VGM 0x53): channels 3-5
        // Channel 3: F-num=0x1AB, block=3
        chip.write_port(1, 0xA4, 0x1D); // ch3: block=(0x1D>>3)&7=3, f_high=0x1D&7=5
        chip.write_port(1, 0xA0, 0xAB); // ch3 f_num low = 0xAB
        assert_eq!(chip.channels[3].frequency, (5 << 8) | 0xAB);
        assert_eq!(chip.channels[3].octave, 3);
    }

    #[test]
    fn test_ym2612_soundchip_trait() {
        let mut chip = YM2612::new();

        // Verify trait methods work
        assert_eq!(chip.name(), "YM2612 (OPN2)");
        assert_eq!(chip.clock_rate(), 7_670_453);

        chip.reset();
        chip.write(0x28, 0xF0);
        chip.clock();

        let mut buffer = [0.0f32; 2];
        chip.generate_samples(&mut buffer, 44100);
    }

    #[test]
    fn test_ym2612_produces_sound_on_key_on() {
        let mut chip = YM2612::new();
        // C4: F-num=0x283, block=4
        chip.write_port(0, 0xA4, 0x22); // block=4, f_high=2
        chip.write_port(0, 0xA0, 0x83); // f_low=0x83
        chip.write_port(0, 0xB4, 0xC0); // stereo enable
        chip.write_reg(0x28, 0xF0);     // key-on ch0, all operators
        // Generate enough samples for envelope attack to complete
        let mut buffer = vec![0.0f32; 44100 * 2]; // 1 second stereo
        chip.generate_samples(&mut buffer, 44100);
        let any_nonzero = buffer.iter().any(|&s| s != 0.0);
        assert!(any_nonzero, "YM2612 should produce non-zero output after key-on");
    }
}
