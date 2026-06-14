/// MIDI Controller numbers (CC)
pub mod midi_cc {
    /// MOD WHEEL.
    pub const MOD_WHEEL: u8 = 1; // Modulation Wheel
    /// BREATH.
    pub const BREATH: u8 = 2; // Breath Controller
    /// FOOT PEDAL.
    pub const FOOT_PEDAL: u8 = 4; // Foot Pedal
    /// PORTAMENTO TIME.
    pub const PORTAMENTO_TIME: u8 = 5; // Portamento Time
    /// VOLUME.
    pub const VOLUME: u8 = 7; // Main Volume
    /// BALANCE.
    pub const BALANCE: u8 = 8; // Balance
    /// PAN.
    pub const PAN: u8 = 10; // Pan
    /// EXPRESSION.
    pub const EXPRESSION: u8 = 11; // Expression Controller
    /// EFFECT CONTROL 1.
    pub const EFFECT_CONTROL_1: u8 = 12; // Effect Control 1
    /// EFFECT CONTROL 2.
    pub const EFFECT_CONTROL_2: u8 = 13; // Effect Control 2
    /// GENERAL PURPOSE SLIDER 1.
    pub const GENERAL_PURPOSE_SLIDER_1: u8 = 16; // General Purpose Slider 1
    /// GENERAL PURPOSE SLIDER 2.
    pub const GENERAL_PURPOSE_SLIDER_2: u8 = 17; // General Purpose Slider 2
    /// GENERAL PURPOSE SLIDER 3.
    pub const GENERAL_PURPOSE_SLIDER_3: u8 = 18; // General Purpose Slider 3
    /// GENERAL PURPOSE SLIDER 4.
    pub const GENERAL_PURPOSE_SLIDER_4: u8 = 19; // General Purpose Slider 4
}

/// Chip-specific CC mappings for synthesizer parameters
pub struct ChipCCMapping {
    /// Modulation wheel (CC1) control
    pub mod_wheel_target: ModWheelTarget,
    /// Filter cutoff control (CC12 or EFFECT_CONTROL_1)
    pub filter_control: Option<u8>,
    /// Resonance control (CC13 or EFFECT_CONTROL_2)
    pub resonance_control: Option<u8>,
    /// Pitch bend range in semitones
    pub pitch_bend_range: u8,
    /// Supports channel aftertouch
    pub supports_aftertouch: bool,
    /// Supports poly aftertouch
    pub supports_poly_aftertouch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Mod Wheel Target.
pub enum ModWheelTarget {
    /// Vibrato.
    Vibrato,
    /// Tremolo.
    Tremolo,
    /// Brightness.
    Brightness,
    /// Filter Cutoff.
    FilterCutoff,
    /// None.
    None,
}

impl ChipCCMapping {
    /// Get mapping for a specific chip
    pub fn for_chip(chip: &str) -> Self {
        match chip.to_uppercase().as_str() {
            // FM Chips: Modulation to Vibrato LFO
            "YM2608" | "OPNA" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: true,
                supports_poly_aftertouch: false,
            },
            "YM2151" | "OPM" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: true,
                supports_poly_aftertouch: true,
            },
            "YM2203" | "OPN" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: true,
                supports_poly_aftertouch: false,
            },
            "YM2413" | "OPLL" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 1,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            // OPL Chips: Modulation to Brightness
            "YM3526" | "OPL" | "YM3812" | "OPL2" | "YMF262" | "OPL3" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Brightness,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: true,
                supports_poly_aftertouch: false,
            },
            "Y8950" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: Some(midi_cc::EFFECT_CONTROL_2),
                pitch_bend_range: 2,
                supports_aftertouch: true,
                supports_poly_aftertouch: false,
            },
            // PSG Chips: Modulation to Filter
            "AY8910" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::FilterCutoff,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: Some(midi_cc::EFFECT_CONTROL_2),
                pitch_bend_range: 1,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "POKEY" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::FilterCutoff,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: Some(midi_cc::EFFECT_CONTROL_2),
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            // Wavetable Chips: Modulation to Vibrato
            "HUC6280" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "K051649" | "SCC" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "K053260" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Brightness,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            // PCM Chips: Modulation to Filter/Volume
            "SEGAPCM" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::FilterCutoff,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "RF5C164" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "C140" | "C352" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Brightness,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "K054539" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::FilterCutoff,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: Some(midi_cc::EFFECT_CONTROL_2),
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "QSOUND" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::FilterCutoff,
                filter_control: Some(midi_cc::EFFECT_CONTROL_1),
                resonance_control: Some(midi_cc::EFFECT_CONTROL_2),
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            // Console chips
            "DMG" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 1,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "NES_APU" | "NESAPU" | "2A03" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            "VRC6" => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: false,
                supports_poly_aftertouch: false,
            },
            // Default mapping
            _ => ChipCCMapping {
                mod_wheel_target: ModWheelTarget::Vibrato,
                filter_control: None,
                resonance_control: None,
                pitch_bend_range: 2,
                supports_aftertouch: true,
                supports_poly_aftertouch: false,
            },
        }
    }

    /// Get the standard pitch bend range for a chip
    pub fn get_pitch_bend_semitones(chip: &str) -> u8 {
        Self::for_chip(chip).pitch_bend_range
    }

    /// Check if chip supports a specific MIDI feature
    pub fn supports_feature(chip: &str, feature: MidiFeature) -> bool {
        let mapping = Self::for_chip(chip);
        match feature {
            MidiFeature::Aftertouch => mapping.supports_aftertouch,
            MidiFeature::PolyAftertouch => mapping.supports_poly_aftertouch,
            MidiFeature::PitchBend => true, // All chips support pitch bend
            MidiFeature::ModulationWheel => mapping.mod_wheel_target != ModWheelTarget::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Midi Feature.
pub enum MidiFeature {
    /// Aftertouch.
    Aftertouch,
    /// Poly Aftertouch.
    PolyAftertouch,
    /// Pitch Bend.
    PitchBend,
    /// Modulation Wheel.
    ModulationWheel,
}

/// Channel Aftertouch to parameter mapping
pub fn map_channel_aftertouch(chip: &str, aftertouch_value: u8) -> Vec<(u8, u8)> {
    // Map aftertouch value (0-127) to chip-specific parameters
    match chip.to_uppercase().as_str() {
        "YM2151" | "OPM" => {
            // OPM: Aftertouch to Total Level (modulation)
            vec![(0x60, (aftertouch_value >> 1) & 0x7F)]
        }
        _ => vec![],
    }
}

/// Pitch bend value mapping (14-bit MIDI value to chip-specific register values)
pub fn map_pitch_bend(chip: &str, bend_value: u16) -> Option<(u8, u8)> {
    // MIDI pitch bend is 14-bit (0x0000 = -2 semitones, 0x2000 = center, 0x3FFF = +2 semitones)
    // Calculate the amount of bend in cents (0-200 cents = 0-2 semitones by default)

    let center = 0x2000u16;
    let _cents = if bend_value > center {
        ((bend_value - center) as u32 * 200) / (0x2000u32)
    } else {
        -((center - bend_value) as i32 * 200 / 0x2000) as u32
    };

    // Per-chip pitch bend register mapping is not yet implemented.
    let _ = chip;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fm_chip_mapping() {
        let mapping = ChipCCMapping::for_chip("YM2608");
        assert_eq!(mapping.mod_wheel_target, ModWheelTarget::Vibrato);
        assert_eq!(mapping.pitch_bend_range, 2);
        assert!(mapping.supports_aftertouch);
    }

    #[test]
    fn test_psg_chip_mapping() {
        let mapping = ChipCCMapping::for_chip("AY8910");
        assert_eq!(mapping.mod_wheel_target, ModWheelTarget::FilterCutoff);
        assert!(mapping.filter_control.is_some());
    }

    #[test]
    fn test_pitch_bend_range() {
        assert_eq!(ChipCCMapping::get_pitch_bend_semitones("YM2608"), 2);
        assert_eq!(ChipCCMapping::get_pitch_bend_semitones("YM2413"), 1);
    }
}
