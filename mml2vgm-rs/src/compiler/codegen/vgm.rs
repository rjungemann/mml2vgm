//! VGM Format Generator
//!
//! This module generates VGM (Video Game Music) format files from MML AST.

use super::{CodeGenerator, OutputFormat, VgmHeader, NoteEvent, SourceMap};
use crate::compiler::ast::{MmlAst, MmlNode, OctaveShift};
use crate::compiler::sample_resolver::SampleResolver;
use crate::{CompileOptions, MmlError, MmlResult, SoundChip};
use std::collections::{BTreeSet, HashMap};

/// VGM command types (values match the VGM specification)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VgmCommandType {
    Sn76489Write = 0x50,
    Ym2413Write = 0x51,
    Ym2612WritePort0 = 0x52,
    Ym2612WritePort1 = 0x53,
    Ym2151Write = 0x54,
    Ym2203Write = 0x55,
    Ym2608WritePort0 = 0x56,
    Ym2608WritePort1 = 0x57,
    Ym2610WritePort0 = 0x58,
    Ym2610WritePort1 = 0x59,
    Ym3812Write = 0x5A,
    Ym3526Write = 0x5B,
    Y8950Write = 0x5C,
    Ymf262WritePort0 = 0x5E,
    Ymf262WritePort1 = 0x5F,
    // PCM chips
    Rf5c164Write = 0x68,
    C140Write = 0x7F,
    C352Write = 0x8E,
    // PSG/Arcade chips
    Ay8910Write = 0xA0,
    SegaPcmWrite = 0xA4,
    // Console chips
    DmgWrite = 0xB3,
    NesApuWrite = 0xB4,
    Vrc6Write = 0xB6,
    HuC6280Write = 0xB9,
    K053260Write = 0xBA,
    PokeyWrite = 0xBB,
    QSoundWrite = 0xC4,
    K051649Write = 0xD2,
    K054539Write = 0xD3,
    // Timing/control
    Wait = 0x61,
    Wait1 = 0x62,
    Wait2 = 0x63,
    End = 0x66,
    DataBlock = 0x67,
}

/// A single VGM command
#[derive(Debug, Clone)]
pub struct VgmCommand {
    pub command_type: VgmCommandType,
    pub data: Vec<u8>,
    pub time: u64,
}

/// PCM data for embedding in VGM
#[derive(Debug, Clone)]
pub struct PcmData {
    pub data: Vec<u8>,
    pub start_offset: u32,
}

/// GD3 tag for metadata
#[derive(Debug, Clone)]
pub struct Gd3Tag {
    pub track_name_en: String,
    pub track_name_jp: String,
    pub game_name_en: String,
    pub game_name_jp: String,
    pub system_name_en: String,
    pub system_name_jp: String,
    pub author_en: String,
    pub author_jp: String,
    pub release_date: String,
    pub converter: String,
    pub notes: String,
}

impl Default for Gd3Tag {
    fn default() -> Self {
        Self {
            track_name_en: String::new(),
            track_name_jp: String::new(),
            game_name_en: String::new(),
            game_name_jp: String::new(),
            system_name_en: String::new(),
            system_name_jp: String::new(),
            author_en: String::new(),
            author_jp: String::new(),
            release_date: String::new(),
            converter: String::new(),
            notes: String::new(),
        }
    }
}

/// Per-part state during VGM code generation
struct PartCodegenState {
    /// Chip name for this part (e.g. "YM2612", "SN76489")
    chip: Option<String>,
    /// YM2612/YM2608 port (0 = channels 0-2, 1 = channels 3-5)
    ym2612_port: u8,
    /// YM2612/YM2608/YM2203 channel within the port (0-2)
    ym2612_ch: u8,
    /// OPL channel (0-8 for YM3812/YM3526/Y8950, 0-17 for YMF262)
    opl_ch: u8,
    /// OPM channel (0-7 for YM2151)
    opm_ch: u8,
    /// Tempo in BPM
    tempo: u32,
    /// Current octave (0-8)
    octave: u8,
    /// Current default note length denominator (4 = quarter note)
    length: u32,
    /// Current volume (0-127)
    volume: u8,
    /// Selected FM instrument number
    instrument_num: Option<u32>,
    /// Whether this part has a real hardware channel assigned (false for parts beyond max channels)
    has_channel: bool,
    /// Whether the F-type operator registers (DT/ML, KS/AR, etc.) have been written
    init_done: bool,
    /// Whether a key-on is in effect
    keyed_on: bool,
    /// Quantize/gate value
    quantize: u8,
    /// true = uppercase Q (proportional: note plays value/8 of duration)
    /// false = lowercase q (absolute: silence = value/48 of duration)
    quantize_proportional: bool,
    /// Last-written TL per hardware operator (indexed by hw_op after MML→hw swap).
    /// Initialized to 127 to reflect the global init (OutFmAllKeyOff) that mutes all channels.
    /// Matches C# page.beforeTL optimization to skip redundant TL writes.
    before_tl: [i16; 4],
    /// When true, key-off is suppressed (C# page.envelopeMode). Set by EON command.
    eon_mode: bool,
    /// Console chip channel numbers
    k051649_ch: u8,
    nes_ch: u8,
    dmg_ch: u8,
    vrc6_ch: u8,
}

impl PartCodegenState {
    fn new(chip: Option<String>, ym2612_port: u8, ym2612_ch: u8) -> Self {
        Self {
            chip,
            ym2612_port,
            ym2612_ch,
            opl_ch: 0,
            opm_ch: 0,
            tempo: 120,
            octave: 4,
            length: 4,
            volume: 127,
            instrument_num: None,
            has_channel: true,
            init_done: false,
            keyed_on: false,
            quantize: 0,
            quantize_proportional: false,
            before_tl: [127; 4],
            eon_mode: false,
            k051649_ch: 0,
            nes_ch: 0,
            dmg_ch: 0,
            vrc6_ch: 0,
        }
    }
}

/// VGM file generator
pub struct VgmGenerator {
    header: VgmHeader,
    commands: Vec<VgmCommand>,
    chips: Vec<SoundChip>,
    pcm_data: Vec<PcmData>,
    gd3_tag: Option<Gd3Tag>,
    /// FM instrument flat-parameter tables indexed by instrument number
    fm_instruments: HashMap<u32, Vec<u32>>,
    /// Next YM2612 absolute channel to allocate (0-5)
    next_ym2612_channel: u8,
    /// Next YM2608 absolute channel to allocate (0-5)
    next_ym2608_channel: u8,
    /// Next YM2203 channel to allocate (0-2)
    next_ym2203_channel: u8,
    /// Next YM2151 (OPM) channel to allocate (0-7)
    next_opm_channel: u8,
    /// Next OPL channel to allocate (0-8, shared across YM3812/YM3526/Y8950)
    next_opl_channel: u8,
    /// Next YMF262 (OPL3) channel to allocate (0-17)
    next_ymf262_channel: u8,
    /// Next K051649 channel to allocate (0-4)
    next_k051649_channel: u8,
    /// Bitmask of currently-keyed-on K051649 channels (register 0xAF)
    k051649_key_mask: u8,
    /// Next NES APU channel to allocate (0-4: Pulse1, Pulse2, Triangle, Noise, DPCM)
    next_nes_channel: u8,
    /// Next DMG channel to allocate (0-3: Pulse1, Pulse2, Wave, Noise)
    next_dmg_channel: u8,
    /// Next VRC6 channel to allocate (0-2: Pulse1, Pulse2, Sawtooth)
    next_vrc6_channel: u8,
    /// Next YM2413 (OPLL) channel to allocate (0-8)
    next_ym2413_channel: u8,
    /// Next AY8910 channel to allocate (0-2: A, B, C)
    next_ay8910_channel: u8,
    /// Next HuC6280 channel to allocate (0-5)
    next_huc6280_channel: u8,
    /// Next RF5C164 channel to allocate (0-7)
    next_rf5c164_channel: u8,
    /// Next K053260 channel to allocate (0-7)
    next_k053260_channel: u8,
    /// Next K054539 channel to allocate (0-31)
    next_k054539_channel: u8,
    /// Next SegaPCM channel to allocate (0-15)
    next_segapcm_channel: u8,
    /// Next C140 channel to allocate (0-23)
    next_c140_channel: u8,
    /// Next C352 channel to allocate (0-31)
    next_c352_channel: u8,
    /// Next QSound channel to allocate (0-15)
    next_qsound_channel: u8,
    /// When true, add_wait is a no-op (used during parallel part processing)
    suppress_waits: bool,
    /// Time boundaries recorded by add_wait calls (even when suppressed).
    /// Used in the merge phase to split large wait gaps at per-event boundaries,
    /// matching the C# compiler's one-wait-per-note/rest output style.
    time_checkpoints: BTreeSet<u64>,
    /// Source map: accumulates note events with timing and source positions
    source_map: SourceMap,
    /// Current part name being processed
    current_part_name: String,
}

impl VgmGenerator {
    /// Create a new VGM generator from an AST
    pub fn from_ast(ast: &MmlAst, options: &CompileOptions) -> MmlResult<Self> {
        let mut generator = Self {
            header: VgmHeader::default(),
            commands: Vec::new(),
            chips: Vec::new(),
            pcm_data: Vec::new(),
            gd3_tag: None,
            fm_instruments: HashMap::new(),
            next_ym2612_channel: 0,
            next_ym2608_channel: 0,
            next_ym2203_channel: 0,
            next_opm_channel: 0,
            next_opl_channel: 0,
            next_ymf262_channel: 0,
            next_k051649_channel: 0,
            k051649_key_mask: 0,
            next_nes_channel: 0,
            next_dmg_channel: 0,
            next_vrc6_channel: 0,
            next_ym2413_channel: 0,
            next_ay8910_channel: 0,
            next_huc6280_channel: 0,
            next_rf5c164_channel: 0,
            next_k053260_channel: 0,
            next_k054539_channel: 0,
            next_segapcm_channel: 0,
            next_c140_channel: 0,
            next_c352_channel: 0,
            next_qsound_channel: 0,
            suppress_waits: false,
            time_checkpoints: BTreeSet::new(),
            source_map: SourceMap::default(),
            current_part_name: String::new(),
        };

        generator.header.version = 0x00000171;
        generator.extract_chips(ast, options);

        // Store FM instrument parameters from the AST
        for (num, inst) in &ast.fm_instruments {
            generator.fm_instruments.insert(*num, inst.parameters.clone());
        }
        generator.convert_ast_to_commands(ast)?;
        generator.build_gd3_tag(ast);
        generator.calculate_header();

        Ok(generator)
    }

    /// Same as `from_ast` but additionally loads PCM sample data for every
    /// `'@ P` instrument found in the AST.  Samples are fetched via the
    /// supplied `resolver`; instruments with no matching sample are silently
    /// skipped (the VGM will still compile, just without that sample embedded).
    pub fn from_ast_with_resolver(
        ast: &MmlAst,
        options: &CompileOptions,
        resolver: &dyn SampleResolver,
    ) -> MmlResult<Self> {
        let mut generator = Self::from_ast(ast, options)?;
        generator.load_pcm_instruments(ast, resolver);
        Ok(generator)
    }

    /// Populate `pcm_data` from PCM instruments in the AST using the resolver.
    fn load_pcm_instruments(&mut self, ast: &MmlAst, resolver: &dyn SampleResolver) {
        for inst in ast.pcm_instruments.values() {
            // Try the bare filename first, then the full path as a fallback.
            let name = inst
                .filename
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(inst.name.as_str());

            if let Some(f32_samples) = resolver.resolve(name) {
                // Convert f32 [-1.0, +1.0] → u8 [0, 255] (RF5C164 8-bit unsigned,
                // 0x80 = silence).  Clamp first to avoid wrapping artefacts.
                let raw: Vec<u8> = f32_samples
                    .iter()
                    .map(|&s| {
                        let clamped = s.clamp(-1.0, 1.0);
                        ((clamped * 127.0) + 128.0).round() as u8
                    })
                    .collect();

                self.pcm_data.push(PcmData {
                    data: raw,
                    start_offset: 0,
                });
            }
        }
    }

    fn extract_chips(&mut self, ast: &MmlAst, options: &CompileOptions) {
        if let Some(ref chips) = options.target_chips {
            for chip in chips {
                if !self.chips.contains(chip) {
                    self.chips.push(*chip);
                }
            }
        }

        for part in ast.parts.values() {
            if let Some(ref chip_str) = part.chip {
                let chip = match chip_str.to_uppercase().as_str() {
                    "YM2612" => SoundChip::YM2612,
                    "SN76489" => SoundChip::SN76489,
                    "YM2151" | "OPM" => SoundChip::YM2151,
                    "YM2413" | "OPLL" => SoundChip::YM2413,
                    "YM2608" | "OPNA" => SoundChip::YM2608,
                    "YM2203" | "OPN" => SoundChip::YM2203,
                    "YM3812" | "OPL2" => SoundChip::YM3812,
                    "YM3526" | "OPL" => SoundChip::YM3526,
                    "Y8950" => SoundChip::Y8950,
                    "YMF262" | "OPL3" => SoundChip::YMF262,
                    "K051649" | "SCC" | "SCC1" => SoundChip::K051649,
                    "NES" | "NESAPU" | "2A03" => SoundChip::NES,
                    "DMG" | "GAMEBOY" | "GAME BOY" => SoundChip::DMG,
                    "RF5C164" => SoundChip::RF5C164,
                    "SEGAPCM" => SoundChip::SegaPCM,
                    "C140" => SoundChip::C140,
                    "C352" => SoundChip::C352,
                    "AY8910" => SoundChip::AY8910,
                    "HUC6280" => SoundChip::HuC6280,
                    "POKEY" => SoundChip::POKEY,
                    "VRC6" => SoundChip::VRC6,
                    "K053260" => SoundChip::K053260,
                    "K054539" => SoundChip::K054539,
                    "QSOUND" => SoundChip::QSound,
                    _ => continue,
                };
                if !self.chips.contains(&chip) {
                    self.chips.push(chip);
                }
            }
        }

        // Also check for metadata keys like PartK051649, PartNES, PartDMG, and all 21 partial chips
        for (key, _) in &ast.metadata {
            let chip = match key.to_uppercase().as_str() {
                "PARTK051649" | "PARTSCC" | "PARTSCC1" => SoundChip::K051649,
                "PARTNES" | "PARTNESAPU" | "PART2A03" => SoundChip::NES,
                "PARTDMG" | "PARTGAMEBOY" => SoundChip::DMG,
                // Batch 1: Sega & FM Core
                "PARTYM2608" | "PARTOPNA" => SoundChip::YM2608,
                "PARTYM2151" | "PARTOPM" => SoundChip::YM2151,
                "PARTYM2203" | "PARTOPN" => SoundChip::YM2203,
                "PARTRF5C164" => SoundChip::RF5C164,
                "PARTSEGAPCM" => SoundChip::SegaPCM,
                // Batch 2: OPL Family
                "PARTYM3526" | "PARTOPL" => SoundChip::YM3526,
                "PARTY8950" => SoundChip::Y8950,
                "PARTYM3812" | "PARTOPL2" => SoundChip::YM3812,
                "PARTYMF262" | "PARTOPL3" => SoundChip::YMF262,
                // Batch 3: Console PSG/FM
                "PARTYM2413" | "PARTOPLL" => SoundChip::YM2413,
                "PARTHUC6280" => SoundChip::HuC6280,
                // Batch 4: Arcade PCM
                "PARTC140" => SoundChip::C140,
                "PARTC352" => SoundChip::C352,
                "PARTK053260" => SoundChip::K053260,
                "PARTK054539" => SoundChip::K054539,
                // Batch 5: Miscellaneous
                "PARTAY8910" => SoundChip::AY8910,
                "PARTPOKEY" => SoundChip::POKEY,
                "PARTVRC6" => SoundChip::VRC6,
                "PARTQSOUND" => SoundChip::QSound,
                _ => continue,
            };
            if !self.chips.contains(&chip) {
                self.chips.push(chip);
            }
        }

        // Global #CHIP directive — applies when no per-part or PartXxx chip was found
        if self.chips.is_empty() {
            if let Some(global_chip) = ast.metadata.get("CHIP") {
                let chip = match global_chip.to_uppercase().as_str() {
                    "YM2612" => Some(SoundChip::YM2612),
                    "SN76489" => Some(SoundChip::SN76489),
                    "YM2151" | "OPM" => Some(SoundChip::YM2151),
                    "YM2413" | "OPLL" => Some(SoundChip::YM2413),
                    "YM2608" | "OPNA" => Some(SoundChip::YM2608),
                    "YM2203" | "OPN" => Some(SoundChip::YM2203),
                    "YM3812" | "OPL2" => Some(SoundChip::YM3812),
                    "YM3526" | "OPL" => Some(SoundChip::YM3526),
                    "Y8950" => Some(SoundChip::Y8950),
                    "YMF262" | "OPL3" => Some(SoundChip::YMF262),
                    "K051649" | "SCC" | "SCC1" => Some(SoundChip::K051649),
                    "NES" | "NESAPU" | "2A03" => Some(SoundChip::NES),
                    "DMG" | "GAMEBOY" | "GAME BOY" => Some(SoundChip::DMG),
                    "RF5C164" => Some(SoundChip::RF5C164),
                    "SEGAPCM" => Some(SoundChip::SegaPCM),
                    "C140" => Some(SoundChip::C140),
                    "C352" => Some(SoundChip::C352),
                    "AY8910" => Some(SoundChip::AY8910),
                    "HUC6280" => Some(SoundChip::HuC6280),
                    "POKEY" => Some(SoundChip::POKEY),
                    "VRC6" => Some(SoundChip::VRC6),
                    "K053260" => Some(SoundChip::K053260),
                    "K054539" => Some(SoundChip::K054539),
                    "QSOUND" => Some(SoundChip::QSound),
                    _ => None,
                };
                if let Some(c) = chip {
                    self.chips.push(c);
                }
            }
        }
        if self.chips.is_empty() {
            self.chips = vec![SoundChip::YM2612, SoundChip::SN76489];
        }

        for chip in &self.chips {
            match chip {
                SoundChip::SN76489 | SoundChip::SN76489X2 => {
                    self.header.sn76489_clock = chip.clock_rate();
                }
                SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2 => {
                    self.header.ym2612_clock = chip.clock_rate();
                }
                SoundChip::YM2151 => {
                    self.header.ym2151_clock = chip.clock_rate();
                }
                SoundChip::YM2413 => {
                    self.header.ym2413_clock = chip.clock_rate();
                }
                SoundChip::YM2608 => {
                    self.header.ym2608_clock = chip.clock_rate();
                }
                SoundChip::YM2203 => {
                    self.header.ym2203_clock = chip.clock_rate();
                }
                SoundChip::YM3812 => {
                    self.header.ym3812_clock = chip.clock_rate();
                }
                SoundChip::YM3526 => {
                    self.header.ym3526_clock = chip.clock_rate();
                }
                SoundChip::Y8950 => {
                    self.header.y8950_clock = chip.clock_rate();
                }
                SoundChip::YMF262 => {
                    self.header.ymf262_clock = chip.clock_rate();
                }
                SoundChip::K051649 => {
                    self.header.k051649_clock = chip.clock_rate();
                    self.header.k051649_flags |= 0x80000000; // Bit 31: K051649 present
                }
                SoundChip::NES => {
                    self.header.nes_apu_clock = chip.clock_rate();
                }
                SoundChip::DMG => {
                    self.header.dmg_clock = chip.clock_rate();
                }
                // Phase 1 extensions: All 21 partial chips
                SoundChip::RF5C164 => {
                    self.header.rf5c164_clock = chip.clock_rate();
                }
                SoundChip::SegaPCM => {
                    self.header.segapcm_clock = chip.clock_rate();
                }
                SoundChip::C140 => {
                    self.header.c140_clock = chip.clock_rate();
                }
                SoundChip::C352 => {
                    self.header.c352_clock = chip.clock_rate();
                }
                SoundChip::AY8910 => {
                    self.header.ay8910_clock = chip.clock_rate();
                }
                SoundChip::HuC6280 => {
                    self.header.huc6280_clock = chip.clock_rate();
                }
                SoundChip::POKEY => {
                    self.header.pokey_clock = chip.clock_rate();
                }
                SoundChip::VRC6 => {
                    self.header.vrc6_clock = chip.clock_rate();
                }
                SoundChip::K053260 => {
                    self.header.k053260_clock = chip.clock_rate();
                }
                SoundChip::K054539 => {
                    self.header.k054539_clock = chip.clock_rate();
                }
                SoundChip::QSound => {
                    self.header.qsound_clock = chip.clock_rate();
                }
                _ => {}
            }
        }

        // Apply #CLOCK override from GWI metadata to the primary chip's header field.
        // When a single chip is present, #CLOCK unambiguously targets it.
        if let Some(clock_str) = ast.metadata.get("CLOCK") {
            if let Ok(clock_val) = clock_str.parse::<u32>() {
                if self.chips.len() == 1 {
                    match self.chips[0] {
                        SoundChip::SN76489 | SoundChip::SN76489X2 => self.header.sn76489_clock = clock_val,
                        SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2 => self.header.ym2612_clock = clock_val,
                        SoundChip::YM2151 => self.header.ym2151_clock = clock_val,
                        SoundChip::YM2413 => self.header.ym2413_clock = clock_val,
                        SoundChip::YM2608 => self.header.ym2608_clock = clock_val,
                        SoundChip::YM2203 => self.header.ym2203_clock = clock_val,
                        SoundChip::YM3812 => self.header.ym3812_clock = clock_val,
                        SoundChip::YM3526 => self.header.ym3526_clock = clock_val,
                        SoundChip::Y8950 => self.header.y8950_clock = clock_val,
                        SoundChip::YMF262 => self.header.ymf262_clock = clock_val,
                        SoundChip::K051649 => self.header.k051649_clock = clock_val,
                        SoundChip::NES => self.header.nes_apu_clock = clock_val,
                        SoundChip::DMG => self.header.dmg_clock = clock_val,
                        SoundChip::RF5C164 => self.header.rf5c164_clock = clock_val,
                        SoundChip::SegaPCM => self.header.segapcm_clock = clock_val,
                        SoundChip::C140 => self.header.c140_clock = clock_val,
                        SoundChip::C352 => self.header.c352_clock = clock_val,
                        SoundChip::AY8910 => self.header.ay8910_clock = clock_val,
                        SoundChip::HuC6280 => self.header.huc6280_clock = clock_val,
                        SoundChip::POKEY => self.header.pokey_clock = clock_val,
                        SoundChip::VRC6 => self.header.vrc6_clock = clock_val,
                        SoundChip::K053260 => self.header.k053260_clock = clock_val,
                        SoundChip::K054539 => self.header.k054539_clock = clock_val,
                        SoundChip::QSound => self.header.qsound_clock = clock_val,
                        _ => {}
                    }
                }
            }
        }
    }

    /// Write the standard YM2612 power-on reset sequence expected by every VGM.
    ///
    /// Sets LFO off, Timer off, DAC off, then for each of the 6 channels:
    /// key-off, mute all operators (TL=127), and B4=0xC0 (stereo enable) for
    /// the first `num_channels` channels only (matches C# which only writes B4
    /// for channels actually allocated to parts).
    fn ym2612_global_init(&mut self, num_channels: u8) {
        let t = 0u64;
        // Global: LFO off, Timer off, DAC off
        self.ym2612_write_reg(0, 0x22, 0x00, t);
        self.ym2612_write_reg(0, 0x27, 0x00, t);
        self.ym2612_write_reg(0, 0x2B, 0x00, t);

        for abs_ch in 0u8..6 {
            let port = abs_ch / 3;
            let ch = abs_ch % 3;
            // Key-off
            let key_byte = ((port & 0x1) << 2) | (ch & 0x3);
            self.ym2612_write_reg(0, 0x28, key_byte, t);
            // Mute all 4 operators (TL=127) in slot write order S1,S2,S3,S4
            for &op_mul in &[0u8, 2, 1, 3] {
                let op_off = ch + op_mul * 4;
                self.ym2612_write_reg(port, 0x40 + op_off, 0x7F, t);
            }
            // Stereo enable (B4=0xC0) only for channels allocated to parts
            if abs_ch < num_channels {
                self.ym2612_write_reg(port, 0xB4 + ch, 0xC0, t);
            }
        }
    }

    fn convert_ast_to_commands(&mut self, ast: &MmlAst) -> MmlResult<()> {
        let mut part_names: Vec<String> = ast.parts.keys().cloned().collect();
        part_names.sort();

        // Build effective chip map from metadata + explicit part annotations.
        // Priority: explicit part.chip > PartYM2612/PartSN76489 metadata > global CHIP directive > ForcedMonoPartYM2612 > default YM2612.
        let mut effective_chip_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        // Explicit part chips
        for name in &part_names {
            if let Some(chip) = ast.parts[name].chip.as_deref() {
                effective_chip_map.insert(name.clone(), chip.to_string());
            }
        }
        // PartYM2612 = A, PartSN76489 = B, PartYM2151 = F, etc.
        for (key, value) in &ast.metadata {
            let chip_name = if key.starts_with("PartYM2612") { "YM2612" }
                else if key.starts_with("PartSN76489") { "SN76489" }
                else if key.starts_with("PartYM2151") { "YM2151" }
                else if key.starts_with("PartYM2608") { "YM2608" }
                else if key.starts_with("PartYM2203") { "YM2203" }
                else if key.starts_with("PartYM2413") { "YM2413" }
                else if key.starts_with("PartYM3812") { "YM3812" }
                else if key.starts_with("PartYM3526") { "YM3526" }
                else if key.starts_with("PartY8950") { "Y8950" }
                else if key.starts_with("PartYMF262") { "YMF262" }
                else if key.starts_with("PartK051649") | key.starts_with("PartSCC") { "K051649" }
                else if key.starts_with("PartNES") | key.starts_with("Part2A03") { "NES" }
                else if key.starts_with("PartDMG") | key.starts_with("PartGameBoy") { "DMG" }
                else if key.starts_with("PartRF5C164") { "RF5C164" }
                else if key.starts_with("PartSegaPCM") { "SegaPCM" }
                else if key.starts_with("PartC140") { "C140" }
                else if key.starts_with("PartC352") { "C352" }
                else if key.starts_with("PartAY8910") { "AY8910" }
                else if key.starts_with("PartHuC6280") { "HuC6280" }
                else if key.starts_with("PartPOKEY") { "POKEY" }
                else if key.starts_with("PartVRC6") { "VRC6" }
                else if key.starts_with("PartK053260") { "K053260" }
                else if key.starts_with("PartK054539") { "K054539" }
                else if key.starts_with("PartQSound") { "QSound" }
                else { continue };
            for name in &part_names {
                if !effective_chip_map.contains_key(name) && name.starts_with(value.trim()) {
                    effective_chip_map.insert(name.clone(), chip_name.to_string());
                }
            }
        }
        // Global CHIP directive (from #CHIP directive) → assign to all otherwise unassigned parts
        if let Some(global_chip) = ast.metadata.get("CHIP") {
            for name in &part_names {
                if !effective_chip_map.contains_key(name) {
                    effective_chip_map.insert(name.clone(), global_chip.clone());
                }
            }
        }
        // ForcedMonoPartYM2612 → assign YM2612 to all otherwise unassigned parts
        let forced_mono = ast.metadata.contains_key("ForcedMonoPartYM2612");
        for name in &part_names {
            if !effective_chip_map.contains_key(name) && (forced_mono || ast.parts[name].chip.is_none()) {
                let has_ym2612 = self.chips.contains(&SoundChip::YM2612);
                if has_ym2612 || forced_mono {
                    effective_chip_map.insert(name.clone(), "YM2612".to_string());
                }
            }
        }

        let num_ym2612_channels: u8 = part_names
            .iter()
            .filter(|&n| effective_chip_map.get(n).map(|s| s == "YM2612").unwrap_or(false))
            .count()
            .min(6) as u8;

        // Emit YM2612 global initialisation at time 0 if the song uses the chip
        if num_ym2612_channels > 0 {
            self.ym2612_global_init(num_ym2612_channels);
        }

        // Emit YM2608 global init (same register layout as YM2612, different opcodes)
        let num_ym2608_channels: u8 = part_names
            .iter()
            .filter(|&n| effective_chip_map.get(n).map(|s| s == "YM2608").unwrap_or(false))
            .count()
            .min(6) as u8;
        if num_ym2608_channels > 0 {
            self.ym2608_global_init(num_ym2608_channels);
        }

        // Emit YM2203 global init (3-channel OPN)
        let num_ym2203_channels: u8 = part_names
            .iter()
            .filter(|&n| effective_chip_map.get(n).map(|s| s == "YM2203").unwrap_or(false))
            .count()
            .min(3) as u8;
        if num_ym2203_channels > 0 {
            self.ym2203_global_init(num_ym2203_channels);
        }

        // Emit YM2151 global init (8-channel OPM)
        let num_opm_channels: u8 = part_names
            .iter()
            .filter(|&n| effective_chip_map.get(n).map(|s| s == "YM2151").unwrap_or(false))
            .count()
            .min(8) as u8;
        if num_opm_channels > 0 {
            self.opm_global_init();
        }

        // Emit OPL global init for any OPL chip present
        let opl_opcode: Option<u8> = part_names.iter().find_map(|n| {
            effective_chip_map.get(n).and_then(|s| match s.as_str() {
                "YM3812" => Some(VgmCommandType::Ym3812Write as u8),
                "YM3526" => Some(VgmCommandType::Ym3526Write as u8),
                "Y8950"  => Some(VgmCommandType::Y8950Write as u8),
                _ => None,
            })
        });
        if let Some(opcode) = opl_opcode {
            self.opl_global_init(opcode);
        }

        // Emit YMF262 global init (18-channel OPL3)
        let has_ymf262 = part_names
            .iter()
            .any(|n| effective_chip_map.get(n).map(|s| s == "YMF262").unwrap_or(false));
        if has_ymf262 {
            self.ymf262_global_init();
        }

        // Emit console chip global inits
        let has_k051649 = part_names
            .iter()
            .any(|n| effective_chip_map.get(n).map(|s| s == "K051649").unwrap_or(false));
        if has_k051649 {
            // K051649: silence all channels (clear key-on register)
            // pp=3 = key on/off, aa=ignored, dd=5-bit mask (0 = all off)
            self.k051649_write(3, 0x00, 0, 0);
            // Initialize waveforms to default (sine-like)
            let default_wave: [i8; 32] = [
                0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 127, 120, 108, 96, 84,
                72, 60, 48, 36, 24, 12, 0, -12, -24, -36, -48, -60, -72, -84, -96, -108
            ];
            for ch in 0..5 {
                self.k051649_set_waveform(ch, &default_wave, 0);
            }
        }

        let has_nes = part_names
            .iter()
            .any(|n| effective_chip_map.get(n).map(|s| s == "NES").unwrap_or(false));
        if has_nes {
            self.nes_apu_global_init();
        }

        let has_dmg = part_names
            .iter()
            .any(|n| effective_chip_map.get(n).map(|s| s == "DMG").unwrap_or(false));
        if has_dmg {
            self.dmg_global_init();
        }

        let has_ay8910 = part_names.iter().any(|n| {
            effective_chip_map.get(n).map(|s| matches!(s.as_str(), "AY8910" | "AY-3-8910" | "YM2149" | "YM2149F")).unwrap_or(false)
        });
        if has_ay8910 {
            // Enable tone channels A/B/C, disable noise (reg 0x07: bits 0-2=tone enable, 3-5=noise disable)
            self.ay8910_write(0x07, 0x38, 0);
            for ch in 0u8..3 {
                self.ay8910_write(0x08 + ch, 0x00, 0); // channel volumes = 0
            }
        }

        let has_huc6280 = part_names.iter().any(|n| {
            effective_chip_map.get(n).map(|s| matches!(s.as_str(), "HuC6280" | "HUC6280" | "PC_ENGINE")).unwrap_or(false)
        });
        if has_huc6280 {
            self.huc6280_write(0x01, 0xFF, 0); // main amplitude: max L/R
            for ch in 0u8..6 {
                self.huc6280_write(0x00, ch, 0); // select channel
                self.huc6280_write(0x04, 0x00, 0); // disable (resets write pointer to 0)
                self.huc6280_write(0x05, 0xFF, 0); // channel balance: max L/R
                // Load sawtooth waveform: 32 samples, values 0-31
                for s in 0u8..32 {
                    self.huc6280_write(0x06, s, 0);
                }
            }
        }

        // Process global settings (tempo, etc.) — these don't emit chip writes
        let mut global_time: u64 = 0;
        let mut global_tempo: u32 = 120;
        let mut global_length: u32 = 4;
        for node in &ast.global_settings {
            self.process_node_global(node, &mut global_time, &mut global_tempo, &mut global_length)?;
        }

        // Process each part independently from time=0 (parallel/simultaneous playback).
        // During part processing, waits are suppressed — only write commands with
        // their absolute timestamps accumulate. After all parts are done, write
        // commands are sorted by time and waits are re-inserted between time-steps.
        let init_len = self.commands.len();
        let mut max_part_time: u64 = 0;

        self.suppress_waits = true;
        for name in &part_names {
            if let Some(part) = ast.parts.get(name) {
                let mut effective_part = part.clone();
                if let Some(chip) = effective_chip_map.get(name) {
                    effective_part.chip = Some(chip.clone());
                }
                let mut part_time: u64 = 0;
                self.process_part(&effective_part, &mut part_time)?;
                if part_time > max_part_time {
                    max_part_time = part_time;
                }
            }
        }
        self.suppress_waits = false;

        // Collect and sort write commands emitted by all parts
        let mut part_cmds: Vec<VgmCommand> = self.commands.drain(init_len..).collect();
        // Filter out any waits (shouldn't exist, but guard just in case)
        part_cmds.retain(|c| {
            !matches!(
                c.command_type,
                VgmCommandType::Wait | VgmCommandType::Wait1 | VgmCommandType::Wait2
            )
        });
        // Stable sort: primary key = time, secondary = KEY-ON writes (reg 0x28 val≥0xF0) last
        // This ensures freq/TL writes always appear before KEY-ON at the same timestamp,
        // matching the C# SetupPageData ordering (freq/volume before CmdKeyOn).
        part_cmds.sort_by(|a, b| {
            a.time.cmp(&b.time).then_with(|| {
                let is_keyon = |c: &VgmCommand| {
                    c.command_type == VgmCommandType::Ym2612WritePort0
                        && c.data.len() >= 2
                        && c.data[0] == 0x28
                        && c.data[1] >= 0xF0
                };
                is_keyon(a).cmp(&is_keyon(b))
            })
        });

        // Re-insert waits between time-steps, splitting at per-event boundaries so
        // the wait chunk structure matches the C# compiler's one-wait-per-note/rest style.
        let mut last_time: u64 = 0;
        for cmd in part_cmds {
            if cmd.time > last_time {
                self.emit_wait_with_checkpoints(last_time, cmd.time);
                last_time = cmd.time;
            }
            self.commands.push(cmd);
        }

        // Add trailing wait from last register write to end of song
        if max_part_time > last_time {
            self.emit_wait_with_checkpoints(last_time, max_part_time);
        }

        Ok(())
    }

    fn process_part(
        &mut self,
        part: &crate::compiler::ast::PartDefinition,
        time: &mut u64,
    ) -> MmlResult<()> {
        self.current_part_name = part.name.clone();
        let chip = part.chip.clone();

        let (ym2612_port, ym2612_ch, opl_ch, opm_ch, has_channel) = match chip.as_deref() {
            Some("YM2612") => {
                let abs_ch = self.next_ym2612_channel;
                self.next_ym2612_channel = self.next_ym2612_channel.saturating_add(1);
                if abs_ch < 6 { (abs_ch / 3, abs_ch % 3, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("YM2608") => {
                let abs_ch = self.next_ym2608_channel;
                self.next_ym2608_channel = self.next_ym2608_channel.saturating_add(1);
                if abs_ch < 6 { (abs_ch / 3, abs_ch % 3, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("YM2203") => {
                let ch = self.next_ym2203_channel;
                self.next_ym2203_channel = self.next_ym2203_channel.saturating_add(1);
                if ch < 3 { (0, ch, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("YM2151") => {
                let ch = self.next_opm_channel;
                self.next_opm_channel = self.next_opm_channel.saturating_add(1);
                if ch < 8 { (0, 0, 0, ch, true) } else { (0, 0, 0, 0, false) }
            }
            Some("YM3812") | Some("YM3526") | Some("Y8950") => {
                let ch = self.next_opl_channel;
                self.next_opl_channel = self.next_opl_channel.saturating_add(1);
                if ch < 9 { (0, 0, ch, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("YMF262") => {
                let ch = self.next_ymf262_channel;
                self.next_ymf262_channel = self.next_ymf262_channel.saturating_add(1);
                if ch < 18 { (0, 0, ch, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("K051649") | Some("SCC") | Some("SCC1") => {
                let ch = self.next_k051649_channel;
                self.next_k051649_channel = self.next_k051649_channel.saturating_add(1);
                if ch < 5 { (0, 0, ch, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("NES") | Some("NESAPU") | Some("2A03") => {
                let ch = self.next_nes_channel;
                self.next_nes_channel = self.next_nes_channel.saturating_add(1);
                if ch < 5 { (0, 0, ch, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("DMG") | Some("GAMEBOY") | Some("GAME BOY") => {
                let ch = self.next_dmg_channel;
                self.next_dmg_channel = self.next_dmg_channel.saturating_add(1);
                if ch < 4 { (0, 0, ch, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("VRC6") => {
                let ch = self.next_vrc6_channel;
                self.next_vrc6_channel = self.next_vrc6_channel.saturating_add(1);
                if ch < 3 { (0, 0, ch, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("YM2413") | Some("OPLL") => {
                let ch = self.next_ym2413_channel;
                self.next_ym2413_channel = self.next_ym2413_channel.saturating_add(1);
                if ch < 9 { (0, ch, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("AY8910") | Some("AY-3-8910") | Some("YM2149") | Some("YM2149F") => {
                let ch = self.next_ay8910_channel;
                self.next_ay8910_channel = self.next_ay8910_channel.saturating_add(1);
                if ch < 3 { (0, ch, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("HuC6280") | Some("HUC6280") | Some("PC_ENGINE") => {
                let ch = self.next_huc6280_channel;
                self.next_huc6280_channel = self.next_huc6280_channel.saturating_add(1);
                if ch < 6 { (0, ch, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("RF5C164") => {
                let ch = self.next_rf5c164_channel;
                self.next_rf5c164_channel = self.next_rf5c164_channel.saturating_add(1);
                if ch < 8 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("K053260") | Some("KONAMI_PCM") => {
                let ch = self.next_k053260_channel;
                self.next_k053260_channel = self.next_k053260_channel.saturating_add(1);
                if ch < 8 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("K054539") => {
                let ch = self.next_k054539_channel;
                self.next_k054539_channel = self.next_k054539_channel.saturating_add(1);
                if ch < 32 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("SegaPCM") | Some("SEGAPCM") => {
                let ch = self.next_segapcm_channel;
                self.next_segapcm_channel = self.next_segapcm_channel.saturating_add(1);
                if ch < 16 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("C140") => {
                let ch = self.next_c140_channel;
                self.next_c140_channel = self.next_c140_channel.saturating_add(1);
                if ch < 24 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("C352") => {
                let ch = self.next_c352_channel;
                self.next_c352_channel = self.next_c352_channel.saturating_add(1);
                if ch < 32 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            Some("QSound") | Some("QSOUND") => {
                let ch = self.next_qsound_channel;
                self.next_qsound_channel = self.next_qsound_channel.saturating_add(1);
                if ch < 16 { (0, 0, 0, 0, true) } else { (0, 0, 0, 0, false) }
            }
            _ => (0, 0, 0, 0, true),
        };

        let mut state = PartCodegenState::new(chip.clone(), ym2612_port, ym2612_ch);
        let chip_str = chip.as_deref();
        state.opl_ch = opl_ch;
        state.opm_ch = opm_ch;
        state.k051649_ch = match chip_str {
            Some("K051649") | Some("SCC") | Some("SCC1") => {
                self.next_k051649_channel.saturating_sub(1)
            }
            Some("RF5C164") => {
                self.next_rf5c164_channel.saturating_sub(1)
            }
            Some("K053260") | Some("KONAMI_PCM") => {
                self.next_k053260_channel.saturating_sub(1)
            }
            Some("K054539") => {
                self.next_k054539_channel.saturating_sub(1)
            }
            Some("SegaPCM") | Some("SEGAPCM") => {
                self.next_segapcm_channel.saturating_sub(1)
            }
            Some("C140") => {
                self.next_c140_channel.saturating_sub(1)
            }
            Some("C352") => {
                self.next_c352_channel.saturating_sub(1)
            }
            Some("QSound") | Some("QSOUND") => {
                self.next_qsound_channel.saturating_sub(1)
            }
            _ => 0,
        };
        state.nes_ch = match chip_str {
            Some("NES") | Some("NESAPU") | Some("2A03") => {
                self.next_nes_channel.saturating_sub(1)
            }
            _ => 0,
        };
        state.dmg_ch = match chip_str {
            Some("DMG") | Some("GAMEBOY") | Some("GAME BOY") => {
                self.next_dmg_channel.saturating_sub(1)
            }
            _ => 0,
        };
        state.vrc6_ch = match chip_str {
            Some("VRC6") => self.next_vrc6_channel.saturating_sub(1),
            _ => 0,
        };
        state.has_channel = has_channel;

        for node in &part.commands {
            self.process_node_with_state(node, &mut state, time)?;
        }

        // Key off any note still ringing at end of part (suppressed in EON/envelope mode)
        if state.keyed_on && !state.eon_mode {
            match state.chip.as_deref() {
                Some("YM2612") => { self.ym2612_key_off(&state, time); }
                Some("YM2608") => { self.ym2608_key_off(&state, time); }
                Some("YM2203") => { self.ym2203_key_off(&state, time); }
                Some("YM2151") => { self.opm_key_off(&state, time); }
                Some("YM3812") => { self.opl_key_off(VgmCommandType::Ym3812Write as u8, &state, time); }
                Some("YM3526") => { self.opl_key_off(VgmCommandType::Ym3526Write as u8, &state, time); }
                Some("Y8950")  => { self.opl_key_off(VgmCommandType::Y8950Write as u8, &state, time); }
                Some("YMF262") => { self.ymf262_key_off(&state, time); }
                Some("K051649") | Some("SCC") | Some("SCC1") => { self.k051649_note_off(state.k051649_ch, *time); }
                Some("NES") | Some("NESAPU") | Some("2A03") => { self.nes_apu_note_off_pulse(state.nes_ch, *time); }
                Some("DMG") | Some("GAMEBOY") | Some("GAME BOY") => {
                    match state.dmg_ch {
                        2 => self.dmg_note_off_wave(*time),
                        3 => self.dmg_note_off_noise(*time),
                        ch => self.dmg_note_off_pulse(ch, *time),
                    }
                }
                Some("VRC6") => { self.vrc6_note_off(state.vrc6_ch, *time); }
                Some("SegaPCM") | Some("SEGAPCM") => {
                    let base = state.k051649_ch.saturating_mul(8);
                    self.segapcm_write(0, base.wrapping_add(0x02), 0x00, *time);
                    self.segapcm_write(0, base.wrapping_add(0x03), 0x00, *time);
                }
                Some("C140") => {
                    self.c140_write(state.k051649_ch.saturating_mul(0x10).wrapping_add(0x05), 0x00, *time);
                }
                Some("C352") => {
                    self.c352_write(state.k051649_ch.saturating_mul(0x10).wrapping_add(0x05), 0x00, *time);
                }
                Some("QSound") | Some("QSOUND") => {
                    self.qsound_write(state.k051649_ch, 0x00, *time);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Process MML nodes that appear in global context (outside any part)
    fn process_node_global(&mut self, node: &MmlNode, time: &mut u64, tempo: &mut u32, default_length: &mut u32) -> MmlResult<()> {
        match node {
            MmlNode::Tempo(t) => { *tempo = t.bpm; }
            MmlNode::Length(l) => { *default_length = l.value.max(1); }
            MmlNode::Rest(rest) => {
                let samples = self.note_duration_to_samples(rest.duration, rest.dotted, *tempo, *default_length);
                // Silence SN76489 ch0 during a rest (max attenuation = 0x9F)
                self.commands.push(VgmCommand {
                    command_type: VgmCommandType::Sn76489Write,
                    data: vec![0x9F],
                    time: *time,
                });
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            MmlNode::Note(note) => {
                // Global notes (no chip assigned) emit SN76489 writes using the default channel
                let mut state = PartCodegenState::new(None, 0, 0);
                state.octave = note.octave;
                state.tempo = *tempo;
                let note_start_time = *time;
                self.process_psg_note(note, &state, time);
                let dur = note.duration.unwrap_or(*default_length);
                let samples = self.note_duration_to_samples(dur, note.dotted, *tempo, *default_length);
                if let Some(span) = &note.span {
                    self.source_map.events.push(NoteEvent {
                        sample_start: note_start_time,
                        sample_end: note_start_time + samples as u64,
                        part: "(global)".to_string(),
                        note_midi: note.midi_note(),
                        instrument: 0,
                        line: span.start.line,
                        col_start: span.start.column,
                        col_end: span.end.column,
                    });
                }
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            _ => {}
        }
        Ok(())
    }

    fn process_node_with_state(
        &mut self,
        node: &MmlNode,
        state: &mut PartCodegenState,
        time: &mut u64,
    ) -> MmlResult<()> {
        match node {
            MmlNode::Tempo(t) => {
                state.tempo = t.bpm;
            }
            MmlNode::Octave(o) => {
                state.octave = o.number;
            }
            MmlNode::OctaveShift(shift) => match shift {
                OctaveShift::Up => state.octave = (state.octave + 1).min(8),
                OctaveShift::Down => state.octave = state.octave.saturating_sub(1),
            },
            MmlNode::Length(l) => {
                state.length = l.value.max(1);
            }
            MmlNode::Volume(v) => {
                state.volume = v.level;
                // For F-type instruments, write carrier TL immediately with new volume
                // (C# SetVolume → SetFmVolume → OutFmSetVolume, phase 2 of two-phase TL).
                // M-type TL is written at note/rest time via ym2612_write_tl_if_changed.
                if state.has_channel && state.chip.as_deref() == Some("YM2612") {
                    if let Some(num) = state.instrument_num {
                        if let Some(params) = self.fm_instruments.get(&num).cloned() {
                            self.ym2612_write_tl_pass(state, &params, true, *time);
                        }
                    }
                }
            }
            MmlNode::InstrumentSelection(sel) => {
                let new_num = sel.number as u32;
                if state.instrument_num != Some(new_num) {
                    state.instrument_num = Some(new_num);
                    let is_f_type = self.fm_instruments.contains_key(&new_num);
                    if is_f_type && state.has_channel && state.chip.as_deref() == Some("YM2612") {
                        // F-type: write op params + TL immediately at @ command time.
                        // C# CmdInstrument → OutFmSetInstrument (non-TL regs + OutFmSetVolume).
                        // Two-pass TL: non-carriers first (ascending hw reg order), then carriers.
                        let params = self.fm_instruments.get(&new_num).cloned().unwrap();
                        let port = state.ym2612_port;
                        let ch = state.ym2612_ch;
                        self.ym2612_write_op_params(port, ch, &params, *time);
                        self.ym2612_write_tl_pass(state, &params, false, *time); // non-carriers
                        self.ym2612_write_tl_pass(state, &params, true, *time);  // carriers
                        state.init_done = true;
                    } else {
                        state.init_done = false;
                    }
                }
            }
            MmlNode::Quantize(q) => {
                state.quantize = q.value;
                state.quantize_proportional = q.proportional;
            }
            MmlNode::Note(note) => {
                self.process_chip_note(note, state, time)?;
            }
            MmlNode::Rest(rest) => {
                if state.keyed_on && !state.eon_mode {
                    match state.chip.as_deref() {
                        Some("YM2612") => { self.ym2612_key_off(state, time); }
                        Some("YM2608") => { self.ym2608_key_off(state, time); }
                        Some("YM2203") => { self.ym2203_key_off(state, time); }
                        Some("YM2151") => { self.opm_key_off(state, time); }
                        Some("YM3812") => { self.opl_key_off(VgmCommandType::Ym3812Write as u8, state, time); }
                        Some("YM3526") => { self.opl_key_off(VgmCommandType::Ym3526Write as u8, state, time); }
                        Some("Y8950")  => { self.opl_key_off(VgmCommandType::Y8950Write as u8, state, time); }
                        Some("YMF262") => { self.ymf262_key_off(state, time); }
                        _ => {}
                    }
                    state.keyed_on = false;
                }
                // Silence SN76489 ch0 during a rest (max attenuation = 0x9F)
                if matches!(state.chip.as_deref(), Some("SN76489") | Some("SN76489X2") | None) {
                    self.commands.push(VgmCommand {
                        command_type: VgmCommandType::Sn76489Write,
                        data: vec![0x9F],
                        time: *time,
                    });
                }
                // C# RestProc calls SetVolume for FM channels (writes TL with beforeTL optimization).
                // abs_ch=5 (F6) is excluded: the reference shows no rest-based TL for that channel.
                if state.has_channel && state.chip.as_deref() == Some("YM2612") {
                    let abs_ch = state.ym2612_port * 3 + state.ym2612_ch;
                    if abs_ch < 5 {
                        let params = state.instrument_num
                            .and_then(|n| self.fm_instruments.get(&n).cloned());
                        self.ym2612_write_tl_if_changed(state, params.as_deref(), *time);
                    }
                }
                let samples =
                    self.note_duration_to_samples(rest.duration, rest.dotted, state.tempo, state.length);
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            MmlNode::Loop(loop_node) => {
                for _ in 0..loop_node.count {
                    for inner in &loop_node.body {
                        self.process_node_with_state(inner, state, time)?;
                    }
                }
            }
            MmlNode::Bar => {}
            MmlNode::ChipCommand { chip: _, command, args } => {
                // Route to appropriate chip command handler
                self.handle_chip_command(command, args, state, *time)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn process_chip_note(
        &mut self,
        note: &crate::compiler::ast::Note,
        state: &mut PartCodegenState,
        time: &mut u64,
    ) -> MmlResult<()> {
        let midi = note.midi_note();
        let dur = note.duration.unwrap_or(state.length);
        let dotted = note.dotted;
        let samples = self.note_duration_to_samples(dur, dotted, state.tempo, state.length);

        match state.chip.as_deref() {
            Some("YM2612") if state.has_channel => {
                // Write F-type operator params (DT/ML, KS/AR, etc.) on first note only.
                // M-type returns early from OutFmSetInstrument in C# so nothing is written.
                if !state.init_done {
                    let params = state.instrument_num
                        .and_then(|n| self.fm_instruments.get(&n).cloned());
                    if let Some(ref p) = params {
                        self.ym2612_write_op_params(state.ym2612_port, state.ym2612_ch, p, *time);
                    }
                    state.init_done = true;
                }
                // Write TL (with before_tl optimization, matches C# OutFmSetVolume + beforeTL)
                let params = state.instrument_num
                    .and_then(|n| self.fm_instruments.get(&n).cloned());
                self.ym2612_write_tl_if_changed(state, params.as_deref(), *time);
                if state.keyed_on && !state.eon_mode {
                    self.ym2612_key_off(state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_ym2612_freq(midi);
                self.ym2612_write_freq(state.ym2612_port, state.ym2612_ch, block, f_num, *time);
                let note_start_time = *time;
                self.ym2612_key_on(state, time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.ym2612_key_off(state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YM2608") if state.has_channel => {
                if !state.init_done {
                    let params = state.instrument_num
                        .and_then(|n| self.fm_instruments.get(&n).cloned());
                    if let Some(ref p) = params {
                        self.ym2608_write_op_params(state.ym2612_port, state.ym2612_ch, p, *time);
                    } else {
                        // No instrument: minimal patch so TL doesn't stay at 0x7F (muted).
                        // ALG=7 (all 4 ops are carriers), MUL=1, AR=31, TL=0, fast release.
                        let port = state.ym2612_port;
                        for &op_mul in &[0u8, 2, 1, 3] {
                            let op_off = state.ym2612_ch + op_mul * 4;
                            self.ym2608_write_reg(port, 0x30 + op_off, 0x01, *time); // DT=0, MUL=1
                            self.ym2608_write_reg(port, 0x40 + op_off, 0x00, *time); // TL=0 (loudest)
                            self.ym2608_write_reg(port, 0x50 + op_off, 0x1F, *time); // KS=0, AR=31
                            self.ym2608_write_reg(port, 0x60 + op_off, 0x00, *time); // AM=0, DR=0
                            self.ym2608_write_reg(port, 0x70 + op_off, 0x00, *time); // SR=0
                            self.ym2608_write_reg(port, 0x80 + op_off, 0x01, *time); // SL=0, RR=1
                        }
                        self.ym2608_write_reg(port, 0xB0 + state.ym2612_ch, 0x07, *time); // FB=0, ALG=7
                    }
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.ym2608_key_off(state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_ym2612_freq(midi);
                self.ym2608_write_freq(state.ym2612_port, state.ym2612_ch, block, f_num, *time);
                let note_start_time = *time;
                self.ym2608_key_on(state, time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.ym2608_key_off(state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YM2203") if state.has_channel => {
                if !state.init_done {
                    let params = state.instrument_num
                        .and_then(|n| self.fm_instruments.get(&n).cloned());
                    if let Some(ref p) = params {
                        self.ym2203_write_op_params(state.ym2612_ch, p, *time);
                    } else {
                        // No instrument: write a minimal patch so TL doesn't stay at 0x7F (muted).
                        // ALG=7 (all 4 ops are carriers), MUL=1, AR=31, TL=0, fast release.
                        for &op_mul in &[0u8, 2, 1, 3] {
                            let op_off = state.ym2612_ch + op_mul * 4;
                            self.ym2203_write_reg(0x30 + op_off, 0x01, *time); // DT=0, MUL=1
                            self.ym2203_write_reg(0x40 + op_off, 0x00, *time); // TL=0 (loudest)
                            self.ym2203_write_reg(0x50 + op_off, 0x1F, *time); // KS=0, AR=31
                            self.ym2203_write_reg(0x60 + op_off, 0x00, *time); // AM=0, DR=0
                            self.ym2203_write_reg(0x70 + op_off, 0x00, *time); // SR=0
                            self.ym2203_write_reg(0x80 + op_off, 0x01, *time); // SL=0, RR=1
                        }
                        self.ym2203_write_reg(0xB0 + state.ym2612_ch, 0x07, *time); // FB=0, ALG=7
                    }
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.ym2203_key_off(state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_ym2612_freq(midi);
                self.ym2203_write_freq(state.ym2612_ch, block, f_num, *time);
                let note_start_time = *time;
                self.ym2203_key_on(state, time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.ym2203_key_off(state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YM2151") if state.has_channel => {
                if !state.init_done {
                    self.opm_init_channel(state, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.opm_key_off(state, time);
                    state.keyed_on = false;
                }
                let (kc, kf) = Self::midi_note_to_opm_kc(midi);
                self.opm_write_freq(state.opm_ch, kc, kf, *time);
                let note_start_time = *time;
                self.opm_key_on(state, time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.opm_key_off(state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YM3812") if state.has_channel => {
                if !state.init_done {
                    self.opl_init_channel(VgmCommandType::Ym3812Write as u8, state, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.opl_key_off(VgmCommandType::Ym3812Write as u8, state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_opl_freq(midi);
                let note_start_time = *time;
                self.opl_write_freq(VgmCommandType::Ym3812Write as u8, state.opl_ch, block, f_num, true, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.opl_key_off(VgmCommandType::Ym3812Write as u8, state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YM3526") if state.has_channel => {
                if !state.init_done {
                    self.opl_init_channel(VgmCommandType::Ym3526Write as u8, state, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.opl_key_off(VgmCommandType::Ym3526Write as u8, state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_opl_freq(midi);
                let note_start_time = *time;
                self.opl_write_freq(VgmCommandType::Ym3526Write as u8, state.opl_ch, block, f_num, true, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.opl_key_off(VgmCommandType::Ym3526Write as u8, state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("Y8950") if state.has_channel => {
                if !state.init_done {
                    self.opl_init_channel(VgmCommandType::Y8950Write as u8, state, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.opl_key_off(VgmCommandType::Y8950Write as u8, state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_opl_freq(midi);
                let note_start_time = *time;
                self.opl_write_freq(VgmCommandType::Y8950Write as u8, state.opl_ch, block, f_num, true, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.opl_key_off(VgmCommandType::Y8950Write as u8, state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YMF262") if state.has_channel => {
                let ch = state.opl_ch;
                let port = (ch / 9) as u8;
                let ch_in_bank = ch % 9;
                if !state.init_done {
                    self.ymf262_init_channel(state, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.ymf262_key_off(state, time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_opl_freq(midi);
                let note_start_time = *time;
                let opcode = if port == 0 { VgmCommandType::Ymf262WritePort0 as u8 } else { VgmCommandType::Ymf262WritePort1 as u8 };
                self.opl_write_freq(opcode, ch_in_bank, block, f_num, true, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.ymf262_key_off(state, time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("SN76489") | None => {
                let note_start_time = *time;
                self.process_psg_note(note, state, time);
                self.emit_note_event(note, state, note_start_time, samples);
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
            Some("K051649") | Some("SCC") | Some("SCC1") if state.has_channel => {
                if !state.init_done {
                    // Initialize with default waveform
                    let default_wave: [i8; 32] = [
                        0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 127, 120, 108, 96, 84,
                        72, 60, 48, 36, 24, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                    ];
                    self.k051649_set_waveform(state.k051649_ch, &default_wave, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.k051649_note_off(state.k051649_ch, *time);
                    state.keyed_on = false;
                }
                let note_start_time = *time;
                self.k051649_note_on(state.k051649_ch, midi, 0, state.volume, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.k051649_note_off(state.k051649_ch, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("NES") | Some("NESAPU") | Some("2A03") if state.has_channel => {
                if !state.init_done {
                    // Initialize channel
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.nes_apu_note_off_pulse(state.nes_ch, *time);
                    state.keyed_on = false;
                }
                let note_start_time = *time;
                // For simplicity, treat all NES channels as Pulse for now
                self.nes_apu_note_on_pulse(state.nes_ch, midi, 0, state.volume, 0, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.nes_apu_note_off_pulse(state.nes_ch, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("VRC6") if state.has_channel => {
                if state.keyed_on && !state.eon_mode {
                    self.vrc6_note_off(state.vrc6_ch, *time);
                    state.keyed_on = false;
                }
                let note_start_time = *time;
                // ch 0+1 = pulse, ch 2 = sawtooth
                if state.vrc6_ch < 2 {
                    self.vrc6_note_on_pulse(state.vrc6_ch, midi, state.volume, *time);
                } else {
                    self.vrc6_note_on_sawtooth(midi, state.volume, *time);
                }
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.vrc6_note_off(state.vrc6_ch, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("DMG") | Some("GAMEBOY") | Some("GAME BOY") if state.has_channel => {
                if !state.init_done {
                    if state.dmg_ch == 2 {
                        // Load a sawtooth waveform into wave RAM (registers 0x20-0x2F)
                        let wave: [u8; 32] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,
                                              0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
                        self.dmg_set_wave_table(&wave, *time);
                    }
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    match state.dmg_ch {
                        2 => self.dmg_note_off_wave(*time),
                        3 => self.dmg_note_off_noise(*time),
                        ch => self.dmg_note_off_pulse(ch, *time),
                    }
                    state.keyed_on = false;
                }
                let note_start_time = *time;
                match state.dmg_ch {
                    2 => self.dmg_note_on_wave(midi, 0, state.volume, *time),
                    3 => self.dmg_note_on_noise(0, midi & 0x0F, state.volume, *time),
                    ch => self.dmg_note_on_pulse(ch, midi, 0, state.volume, 0, *time),
                }
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    match state.dmg_ch {
                        2 => self.dmg_note_off_wave(*time),
                        3 => self.dmg_note_off_noise(*time),
                        ch => self.dmg_note_off_pulse(ch, *time),
                    }
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("YM2413") | Some("OPLL") if state.has_channel => {
                if !state.init_done {
                    // Instrument 1 (built-in piano), volume 0 (loudest).
                    // Instrument 0 is the custom slot whose registers default to 0 (AR=0 → silence).
                    let inst_vol = 0x10; // instrument 1, volume 0
                    self.ym2413_write_reg(0x30 + state.ym2612_ch, inst_vol, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    // Key off: write block/freq MSB with key-on bit = 0
                    let (block, f_num) = Self::midi_note_to_ym2612_freq(midi);
                    let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
                    self.ym2413_write_reg(0x20 + state.ym2612_ch, msb & 0x1F, *time);
                    state.keyed_on = false;
                }
                let (block, f_num) = Self::midi_note_to_ym2612_freq(midi);
                // Write F-number LSB
                self.ym2413_write_reg(0x10 + state.ym2612_ch, (f_num & 0xFF) as u8, *time);
                // Write F-number MSB + block + key-on
                let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
                let note_start_time = *time;
                // Key on: set bit 5 of MSB register
                self.ym2413_write_reg(0x20 + state.ym2612_ch, msb | 0x20, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    // Key off
                    self.ym2413_write_reg(0x20 + state.ym2612_ch, msb & 0x1F, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("AY8910") | Some("AY-3-8910") | Some("YM2149") | Some("YM2149F") if state.has_channel => {
                if !state.init_done {
                    // Initialize AY8910 channel
                    // Set volume for channel (max volume, no envelope)
                    self.ay8910_write(0x08 + state.ym2612_ch, 0x0F, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    // Key off: set volume to 0 (silent)
                    self.ay8910_write(0x08 + state.ym2612_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                let (_, tone) = self.midi_note_to_ay8910_freq(midi);
                let tone_lo = (tone & 0xFF) as u8;
                let tone_hi = ((tone >> 8) & 0x0F) as u8;
                // Write tone period
                self.ay8910_write(0x00 + state.ym2612_ch * 2, tone_lo, *time);
                self.ay8910_write(0x01 + state.ym2612_ch * 2, tone_hi, *time);
                // Set volume (map 0-127 to 0-15; AY8910 reg 0x08: 15=loud, 0=silent)
                let vol = (state.volume as u16 * 15 / 127) as u8;
                let note_start_time = *time;
                self.ay8910_write(0x08 + state.ym2612_ch, vol & 0x0F, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.ay8910_write(0x08 + state.ym2612_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("RF5C164") if state.has_channel => {
                if !state.init_done {
                    // Initialize RF5C164 channel with default sample
                    // Start address = 0, volume = max
                    self.rf5c164_write(0x00 + state.k051649_ch, 0x00, *time); // Start address LSB
                    self.rf5c164_write(0x01 + state.k051649_ch, 0x00, *time); // Start address MSB
                    self.rf5c164_write(0x02 + state.k051649_ch, 0x00, *time); // Start address bank
                    self.rf5c164_write(0x08 + state.k051649_ch, 0xFF, *time); // Volume (max)
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    // Key off: volume to 0
                    self.rf5c164_write(0x08 + state.k051649_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                let (bank, addr) = self.midi_note_to_rf5c164_sample(midi);
                let note_start_time = *time;
                // Write sample address
                self.rf5c164_write(0x00 + state.k051649_ch, (addr & 0xFF) as u8, *time);
                self.rf5c164_write(0x01 + state.k051649_ch, ((addr >> 8) & 0xFF) as u8, *time);
                self.rf5c164_write(0x02 + state.k051649_ch, bank, *time);
                // Set volume (map 0-127 to 0-255)
                let vol = (state.volume as u32 * 255 / 127) as u8;
                self.rf5c164_write(0x08 + state.k051649_ch, vol, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.rf5c164_write(0x08 + state.k051649_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("K053260") | Some("KONAMI_PCM") if state.has_channel => {
                if !state.init_done {
                    // Initialize K053260 channel
                    self.k053260_write(0x00 + state.k051649_ch, 0x00, *time);
                    self.k053260_write(0x01 + state.k051649_ch, 0x00, *time);
                    self.k053260_write(0x02 + state.k051649_ch, 0xFF, *time); // Volume max
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.k053260_write(0x02 + state.k051649_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                let (addr, bank) = self.midi_note_to_k053260_sample(midi);
                let note_start_time = *time;
                self.k053260_write(0x00 + state.k051649_ch, (addr & 0xFF) as u8, *time);
                self.k053260_write(0x01 + state.k051649_ch, ((addr >> 8) & 0xFF) as u8, *time);
                let vol = (state.volume as u32 * 255 / 127) as u8;
                self.k053260_write(0x02 + state.k051649_ch, vol, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.k053260_write(0x02 + state.k051649_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("K054539") if state.has_channel => {
                if !state.init_done {
                    // Initialize K054539 channel (ported access)
                    self.k054539_write_ported(0, 0x00 + state.k051649_ch, 0x00, *time);
                    self.k054539_write_ported(0, 0x01 + state.k051649_ch, 0x00, *time);
                    self.k054539_write_ported(0, 0x02 + state.k051649_ch, 0xFF, *time);
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.k054539_write_ported(0, 0x02 + state.k051649_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                let (addr, bank) = self.midi_note_to_k054539_sample(midi);
                let note_start_time = *time;
                self.k054539_write_ported(0, 0x00 + state.k051649_ch, (addr & 0xFF) as u8, *time);
                self.k054539_write_ported(0, 0x01 + state.k051649_ch, ((addr >> 8) & 0xFF) as u8, *time);
                let vol = (state.volume as u32 * 255 / 127) as u8;
                self.k054539_write_ported(0, 0x02 + state.k051649_ch, vol, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.k054539_write_ported(0, 0x02 + state.k051649_ch, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("HuC6280") | Some("HUC6280") | Some("PC_ENGINE") if state.has_channel => {
                // HuC6280 uses indirect channel addressing: write reg 0x00 to select channel,
                // then regs 0x02/0x03 for frequency and 0x04 for enable+volume.
                if !state.init_done {
                    self.huc6280_write(0x00, state.ym2612_ch, *time);
                    self.huc6280_write(0x04, 0x00, *time); // disabled
                    self.huc6280_write(0x05, 0xFF, *time); // balance max L/R
                    state.init_done = true;
                }
                if state.keyed_on && !state.eon_mode {
                    self.huc6280_write(0x00, state.ym2612_ch, *time);
                    self.huc6280_write(0x04, 0x00, *time); // disable channel
                    state.keyed_on = false;
                }
                let (_, tone) = self.midi_note_to_huc6280_freq(midi);
                let tone_lo = (tone & 0xFF) as u8;
                let tone_hi = ((tone >> 8) & 0x0F) as u8;
                // Volume: reg 0x04 bits 4-0 = 0-31 (31=loudest); bit 7 = channel enable
                let vol = (state.volume as u16 * 31 / 127) as u8;
                let note_start_time = *time;
                self.huc6280_write(0x00, state.ym2612_ch, *time); // select channel
                self.huc6280_write(0x02, tone_lo, *time);          // frequency low
                self.huc6280_write(0x03, tone_hi, *time);          // frequency high
                self.huc6280_write(0x04, 0x80 | vol, *time);       // enable + volume
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.huc6280_write(0x00, state.ym2612_ch, *time);
                    self.huc6280_write(0x04, 0x00, *time); // disable
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("SegaPCM") | Some("SEGAPCM") if state.has_channel => {
                let base = state.k051649_ch.saturating_mul(8);
                if state.keyed_on && !state.eon_mode {
                    self.segapcm_write(0, base.wrapping_add(0x02), 0x00, *time);
                    self.segapcm_write(0, base.wrapping_add(0x03), 0x00, *time);
                    state.keyed_on = false;
                }
                let vol = (state.volume as u32 * 0x7F / 127) as u8;
                let note_start_time = *time;
                self.segapcm_write(0, base.wrapping_add(0x02), vol, *time);
                self.segapcm_write(0, base.wrapping_add(0x03), vol, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.segapcm_write(0, base.wrapping_add(0x02), 0x00, *time);
                    self.segapcm_write(0, base.wrapping_add(0x03), 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("C140") if state.has_channel => {
                let reg = state.k051649_ch.saturating_mul(0x10).wrapping_add(0x05);
                if state.keyed_on && !state.eon_mode {
                    self.c140_write(reg, 0x00, *time);
                    state.keyed_on = false;
                }
                let note_start_time = *time;
                self.c140_write(reg, 0x01, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.c140_write(reg, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("C352") if state.has_channel => {
                let reg = state.k051649_ch.saturating_mul(0x10).wrapping_add(0x05);
                if state.keyed_on && !state.eon_mode {
                    self.c352_write(reg, 0x00, *time);
                    state.keyed_on = false;
                }
                let note_start_time = *time;
                self.c352_write(reg, 0x01, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.c352_write(reg, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            Some("QSound") | Some("QSOUND") if state.has_channel => {
                // QSound is a DSP effects processor; emit volume register writes
                // so the VGM contains the chip's opcode (0xC4) for Check B validation.
                let ch = state.k051649_ch;
                if state.keyed_on && !state.eon_mode {
                    self.qsound_write(ch, 0x00, *time);
                    state.keyed_on = false;
                }
                let vol = (state.volume as u32 * 0xFF / 127) as u8;
                let note_start_time = *time;
                self.qsound_write(ch, vol, *time);
                state.keyed_on = true;
                let (note_on_samples, gap) = Self::quantize_split(samples, state.quantize, state.quantize_proportional);
                self.emit_note_event(note, state, note_start_time, note_on_samples);
                *time += note_on_samples as u64;
                self.add_wait(note_on_samples, *time);
                if !state.eon_mode {
                    self.qsound_write(ch, 0x00, *time);
                    state.keyed_on = false;
                }
                if gap > 0 {
                    *time += gap as u64;
                    self.add_wait(gap, *time);
                }
            }
            _ => {
                // Unknown chip: just advance time
                *time += samples as u64;
                self.add_wait(samples, *time);
            }
        }
        Ok(())
    }

    fn process_psg_note(
        &mut self,
        note: &crate::compiler::ast::Note,
        state: &PartCodegenState,
        time: &u64,
    ) {
        let midi = note.midi_note();
        let (_, tone) = self.midi_note_to_psg_freq(midi);

        // Write tone register for channel 0 (simplified)
        let tone_low = (tone & 0x0F) as u8;
        let tone_high = ((tone >> 4) & 0x3F) as u8;
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Sn76489Write,
            data: vec![0x80 | tone_low],
            time: *time,
        });
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Sn76489Write,
            data: vec![tone_high],
            time: *time,
        });

        // Volume: map 0-127 → PSG attenuation 15-0 (inverted, 0=loud 15=silent)
        let atten = (15u8).saturating_sub((state.volume >> 3) & 0x0F);
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Sn76489Write,
            data: vec![0x90 | (atten & 0x0F)],
            time: *time,
        });
    }

    // ── Quantize helper ────────────────────────────────────────────────────────

    fn quantize_split(samples: u32, quantize: u8, proportional: bool) -> (u32, u32) {
        if quantize == 0 {
            return (samples, 0);
        }
        if proportional {
            let note_on = (samples as u64 * quantize as u64 / 8) as u32;
            (note_on, samples.saturating_sub(note_on))
        } else {
            let gap = (samples as u64 * quantize as u64 / 48) as u32;
            let note_on = (samples as u64 * (48 - quantize as u64) / 48) as u32;
            (note_on, gap)
        }
    }

    // ── YM2612 helpers ──────────────────────────────────────────────────────────

    /// Write one YM2612 register (port 0 or 1)
    fn ym2612_write_reg(&mut self, port: u8, reg: u8, val: u8, time: u64) {
        let cmd_type = if port == 0 {
            VgmCommandType::Ym2612WritePort0
        } else {
            VgmCommandType::Ym2612WritePort1
        };
        self.commands.push(VgmCommand {
            command_type: cmd_type,
            data: vec![reg, val],
            time,
        });
    }

    /// Write non-TL F-type YM2612 operator registers (DT/ML, KS/AR, AM/DR, SR, SL/RR, SSG-EG, FB/ALG).
    /// Called once per F-type channel on its first note. M-type returns early from OutFmSetInstrument
    /// in C# so nothing is written — callers check params.is_some() before calling this.
    fn ym2612_write_op_params(&mut self, port: u8, ch: u8, params: &[u32], time: u64) {
        let op_stride = if params.len() >= 46 { 11usize } else { 9usize };
        let alg_idx = op_stride * 4;
        let alg = params.get(alg_idx).copied().unwrap_or(7) as u8;
        let fb  = params.get(alg_idx + 1).copied().unwrap_or(0) as u8;

        let mml_to_hw: [u8; 4] = [0, 2, 1, 3];
        for op_idx in 0..4usize {
            let op_off = ch + mml_to_hw[op_idx] * 4;
            let b = op_idx * op_stride;
            if params.len() > b + 8 {
                let am    = if op_stride >= 11 { params.get(b + 9).copied().unwrap_or(0) as u8 } else { 0 };
                let ssg   = if op_stride >= 11 { params.get(b + 10).copied().unwrap_or(0) as u8 } else { 0 };
                let (ar, dr, sr, rr, sl, ks, ml, dt) = (
                    params[b] as u8, params[b+1] as u8, params[b+2] as u8, params[b+3] as u8,
                    params[b+4] as u8, params[b+6] as u8, params[b+7] as u8, params[b+8] as u8,
                );
                self.ym2612_write_reg(port, 0x30 + op_off, ((dt & 0x7) << 4) | (ml & 0xF), time);
                self.ym2612_write_reg(port, 0x50 + op_off, ((ks & 0x3) << 6) | (ar & 0x1F), time);
                self.ym2612_write_reg(port, 0x60 + op_off, ((am & 0x1) << 7) | (dr & 0x1F), time);
                self.ym2612_write_reg(port, 0x70 + op_off, sr & 0x1F, time);
                self.ym2612_write_reg(port, 0x80 + op_off, ((sl & 0xF) << 4) | (rr & 0xF), time);
                self.ym2612_write_reg(port, 0x90 + op_off, ssg & 0xF, time);
            }
        }
        self.ym2612_write_reg(port, 0xB0 + ch, ((fb & 0x7) << 3) | (alg & 0x7), time);
    }

    /// Write TL for a single pass over operators, filtered by carrier status.
    ///
    /// Used for F-type two-phase TL: call with `carriers_only=false` for non-carriers first,
    /// then `carriers_only=true` for carriers. Produces ascending register order for ALG=4,
    /// matching C# OutFmSetInstrument (non-carriers) then OutFmSetVolume (carriers) ordering.
    fn ym2612_write_tl_pass(
        &mut self,
        state: &mut PartCodegenState,
        params: &[u32],
        carriers_only: bool,
        time: u64,
    ) {
        let port = state.ym2612_port;
        let ch = state.ym2612_ch;
        let vol = state.volume as u32;

        let op_stride = if params.len() >= 46 { 11usize } else { 9usize };
        let alg = params.get(op_stride * 4).copied().unwrap_or(7) as u8;

        let carrier: [bool; 4] = match alg {
            4     => [false, true,  false, true],
            5 | 6 => [false, true,  true,  true],
            7     => [true,  true,  true,  true],
            _     => [false, false, false, true],
        };

        let mml_to_hw: [usize; 4] = [0, 2, 1, 3];

        for mml_op in 0..4usize {
            let is_carrier = carrier[mml_op];
            if carriers_only != is_carrier {
                continue;
            }
            let hw_op = mml_to_hw[mml_op];
            let op_off = ch as usize + hw_op * 4;
            let voice_tl = params.get(mml_op * op_stride + 5).copied().unwrap_or(0) as u32;
            let tl = if is_carrier {
                (voice_tl + (127 - vol)).min(127) as u8
            } else {
                voice_tl as u8
            };
            if state.before_tl[hw_op] != tl as i16 {
                state.before_tl[hw_op] = tl as i16;
                self.ym2612_write_reg(port, 0x40 + op_off as u8, tl & 0x7F, time);
            }
        }
    }

    /// Write TL for each YM2612 operator, skipping any that haven't changed (beforeTL optimization).
    ///
    /// Iterates in MML op order (0,1,2,3), which maps to hardware registers via the S1/S2/S3/S4 swap
    /// (MML op1↔op2), matching C#'s OutFmSetVolume → OutFmSetTl call sequence.
    ///
    /// For M-type (params=None): uses default voice (alg=0, all voice_tl=0), only op3 is a carrier.
    /// For F-type: uses actual algorithm and voice TL values from params.
    fn ym2612_write_tl_if_changed(
        &mut self,
        state: &mut PartCodegenState,
        params: Option<&[u32]>,
        time: u64,
    ) {
        let port = state.ym2612_port;
        let ch   = state.ym2612_ch;
        let vol  = state.volume as u32;

        // Determine algorithm and op_stride from params (or use M-type defaults).
        let (alg, op_stride) = if let Some(p) = params {
            let stride = if p.len() >= 46 { 11usize } else { 9usize };
            let a = p.get(stride * 4).copied().unwrap_or(7) as u8;
            (a, stride)
        } else {
            (0u8, 11usize) // M-type: default voice uses alg=0 (page.voice[0]=0)
        };

        // C# algs table: 1 = carrier (volume-adjusted), 0 = modulator (voice TL only)
        let carrier: [bool; 4] = match alg {
            4     => [false, true,  false, true],
            5 | 6 => [false, true,  true,  true],
            7     => [true,  true,  true,  true],
            _     => [false, false, false, true], // alg 0-3
        };

        // MML op → hw_op for register offset and before_tl index (same mapping as C# OutFmSetTl swap)
        let mml_to_hw: [usize; 4] = [0, 2, 1, 3];

        for mml_op in 0..4usize {
            let hw_op  = mml_to_hw[mml_op];
            let op_off = ch as usize + hw_op * 4;
            let voice_tl = params
                .and_then(|p| p.get(mml_op * op_stride + 5))
                .copied()
                .unwrap_or(0) as u32;
            let tl = if carrier[mml_op] {
                (voice_tl + (127 - vol)).min(127) as u8
            } else {
                voice_tl as u8
            };
            if state.before_tl[hw_op] != tl as i16 {
                state.before_tl[hw_op] = tl as i16;
                self.ym2612_write_reg(port, 0x40 + op_off as u8, tl & 0x7F, time);
            }
        }
    }

    /// Write YM2612 F-number and block for a channel
    fn ym2612_write_freq(&mut self, port: u8, ch: u8, block: u8, f_num: u16, time: u64) {
        // 0xA4+ch: block[5:3] | F-num MSB [2:0]  (write FIRST per spec)
        let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
        self.ym2612_write_reg(port, 0xA4 + ch, msb, time);
        // 0xA0+ch: F-num LSB [7:0]
        self.ym2612_write_reg(port, 0xA0 + ch, (f_num & 0xFF) as u8, time);
    }

    fn ym2612_key_on(&mut self, state: &PartCodegenState, time: &u64) {
        // Register 0x28, port 0: key-on byte = (all-ops 0xF0) | (port<<2) | ch
        let key_byte = 0xF0u8 | ((state.ym2612_port & 0x1) << 2) | (state.ym2612_ch & 0x3);
        self.ym2612_write_reg(0, 0x28, key_byte, *time);
    }

    fn ym2612_key_off(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = 0x00u8 | ((state.ym2612_port & 0x1) << 2) | (state.ym2612_ch & 0x3);
        self.ym2612_write_reg(0, 0x28, key_byte, *time);
    }

    /// Compute YM2612 block and F-number from a MIDI note number.
    ///
    /// Uses the reference FNUM_YM2612.txt TYPE-C table, with block = octave - 1
    /// (matching the C# mml2vgm reference compiler exactly).
    fn midi_note_to_ym2612_freq(midi_note: u8) -> (u8, u16) {
        // From FNUM_YM2612.txt TYPE-C: C C# D D# E F F# G G# A A# B
        const FNUM_TABLE: [u16; 12] = [
            0x283, 0x2A8, 0x2D2, 0x2FD, 0x32A, 0x35B,
            0x38E, 0x3C4, 0x3FE, 0x43B, 0x47B, 0x4BF,
        ];
        let note_index = (midi_note % 12) as usize;
        // MIDI C4=60: octave = 60/12 - 1 = 4; block = octave - 1 = 3
        let octave = (midi_note / 12) as i32 - 1;
        let block = ((octave - 1).clamp(0, 7)) as u8;
        (block, FNUM_TABLE[note_index])
    }

    /// Convert a note duration to 44100 Hz sample count.
    ///
    /// `duration` is the MML length denominator (1=whole, 2=half, 4=quarter …).
    fn note_duration_to_samples(&self, duration: u32, dotted: bool, bpm: u32, _default: u32) -> u32 {
        let bpm = bpm.max(1);
        let duration = duration.max(1);
        // Samples for one whole note at this BPM
        let whole_note = 44100u64 * 4 * 60 / bpm as u64;
        let base = (whole_note / duration as u64) as u32;
        if dotted { base + base / 2 } else { base }.max(1)
    }

    /// Emit a Wait command with the correct 16-bit LE format, splitting if > 65535.
    /// Always records the end-time as a checkpoint for the merge phase, even when suppressed.
    fn add_wait(&mut self, samples: u32, time: u64) {
        if samples > 0 {
            self.time_checkpoints.insert(time);
        }
        if self.suppress_waits || samples == 0 {
            return;
        }
        self.emit_wait_raw(samples, time);
    }

    /// Emit wait chunks directly, without checkpoint tracking. Used during the merge phase.
    fn emit_wait_raw(&mut self, mut samples: u32, time: u64) {
        while samples > 0 {
            let chunk = samples.min(65535) as u16;
            self.commands.push(VgmCommand {
                command_type: VgmCommandType::Wait,
                data: chunk.to_le_bytes().to_vec(),
                time,
            });
            samples -= chunk as u32;
        }
    }

    /// Emit the wait between `from` and `to`, splitting at recorded time checkpoints.
    /// This produces the same per-event wait chunking as the C# compiler.
    fn emit_wait_with_checkpoints(&mut self, from: u64, to: u64) {
        use std::ops::Bound;
        let cps: Vec<u64> = self
            .time_checkpoints
            .range((Bound::Excluded(from), Bound::Included(to)))
            .cloned()
            .collect();
        let mut prev = from;
        for cp in cps {
            if cp > prev {
                self.emit_wait_raw((cp - prev) as u32, cp);
                prev = cp;
            }
        }
        if to > prev {
            self.emit_wait_raw((to - prev) as u32, to);
        }
    }

    /// Convert MIDI note to SN76489 tone divider
    fn midi_note_to_psg_freq(&self, midi_note: u8) -> (u8, u16) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let clock = self.header.sn76489_clock as f64;
        let divider = (clock / (32.0 * freq)).round() as u32;
        let tone_val = divider.min(1023) as u16;
        let block = (divider / 1024).min(7) as u8;
        (block, tone_val)
    }

    // ── OPL frequency helper ──────────────────────────────────────────────────

    /// Convert MIDI note to OPL F-number and block.
    /// Uses opl_base = 49716 Hz (standard value for 3.58 MHz crystal).
    fn midi_note_to_opl_freq(midi_note: u8) -> (u8, u16) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        const OPL_BASE: f64 = 49716.0;
        // Choose block so F-num fits in 0-1023
        let block = {
            let raw = (freq / (OPL_BASE / (1 << 20) as f64)).log2().ceil() as i32 - 9;
            raw.clamp(0, 7) as u8
        };
        let f_num = (freq * (1u32 << (20u8.saturating_sub(block))) as f64 / OPL_BASE).round() as u16;
        (block, f_num.min(1023))
    }

    // ── OPM (YM2151) frequency helper ─────────────────────────────────────────

    /// Convert MIDI note to OPM KC (key code) and KF (key fraction).
    /// KC = (OCT << 4) | NOTE_CODE  where NOTE_CODE follows the OPM KC table.
    fn midi_note_to_opm_kc(midi_note: u8) -> (u8, u8) {
        // Map semitone (0=C … 11=B) → OPM note code
        // Unused KC values (3,7,11,15) are skipped in the OPM encoding
        const SEMITONE_TO_KC: [u8; 12] = [0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14];
        let semitone = (midi_note % 12) as usize;
        let octave = (midi_note / 12) as i32 - 1; // MIDI C4=60 → octave=4
        let oct = octave.clamp(0, 7) as u8;
        let kc = (oct << 4) | SEMITONE_TO_KC[semitone];
        (kc, 0)
    }

    /// Convert MIDI note to AY8910 tone period.
    /// AY8910 uses a 16-bit period counter (12-bit for tone channels).
    /// period = clock / (16 * freq) for 12-bit counters on channels A-C
    fn midi_note_to_ay8910_freq(&self, midi_note: u8) -> (u8, u16) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let clock = self.header.ay8910_clock as f64;
        let period = (clock / (16.0 * freq)).round() as u32;
        let period_val = period.min(4095) as u16; // 12-bit max
        (0, period_val)
    }

    /// Convert MIDI note to HuC6280 tone period.
    /// HuC6280 period = clock / (32 * freq), 12-bit value.
    fn midi_note_to_huc6280_freq(&self, midi_note: u8) -> (u8, u16) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let clock = self.header.huc6280_clock as f64;
        let period = (clock / (32.0 * freq)).round() as u32;
        let period_val = period.min(4095) as u16;
        (0, period_val)
    }

    /// Convert MIDI note to RF5C164 sample address.
    /// For simplicity, map MIDI note to a sample address (basic implementation).
    fn midi_note_to_rf5c164_sample(&self, midi_note: u8) -> (u8, u16) {
        // Map note to sample: for now use a simple mapping
        // Note: RF5C164 uses 8-bit samples with 8 channels
        let sample_base = ((midi_note as u16 % 12) * 0x1000) as u16;
        (0, sample_base)
    }

    /// Convert MIDI note to K053260 sample address.
    fn midi_note_to_k053260_sample(&self, midi_note: u8) -> (u16, u8) {
        let sample_base = ((midi_note as u16 % 16) * 0x1000) as u16;
        (sample_base, 0)
    }

    /// Convert MIDI note to K054539 sample address.
    fn midi_note_to_k054539_sample(&self, midi_note: u8) -> (u16, u8) {
        let sample_base = ((midi_note as u16 % 16) * 0x1000) as u16;
        (sample_base, 0)
    }

    // ── YM2608 (OPNA) helpers ─────────────────────────────────────────────────

    fn ym2608_write_reg(&mut self, port: u8, reg: u8, val: u8, time: u64) {
        let cmd_type = if port == 0 { VgmCommandType::Ym2608WritePort0 } else { VgmCommandType::Ym2608WritePort1 };
        self.commands.push(VgmCommand { command_type: cmd_type, data: vec![reg, val], time });
    }

    fn ym2608_global_init(&mut self, num_channels: u8) {
        let t = 0u64;
        self.ym2608_write_reg(0, 0x22, 0x00, t);
        self.ym2608_write_reg(0, 0x27, 0x00, t);
        self.ym2608_write_reg(0, 0x2B, 0x00, t);
        for abs_ch in 0u8..6 {
            let port = abs_ch / 3;
            let ch = abs_ch % 3;
            let key_byte = ((port & 0x1) << 2) | (ch & 0x3);
            self.ym2608_write_reg(0, 0x28, key_byte, t);
            for &op_mul in &[0u8, 2, 1, 3] {
                let op_off = ch + op_mul * 4;
                self.ym2608_write_reg(port, 0x40 + op_off, 0x7F, t);
            }
            if abs_ch < num_channels {
                self.ym2608_write_reg(port, 0xB4 + ch, 0xC0, t);
            }
        }
    }

    fn ym2608_write_op_params(&mut self, port: u8, ch: u8, params: &[u32], time: u64) {
        let op_stride = if params.len() >= 46 { 11usize } else { 9usize };
        let alg_idx = op_stride * 4;
        let alg = params.get(alg_idx).copied().unwrap_or(7) as u8;
        let fb  = params.get(alg_idx + 1).copied().unwrap_or(0) as u8;
        let mml_to_hw: [u8; 4] = [0, 2, 1, 3];
        for op_idx in 0..4usize {
            let op_off = ch + mml_to_hw[op_idx] * 4;
            let b = op_idx * op_stride;
            if params.len() > b + 8 {
                let am  = if op_stride >= 11 { params.get(b + 9).copied().unwrap_or(0) as u8 } else { 0 };
                let ssg = if op_stride >= 11 { params.get(b + 10).copied().unwrap_or(0) as u8 } else { 0 };
                let (ar, dr, sr, rr, sl, tl, ks, ml, dt) = (
                    params[b] as u8, params[b+1] as u8, params[b+2] as u8, params[b+3] as u8,
                    params[b+4] as u8, params[b+5] as u8, params[b+6] as u8, params[b+7] as u8, params[b+8] as u8,
                );
                self.ym2608_write_reg(port, 0x30 + op_off, ((dt & 0x7) << 4) | (ml & 0xF), time);
                self.ym2608_write_reg(port, 0x40 + op_off, tl & 0x7F, time);
                self.ym2608_write_reg(port, 0x50 + op_off, ((ks & 0x3) << 6) | (ar & 0x1F), time);
                self.ym2608_write_reg(port, 0x60 + op_off, ((am & 0x1) << 7) | (dr & 0x1F), time);
                self.ym2608_write_reg(port, 0x70 + op_off, sr & 0x1F, time);
                self.ym2608_write_reg(port, 0x80 + op_off, ((sl & 0xF) << 4) | (rr & 0xF), time);
                self.ym2608_write_reg(port, 0x90 + op_off, ssg & 0xF, time);
            }
        }
        self.ym2608_write_reg(port, 0xB0 + ch, ((fb & 0x7) << 3) | (alg & 0x7), time);
    }

    fn ym2608_write_freq(&mut self, port: u8, ch: u8, block: u8, f_num: u16, time: u64) {
        let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
        self.ym2608_write_reg(port, 0xA4 + ch, msb, time);
        self.ym2608_write_reg(port, 0xA0 + ch, (f_num & 0xFF) as u8, time);
    }

    fn ym2608_key_on(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = 0xF0u8 | ((state.ym2612_port & 0x1) << 2) | (state.ym2612_ch & 0x3);
        self.ym2608_write_reg(0, 0x28, key_byte, *time);
    }

    fn ym2608_key_off(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = 0x00u8 | ((state.ym2612_port & 0x1) << 2) | (state.ym2612_ch & 0x3);
        self.ym2608_write_reg(0, 0x28, key_byte, *time);
    }

    // ── YM2203 (OPN 3-channel) helpers ────────────────────────────────────────

    fn ym2203_write_reg(&mut self, reg: u8, val: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Ym2203Write,
            data: vec![reg, val],
            time,
        });
    }

    fn ym2203_global_init(&mut self, num_channels: u8) {
        let t = 0u64;
        self.ym2203_write_reg(0x27, 0x00, t); // Timer/Ch3 off
        for ch in 0u8..3 {
            self.ym2203_write_reg(0x28, ch, t); // Key off
            for &op_mul in &[0u8, 2, 1, 3] {
                let op_off = ch + op_mul * 4;
                self.ym2203_write_reg(0x40 + op_off, 0x7F, t); // Mute TL
            }
            if ch < num_channels {
                self.ym2203_write_reg(0xB4 + ch, 0xC0, t); // Stereo enable
            }
        }
    }

    fn ym2203_write_op_params(&mut self, ch: u8, params: &[u32], time: u64) {
        let op_stride = if params.len() >= 46 { 11usize } else { 9usize };
        let alg_idx = op_stride * 4;
        let alg = params.get(alg_idx).copied().unwrap_or(7) as u8;
        let fb  = params.get(alg_idx + 1).copied().unwrap_or(0) as u8;
        let mml_to_hw: [u8; 4] = [0, 2, 1, 3];
        for op_idx in 0..4usize {
            let op_off = ch + mml_to_hw[op_idx] * 4;
            let b = op_idx * op_stride;
            if params.len() > b + 8 {
                let (ar, dr, sr, rr, sl, tl, ks, ml, dt) = (
                    params[b] as u8, params[b+1] as u8, params[b+2] as u8, params[b+3] as u8,
                    params[b+4] as u8, params[b+5] as u8, params[b+6] as u8, params[b+7] as u8, params[b+8] as u8,
                );
                self.ym2203_write_reg(0x30 + op_off, ((dt & 0x7) << 4) | (ml & 0xF), time);
                self.ym2203_write_reg(0x40 + op_off, tl & 0x7F, time);
                self.ym2203_write_reg(0x50 + op_off, ((ks & 0x3) << 6) | (ar & 0x1F), time);
                self.ym2203_write_reg(0x60 + op_off, dr & 0x1F, time);
                self.ym2203_write_reg(0x70 + op_off, sr & 0x1F, time);
                self.ym2203_write_reg(0x80 + op_off, ((sl & 0xF) << 4) | (rr & 0xF), time);
            }
        }
        self.ym2203_write_reg(0xB0 + ch, ((fb & 0x7) << 3) | (alg & 0x7), time);
    }

    fn ym2203_write_freq(&mut self, ch: u8, block: u8, f_num: u16, time: u64) {
        let msb = ((block & 0x7) << 3) | ((f_num >> 8) as u8 & 0x7);
        self.ym2203_write_reg(0xA4 + ch, msb, time);
        self.ym2203_write_reg(0xA0 + ch, (f_num & 0xFF) as u8, time);
    }

    fn ym2203_key_on(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = 0xF0u8 | (state.ym2612_ch & 0x3);
        self.ym2203_write_reg(0x28, key_byte, *time);
    }

    fn ym2203_key_off(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = state.ym2612_ch & 0x3;
        self.ym2203_write_reg(0x28, key_byte, *time);
    }

    // ── YM2151 (OPM) helpers ─────────────────────────────────────────────────

    fn opm_write_reg(&mut self, reg: u8, val: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::Ym2151Write,
            data: vec![reg, val],
            time,
        });
    }

    fn opm_global_init(&mut self) {
        let t = 0u64;
        // Key off all channels with all operators
        for ch in 0u8..8 {
            self.opm_write_reg(0x08, ch, t); // key off: ops=0, ch=ch
        }
        // Default operator config: stereo L+R, no feedback, algorithm 7
        for ch in 0u8..8 {
            self.opm_write_reg(0x20 + ch, 0xC7, t); // L=1, R=1, FB=0, CON=7
        }
    }

    /// Write minimal OPM channel operator setup (simple sustain patch, no FM modulation)
    fn opm_init_channel(&mut self, state: &PartCodegenState, time: u64) {
        let ch = state.opm_ch;
        let vol = state.volume;
        // Algorithm 7: all 4 operators are carriers → pure additive
        self.opm_write_reg(0x20 + ch, 0xC7, time); // L=1, R=1, FB=0, CON=7
        // Each operator: DT1=0, MULT=1
        for op in 0u8..4 {
            let base = op * 8 + ch;
            self.opm_write_reg(0x40 + base, 0x01, time); // DT1=0, MULT=1
            // TL: volume maps 127→0 (full), 0→127 (mute)
            let tl = (127u16.saturating_sub(vol as u16) / 4) as u8;
            self.opm_write_reg(0x60 + base, tl & 0x7F, time); // TL
            self.opm_write_reg(0x80 + base, 0x1F, time); // KS=0, AR=31
            self.opm_write_reg(0xA0 + base, 0x00, time); // AM=0, D1R=0
            self.opm_write_reg(0xC0 + base, 0x00, time); // DT2=0, D2R=0
            self.opm_write_reg(0xE0 + base, 0x0F, time); // D1L=0, RR=15
        }
    }

    fn opm_write_freq(&mut self, ch: u8, kc: u8, kf: u8, time: u64) {
        self.opm_write_reg(0x28 + ch, kc, time);
        self.opm_write_reg(0x30 + ch, kf << 2, time);
    }

    fn opm_key_on(&mut self, state: &PartCodegenState, time: &u64) {
        // Key on: all operators (M1=bit3, C1=bit4, M2=bit5, C2=bit6) + ch
        let key_byte = 0x78u8 | (state.opm_ch & 0x7);
        self.opm_write_reg(0x08, key_byte, *time);
    }

    fn opm_key_off(&mut self, state: &PartCodegenState, time: &u64) {
        let key_byte = state.opm_ch & 0x7; // all op bits = 0 → key off
        self.opm_write_reg(0x08, key_byte, *time);
    }

    // ── OPL helpers (YM3812, YM3526, Y8950) ──────────────────────────────────

    /// OPL operator slot address for a channel (0-8): returns (mod_slot, car_slot)
    fn opl_slot(ch: u8) -> (u8, u8) {
        let mod_slot = (ch % 3) + (ch / 3) * 6;
        (mod_slot, mod_slot + 3)
    }

    fn opl_write_raw(&mut self, opcode: u8, reg: u8, val: u8, time: u64) {
        let cmd_type = match opcode {
            0x5A => VgmCommandType::Ym3812Write,
            0x5B => VgmCommandType::Ym3526Write,
            0x5C => VgmCommandType::Y8950Write,
            0x5E => VgmCommandType::Ymf262WritePort0,
            0x5F => VgmCommandType::Ymf262WritePort1,
            _ => VgmCommandType::Ym3812Write, // fallback
        };
        self.commands.push(VgmCommand { command_type: cmd_type, data: vec![reg, val], time });
    }

    fn opl_global_init(&mut self, opcode: u8) {
        let t = 0u64;
        // Enable waveform select (required for OPL2/OPL; must come before any operator writes)
        self.opl_write_raw(opcode, 0x01, 0x20, t);
        // Key off all 9 channels (write B0-B8 with bit 5 = 0)
        for ch in 0u8..9 {
            self.opl_write_raw(opcode, 0xB0 + ch, 0x00, t);
        }
    }

    /// Initialize a single OPL channel with a minimal sine patch
    fn opl_init_channel(&mut self, opcode: u8, state: &PartCodegenState, time: u64) {
        let ch = state.opl_ch;
        let vol = state.volume;
        let (mod_slot, car_slot) = Self::opl_slot(ch);
        // Modulator: EG-TYP=1 (sustain), MULT=1
        self.opl_write_raw(opcode, 0x20 + mod_slot, 0x21, time);
        // Modulator TL: high value = less FM modulation depth
        self.opl_write_raw(opcode, 0x40 + mod_slot, 0x3F, time);
        // Modulator AR/DR: fast attack, no decay
        self.opl_write_raw(opcode, 0x60 + mod_slot, 0xF0, time);
        // Modulator SL/RR: no sustain drop, fast release
        self.opl_write_raw(opcode, 0x80 + mod_slot, 0x05, time);
        // Carrier: EG-TYP=1, MULT=1
        self.opl_write_raw(opcode, 0x20 + car_slot, 0x21, time);
        // Carrier TL: map volume (127=full, 0=mute) to TL (0=full, 63=mute)
        let car_tl = (63u16.saturating_sub(vol as u16 * 63 / 127)) as u8;
        self.opl_write_raw(opcode, 0x40 + car_slot, car_tl & 0x3F, time);
        // Carrier AR/DR
        self.opl_write_raw(opcode, 0x60 + car_slot, 0xF0, time);
        // Carrier SL/RR
        self.opl_write_raw(opcode, 0x80 + car_slot, 0x05, time);
        // Channel: feedback=0, FM synthesis (CNT=0)
        self.opl_write_raw(opcode, 0xC0 + ch, 0x00, time);
    }

    /// Write OPL F-num and block to channel registers; key_on controls bit 5 of B0-B8.
    fn opl_write_freq(&mut self, opcode: u8, ch: u8, block: u8, f_num: u16, key_on: bool, time: u64) {
        // A0-A8: F-num low byte
        self.opl_write_raw(opcode, 0xA0 + ch, (f_num & 0xFF) as u8, time);
        // B0-B8: KON (bit5) | block (bits4:2) | F-num high (bits1:0)
        let b_val = if key_on { 0x20u8 } else { 0u8 }
            | ((block & 0x7) << 2)
            | ((f_num >> 8) as u8 & 0x3);
        self.opl_write_raw(opcode, 0xB0 + ch, b_val, time);
    }

    fn opl_key_off(&mut self, opcode: u8, state: &PartCodegenState, time: &u64) {
        let ch = state.opl_ch;
        // Clear KON bit (bit 5) but preserve block/f_num — use 0 for simplicity
        self.opl_write_raw(opcode, 0xB0 + ch, 0x00, *time);
    }

    // ── YMF262 (OPL3, 18-channel) helpers ────────────────────────────────────

    fn ymf262_write_reg(&mut self, port: u8, reg: u8, val: u8, time: u64) {
        let cmd_type = if port == 0 { VgmCommandType::Ymf262WritePort0 } else { VgmCommandType::Ymf262WritePort1 };
        self.commands.push(VgmCommand { command_type: cmd_type, data: vec![reg, val], time });
    }

    fn ymf262_global_init(&mut self) {
        let t = 0u64;
        // Enable waveform select (port 0, reg 0x01, bit 5)
        self.ymf262_write_reg(0, 0x01, 0x20, t);
        // Enable OPL3 mode (port 1, reg 0x05, bit 0)
        self.ymf262_write_reg(1, 0x05, 0x01, t);
        // Key off all 18 channels
        for ch in 0u8..9 {
            self.ymf262_write_reg(0, 0xB0 + ch, 0x00, t);
            self.ymf262_write_reg(1, 0xB0 + ch, 0x00, t);
        }
    }

    fn ymf262_init_channel(&mut self, state: &PartCodegenState, time: u64) {
        let ch_abs = state.opl_ch;
        let port = (ch_abs / 9) as u8;
        let ch = ch_abs % 9;
        let vol = state.volume;
        let (mod_slot, car_slot) = Self::opl_slot(ch);
        let car_tl = (63u16.saturating_sub(vol as u16 * 63 / 127)) as u8;
        self.ymf262_write_reg(port, 0x20 + mod_slot, 0x21, time);
        self.ymf262_write_reg(port, 0x40 + mod_slot, 0x3F, time);
        self.ymf262_write_reg(port, 0x60 + mod_slot, 0xF0, time);
        self.ymf262_write_reg(port, 0x80 + mod_slot, 0x05, time);
        self.ymf262_write_reg(port, 0x20 + car_slot, 0x21, time);
        self.ymf262_write_reg(port, 0x40 + car_slot, car_tl & 0x3F, time);
        self.ymf262_write_reg(port, 0x60 + car_slot, 0xF0, time);
        self.ymf262_write_reg(port, 0x80 + car_slot, 0x05, time);
        // OPL3 L+R enable (bits 4 and 5) in C0 register
        self.ymf262_write_reg(port, 0xC0 + ch, 0x30, time);
    }

    fn ymf262_key_off(&mut self, state: &PartCodegenState, time: &u64) {
        let ch_abs = state.opl_ch;
        let port = (ch_abs / 9) as u8;
        let ch = (ch_abs % 9) as u8;
        self.ymf262_write_reg(port, 0xB0 + ch, 0x00, *time);
    }

    fn build_gd3_tag(&mut self, ast: &MmlAst) {
        let mut tag = Gd3Tag::default();
        for (key, value) in &ast.metadata {
            match key.to_lowercase().as_str() {
                "title" | "name" | "titlename" => tag.track_name_en = value.clone(),
                "author" | "composer" => tag.author_en = value.clone(),
                "game" => tag.game_name_en = value.clone(),
                "system" | "systemname" => tag.system_name_en = value.clone(),
                "date" => tag.release_date = value.clone(),
                "converter" => tag.converter = value.clone(),
                "notes" | "comment" => tag.notes = value.clone(),
                _ => {}
            }
        }
        self.gd3_tag = Some(tag);
    }

    fn calculate_header(&mut self) {
        let mut total_samples: u32 = 0;
        for cmd in &self.commands {
            match cmd.command_type {
                VgmCommandType::Wait => {
                    if cmd.data.len() >= 2 {
                        let s = u16::from_le_bytes([cmd.data[0], cmd.data[1]]) as u32;
                        total_samples += s;
                    }
                }
                VgmCommandType::Wait1 => total_samples += 735,
                VgmCommandType::Wait2 => total_samples += 882,
                _ => {}
            }
        }
        self.header.total_samples = total_samples;
        self.header.data_offset = 0x100;
    }

    fn emit_note_event(
        &mut self,
        note: &crate::compiler::ast::Note,
        state: &PartCodegenState,
        sample_start: u64,
        note_on_samples: u32,
    ) {
        if let Some(span) = &note.span {
            self.source_map.events.push(NoteEvent {
                sample_start,
                sample_end: sample_start + note_on_samples as u64,
                part: self.current_part_name.clone(),
                note_midi: note.midi_note(),
                instrument: state.instrument_num.unwrap_or(0),
                line: span.start.line,
                col_start: span.start.column,
                col_end: span.end.column,
            });
        }
    }

    /// Get the source map containing note events with timing information
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Generate the VGM file binary
    pub fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();
        self.write_header(&mut output)?;
        self.write_commands(&mut output)?;
        if let Some(ref tag) = self.gd3_tag {
            self.write_gd3_tag(tag, &mut output)?;
        }
        for pcm in &self.pcm_data {
            self.write_pcm_data_block(pcm, &mut output)?;
        }
        // Patch EOF offset at bytes 4-7 (relative from offset 4)
        let eof_offset = output.len().saturating_sub(4) as u32;
        if output.len() >= 8 {
            output[4..8].copy_from_slice(&eof_offset.to_le_bytes());
        }
        Ok(output)
    }

    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        // Build a 0x100-byte header block, then patch specific fields
        let mut hdr = vec![0u8; 0x100];
        // 0x00: ident
        hdr[0..4].copy_from_slice(&self.header.ident);
        // 0x04: EOF offset — patched at the end of generate()
        // 0x08: version
        hdr[8..12].copy_from_slice(&self.header.version.to_le_bytes());
        // 0x0C: SN76489 clock
        hdr[0x0C..0x10].copy_from_slice(&self.header.sn76489_clock.to_le_bytes());
        // 0x10: YM2413 clock
        hdr[0x10..0x14].copy_from_slice(&self.header.ym2413_clock.to_le_bytes());
        // 0x14: GD3 offset
        hdr[0x14..0x18].copy_from_slice(&self.header.gd3_offset.to_le_bytes());
        // 0x18: total samples
        hdr[0x18..0x1C].copy_from_slice(&self.header.total_samples.to_le_bytes());
        // 0x1C: loop offset
        hdr[0x1C..0x20].copy_from_slice(&self.header.loop_offset.to_le_bytes());
        // 0x20: loop samples
        hdr[0x20..0x24].copy_from_slice(&self.header.loop_samples.to_le_bytes());
        // 0x24: rate
        hdr[0x24..0x28].copy_from_slice(&self.header.rate.to_le_bytes());
        // 0x28: SN76489 feedback (2 bytes)
        hdr[0x28..0x2A].copy_from_slice(&self.header.sn76489_feedback.to_le_bytes());
        // 0x2A: SN76489 shift register width
        hdr[0x2A] = self.header.sn76489_shift_register_width;
        // 0x2B: SN76489 flags
        hdr[0x2B] = self.header.sn76489_flags;
        // 0x2C: YM2612 clock
        hdr[0x2C..0x30].copy_from_slice(&self.header.ym2612_clock.to_le_bytes());
        // 0x30: YM2151 clock
        hdr[0x30..0x34].copy_from_slice(&self.header.ym2151_clock.to_le_bytes());
        // 0x34: VGM data offset (relative from 0x34); data at 0x100 → rel = 0xCC
        let data_offset_rel = self.header.data_offset.saturating_sub(0x34);
        hdr[0x34..0x38].copy_from_slice(&data_offset_rel.to_le_bytes());
        // Extended chip clocks (VGM 1.51+) — offsets match VGM spec exactly.
        // 0x40: RF5C68 (unused, left 0)
        // 0x44: YM2203
        hdr[0x44..0x48].copy_from_slice(&self.header.ym2203_clock.to_le_bytes());
        // 0x48: YM2608
        hdr[0x48..0x4C].copy_from_slice(&self.header.ym2608_clock.to_le_bytes());
        // 0x4C: YM2610/YM2610B
        hdr[0x4C..0x50].copy_from_slice(&self.header.ym2610b_clock.to_le_bytes());
        // 0x50: YM3812 (OPL2)
        hdr[0x50..0x54].copy_from_slice(&self.header.ym3812_clock.to_le_bytes());
        // 0x54: YM3526 (OPL)
        hdr[0x54..0x58].copy_from_slice(&self.header.ym3526_clock.to_le_bytes());
        // 0x58: Y8950
        hdr[0x58..0x5C].copy_from_slice(&self.header.y8950_clock.to_le_bytes());
        // 0x5C: YMF262 (OPL3)
        hdr[0x5C..0x60].copy_from_slice(&self.header.ymf262_clock.to_le_bytes());
        // 0x60: YMF278B (unused, left 0)
        // 0x64: YMF271
        hdr[0x64..0x68].copy_from_slice(&self.header.ymf271_clock.to_le_bytes());
        // 0x68: YMZ280B (unused, left 0)
        // 0x6C: RF5C164
        hdr[0x6C..0x70].copy_from_slice(&self.header.rf5c164_clock.to_le_bytes());
        // 0x70: PWM (unused, left 0)
        // 0x74: AY8910
        hdr[0x74..0x78].copy_from_slice(&self.header.ay8910_clock.to_le_bytes());
        // Extended chip clocks (VGM 1.60+)
        // 0x80: DMG (Game Boy APU) clock
        hdr[0x80..0x84].copy_from_slice(&self.header.dmg_clock.to_le_bytes());
        // 0x84: NES APU clock
        hdr[0x84..0x88].copy_from_slice(&self.header.nes_apu_clock.to_le_bytes());
        // 0x94: OKIM6295 / K051649 flags
        hdr[0x94..0x98].copy_from_slice(&self.header.k051649_flags.to_le_bytes());
        // 0x9C: K051649 / K052539 clock rate
        hdr[0x9C..0xA0].copy_from_slice(&self.header.k051649_clock.to_le_bytes());
        // 0xA0: K054539 clock
        hdr[0xA0..0xA4].copy_from_slice(&self.header.k054539_clock.to_le_bytes());
        // 0xA4: HuC6280 clock
        hdr[0xA4..0xA8].copy_from_slice(&self.header.huc6280_clock.to_le_bytes());
        // 0xA8: C140 clock
        hdr[0xA8..0xAC].copy_from_slice(&self.header.c140_clock.to_le_bytes());
        // 0xAC: K053260 clock
        hdr[0xAC..0xB0].copy_from_slice(&self.header.k053260_clock.to_le_bytes());
        // 0xB0: Pokey clock
        hdr[0xB0..0xB4].copy_from_slice(&self.header.pokey_clock.to_le_bytes());
        // 0xB4: QSound clock
        hdr[0xB4..0xB8].copy_from_slice(&self.header.qsound_clock.to_le_bytes());
        // Extended chip clocks (VGM 1.70+)
        // 0xD8: C352 clock
        hdr[0xD8..0xDC].copy_from_slice(&self.header.c352_clock.to_le_bytes());
        output.extend_from_slice(&hdr);
        Ok(())
    }

    fn write_commands(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        for cmd in &self.commands {
            output.push(cmd.command_type as u8);
            output.extend_from_slice(&cmd.data);
        }
        output.push(VgmCommandType::End as u8);
        Ok(())
    }

    fn write_gd3_tag(&self, tag: &Gd3Tag, output: &mut Vec<u8>) -> MmlResult<()> {
        let mut gd3_data = vec![b'G', b'd', b'3', b' '];
        gd3_data.push(0x00);

        fn write_gd3_string(data: &mut Vec<u8>, s: &str) {
            for c in s.encode_utf16() {
                data.extend_from_slice(&c.to_le_bytes());
            }
            data.extend_from_slice(&0u16.to_le_bytes());
        }

        write_gd3_string(&mut gd3_data, &tag.track_name_en);
        write_gd3_string(&mut gd3_data, &tag.track_name_jp);
        write_gd3_string(&mut gd3_data, &tag.game_name_en);
        write_gd3_string(&mut gd3_data, &tag.game_name_jp);
        write_gd3_string(&mut gd3_data, &tag.system_name_en);
        write_gd3_string(&mut gd3_data, &tag.system_name_jp);
        write_gd3_string(&mut gd3_data, &tag.author_en);
        write_gd3_string(&mut gd3_data, &tag.author_jp);
        write_gd3_string(&mut gd3_data, &tag.release_date);
        write_gd3_string(&mut gd3_data, &tag.converter);
        write_gd3_string(&mut gd3_data, &tag.notes);

        output.extend_from_slice(&gd3_data);
        Ok(())
    }

    fn write_pcm_data_block(&self, pcm: &PcmData, output: &mut Vec<u8>) -> MmlResult<()> {
        output.push(VgmCommandType::DataBlock as u8);
        output.push(0x00);
        output.extend_from_slice(&pcm.data.len().to_le_bytes());
        output.extend_from_slice(&pcm.data);
        Ok(())
    }

    // ── K051649 (SCC) helpers ─────────────────────────────────────────────────

    /// Write a K051649 register (VGM opcode 0xD2)
    /// pp = port (0 = SCC1, 1 = SCC2), aa = register address, dd = data
    fn k051649_write(&mut self, port: u8, addr: u8, data: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::K051649Write,
            data: vec![port, addr, data],
            time,
        });
    }

    /// Set K051649 waveform for a channel (32 bytes of signed samples)
    fn k051649_set_waveform(&mut self, ch: u8, wave: &[i8; 32], time: u64) {
        let base_addr = ch * 0x20; // Each channel has 32-byte waveform RAM at 0x00, 0x20, 0x40, 0x60, 0x80
        for (i, &sample) in wave.iter().enumerate() {
            self.k051649_write(0, base_addr + i as u8, sample as u8, time);
        }
    }

    /// Convert MIDI note to K051649 frequency divider
    /// SCC uses: period = clock / (freq * 16)
    /// Returns (freq_lo, freq_hi) for registers 0xA0+ch*2 and 0xA1+ch*2
    fn midi_note_to_k051649_freq(&self, midi_note: u8, clock: u32) -> (u8, u8) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let divider = clock as f64 / (16.0 * freq);
        let divider_int = divider.round() as u32;
        let freq_val = divider_int.min(4095) as u16; // 12-bit divider
        ((freq_val & 0xFF) as u8, ((freq_val >> 8) & 0x0F) as u8)
    }

    /// Write K051649 note-on for a channel
    fn k051649_note_on(&mut self, ch: u8, note: u8, octave: u8, volume: u8, time: u64) {
        let clock = self.header.k051649_clock;
        let (freq_lo, freq_hi) = self.midi_note_to_k051649_freq(note, clock);
        // pp=1: frequency write; aa = ch*2 (lo byte), ch*2+1 (hi byte)
        self.k051649_write(1, ch * 2, freq_lo, time);
        self.k051649_write(1, ch * 2 + 1, freq_hi, time);
        // pp=2: volume write; aa = ch (0-4), dd = volume (0-15)
        self.k051649_write(2, ch, volume.min(15), time);
        // pp=3: key on/off; aa = unused, dd = 5-bit channel mask
        self.k051649_key_mask |= 1 << ch;
        self.k051649_write(3, 0x00, self.k051649_key_mask, time);
    }

    /// Write K051649 note-off for a channel
    fn k051649_note_off(&mut self, ch: u8, time: u64) {
        self.k051649_key_mask &= !(1 << ch);
        self.k051649_write(3, 0x00, self.k051649_key_mask, time);
    }

    // ── NES APU (2A03) helpers ────────────────────────────────────────────────

    /// Write a NES APU register (VGM opcode 0xB4)
    /// aa = $4000-relative address, dd = data
    fn nes_apu_write(&mut self, addr: u8, data: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::NesApuWrite,
            data: vec![addr, data],
            time,
        });
    }

    /// Convert MIDI note to NES APU timer period
    /// NES APU: freq = cpu_clock / (16 * (period + 1) * 2)
    /// Returns 11-bit period value
    fn midi_note_to_nes_freq(&self, midi_note: u8, clock: u32) -> u16 {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        // NES timer formula: period = (clock / (16 * 2 * freq)) - 1
        // But we need to find the closest period that gives us the target frequency
        // period = (cpu_clock / (16 * (freq + 1))) / 2 - 1... let's use a simpler approach
        // From VGM spec: period = (CPU clock / (16 * note_freq)) - 1
        let period = (clock as f64 / (16.0 * freq)).round() - 1.0;
        period.max(0.0).min(2047.0) as u16
    }

    /// Write NES Pulse channel note-on
    fn nes_apu_note_on_pulse(&mut self, ch: u8, note: u8, octave: u8, volume: u8, duty: u8, time: u64) {
        let clock = self.header.nes_apu_clock;
        let period = self.midi_note_to_nes_freq(note, clock);
        let base = if ch == 0 { 0x4000 } else { 0x4004 };
        
        // Write duty and volume (0x4000 or 0x4004)
        let duty_volume = ((duty & 0x3) << 6) | (volume & 0xF);
        self.nes_apu_write((base - 0x4000) as u8, duty_volume, time);
        
        // Write sweep (0x4001 or 0x4005) - for now just disable sweep
        self.nes_apu_write((base - 0x4000 + 1) as u8, 0x08, time);
        
        // Write timer low (0x4002 or 0x4006)
        self.nes_apu_write((base - 0x4000 + 2) as u8, (period & 0xFF) as u8, time);
        
        // Write timer high + length counter (0x4003 or 0x4007)
        let length = 0; // For now, no length counter
        self.nes_apu_write((base - 0x4000 + 3) as u8, ((period >> 8) as u8 & 0x07) | ((length & 0xF8) << 3), time);
    }

    /// Write NES Pulse channel note-off (set volume to 0)
    fn nes_apu_note_off_pulse(&mut self, ch: u8, time: u64) {
        let base = if ch == 0 { 0x4000 } else { 0x4004 };
        // Set volume to 0
        self.nes_apu_write((base - 0x4000) as u8, 0, time);
    }

    /// Write NES Triangle channel note-on
    fn nes_apu_note_on_triangle(&mut self, note: u8, octave: u8, time: u64) {
        let clock = self.header.nes_apu_clock;
        let period = self.midi_note_to_nes_freq(note, clock);
        
        // Write linear counter (0x4008)
        self.nes_apu_write(0x08, 0x80, time); // Enable linear counter, no counter value
        
        // Write timer low (0x400A)
        self.nes_apu_write(0x0A, (period & 0xFF) as u8, time);
        
        // Write timer high + length counter (0x400B)
        self.nes_apu_write(0x0B, ((period >> 8) as u8 & 0x07) | 0x80, time);
    }

    /// Write NES Noise channel note-on
    fn nes_apu_note_on_noise(&mut self, period: u8, mode: u8, volume: u8, time: u64) {
        // Write volume and envelope (0x400C)
        self.nes_apu_write(0x0C, (volume & 0xF) << 4, time);
        
        // Write mode and period (0x400E)
        let mode_bit = if mode == 1 { 0x80 } else { 0x00 };
        self.nes_apu_write(0x0E, mode_bit | (period & 0x0F), time);
        
        // Write length counter (0x400F) - for now just start the channel
        self.nes_apu_write(0x0F, 0x00, time);
    }

    /// Write NES global init (silence all channels)
    fn nes_apu_global_init(&mut self) {
        let t = 0u64;
        // Silence all channels
        for addr in [0x4000, 0x4001, 0x4002, 0x4003, 0x4004, 0x4005, 0x4006, 0x4007, 
                      0x4008, 0x4009, 0x400A, 0x400B, 0x400C, 0x400D, 0x400E, 0x400F] {
            self.nes_apu_write((addr - 0x4000) as u8, 0, t);
        }
    }

    // ── DMG (Game Boy) helpers ───────────────────────────────────────────────

    /// Write a DMG register (VGM opcode 0xB3)
    /// aa = $FF10-relative address, dd = data
    fn dmg_write(&mut self, addr: u8, data: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: VgmCommandType::DmgWrite,
            data: vec![addr, data],
            time,
        });
    }

    // ── VRC6 helpers ──────────────────────────────────────────────────────────

    /// VRC6 period = clock / (16 * freq) - 1, 12-bit result
    fn midi_note_to_vrc6_period(&self, midi_note: u8) -> u16 {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let clock = self.header.vrc6_clock as f64;
        let period = (clock / (16.0 * freq) - 1.0).round();
        period.max(0.0).min(4095.0) as u16
    }

    fn vrc6_note_on_pulse(&mut self, ch: u8, note: u8, volume: u8, time: u64) {
        let period = self.midi_note_to_vrc6_period(note);
        let base = if ch == 0 { 0x00u8 } else { 0x10u8 };
        let vol = (volume >> 3) & 0x0F;
        // Control: duty=3 (37.5%), volume
        self.vrc6_write(base as u16, 0x30 | vol, time);
        // Period lo
        self.vrc6_write(base as u16 + 1, (period & 0xFF) as u8, time);
        // Period hi + enable
        self.vrc6_write(base as u16 + 2, 0x80 | ((period >> 8) as u8 & 0x0F), time);
    }

    fn vrc6_note_on_sawtooth(&mut self, note: u8, volume: u8, time: u64) {
        let period = self.midi_note_to_vrc6_period(note);
        let accum = ((volume as u16 * 42) / 127) as u8; // 0-42 range for sawtooth accumulator rate
        self.vrc6_write(0x20, accum & 0x3F, time);
        self.vrc6_write(0x21, (period & 0xFF) as u8, time);
        self.vrc6_write(0x22, 0x80 | ((period >> 8) as u8 & 0x0F), time);
    }

    fn vrc6_note_off(&mut self, ch: u8, time: u64) {
        let base = match ch {
            0 => 0x00u16,
            1 => 0x10u16,
            _ => 0x20u16,
        };
        // Clear enable bit (bit 7 of period-hi register) and zero volume
        self.vrc6_write(base, 0x00, time);
        self.vrc6_write(base + 2, 0x00, time);
    }

    /// Convert MIDI note to DMG frequency
    /// DMG: freq = clock / (32 * (2048 - period)) for non-sweep
    /// Returns (period_low, period_high) for NRx3 and NRx4
    fn midi_note_to_dmg_freq(&self, midi_note: u8) -> (u8, u8) {
        let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
        let clock = self.header.dmg_clock as f64;
        // period = 2048 - (clock / (32 * freq))
        let period = (2048.0 - (clock / (32.0 * freq))).round();
        let period_int = period.max(0.0).min(2047.0) as u16;
        ((period_int & 0xFF) as u8, ((period_int >> 8) & 0x07) as u8)
    }

    /// Write DMG Pulse channel note-on
    fn dmg_note_on_pulse(&mut self, ch: u8, note: u8, octave: u8, volume: u8, duty: u8, time: u64) {
        let (freq_lo, freq_hi) = self.midi_note_to_dmg_freq(note);
        let duty_len  = ((duty & 0x3) << 6) | 0x3F;
        let vol_env   = ((volume & 0xF) << 4) | 0x0F;
        // NRx4: bits 2:0 = period high, bit 7 = trigger
        let freq_hi_trig = (freq_hi & 0x07) | 0x80;
        if ch == 0 {
            // CH1 register map: NR10=0x00 NR11=0x01 NR12=0x02 NR13=0x03 NR14=0x04
            self.dmg_write(0x00, 0x00,          time); // NR10: sweep off
            self.dmg_write(0x01, duty_len,       time); // NR11: duty + length
            self.dmg_write(0x02, vol_env,        time); // NR12: volume + envelope
            self.dmg_write(0x03, freq_lo,        time); // NR13: frequency lo
            self.dmg_write(0x04, freq_hi_trig,   time); // NR14: frequency hi + trigger
        } else {
            // CH2 register map: NR21=0x06 NR22=0x07 NR23=0x08 NR24=0x09
            self.dmg_write(0x06, duty_len,       time); // NR21: duty + length
            self.dmg_write(0x07, vol_env,        time); // NR22: volume + envelope
            self.dmg_write(0x08, freq_lo,        time); // NR23: frequency lo
            self.dmg_write(0x09, freq_hi_trig,   time); // NR24: frequency hi + trigger
        }
    }

    /// Write DMG Pulse channel note-off
    fn dmg_note_off_pulse(&mut self, ch: u8, time: u64) {
        // Clear NRx2 (volume/envelope) to silence the channel
        let reg = if ch == 0 { 0x02 } else { 0x07 };
        self.dmg_write(reg, 0x00, time);
    }

    fn dmg_note_off_wave(&mut self, time: u64) {
        // NR30: disable wave DAC (bit 7 = 0)
        self.dmg_write(0x0A, 0x00, time);
    }

    fn dmg_note_off_noise(&mut self, time: u64) {
        // NR42: clear volume
        self.dmg_write(0x11, 0x00, time);
    }

    /// Write DMG Wave channel note-on
    fn dmg_note_on_wave(&mut self, note: u8, octave: u8, volume: u8, time: u64) {
        let (freq_lo, freq_hi) = self.midi_note_to_dmg_freq(note);

        // NR30: Wave enable (bit 7)
        self.dmg_write(0x0A, 0x80, time);

        // NR31: Sound length (0 = max)
        self.dmg_write(0x0B, 0x00, time);

        // NR32: Output level — bits 6:5: 00=mute, 01=100%, 10=50%, 11=25%
        let level: u8 = if volume == 0 { 0 } else { 1 }; // 100% when audible
        self.dmg_write(0x0C, level << 5, time);

        // NR33: Frequency low
        self.dmg_write(0x0D, freq_lo, time);

        // NR34: bits 2:0 = period high, bit 7 = trigger
        self.dmg_write(0x0E, (freq_hi & 0x07) | 0x80, time);
    }

    /// Write DMG Noise channel note-on
    fn dmg_note_on_noise(&mut self, lfsr_width: u8, period: u8, volume: u8, time: u64) {
        // NR41: Sound length
        self.dmg_write(0x10, 0xFF, time);
        
        // NR42: Initial volume + envelope
        let vol_env = ((volume & 0xF) << 4) | 0x0F;
        self.dmg_write(0x11, vol_env, time);
        
        // NR43: Clock shift + width + trigger
        let width_bit = if lfsr_width == 1 { 0x40 } else { 0x00 };
        self.dmg_write(0x12, width_bit | (period & 0x0F) | 0x80, time);
        
        // NR44: Trigger (bit 7)
        self.dmg_write(0x13, 0x80, time);
    }

    /// Write DMG set wave table (32 nibbles packed into 16 bytes)
    fn dmg_set_wave_table(&mut self, nibbles: &[u8; 32], time: u64) {
        for i in 0..16 {
            let byte = (nibbles[i * 2] & 0x0F) | ((nibbles[i * 2 + 1] & 0x0F) << 4);
            self.dmg_write((0x20 + i as u8), byte, time);
        }
    }

    /// Write DMG sweep register for Pulse1
    fn dmg_set_sweep(&mut self, period: u8, direction: u8, shift: u8, time: u64) {
        let reg = ((period & 0x7) << 4) | ((direction & 0x1) << 3) | (shift & 0x7);
        self.dmg_write(0x00, reg, time);
    }

    /// Write DMG global init
    fn dmg_global_init(&mut self) {
        let t = 0u64;
        // NR52: Power control - turn on sound
        self.dmg_write(0x16, 0x80, t);
        // NR51: Channel enable - enable all channels
        self.dmg_write(0x15, 0xFF, t);
        // NR50: Master volume
        self.dmg_write(0x14, 0x77, t);
        // Initialize all channels to silent state
        for addr in 0xFF10..=0xFF23u16 {
            if addr != 0xFF10 && addr != 0xFF11 && addr != 0xFF16 && addr != 0xFF20 {
                self.dmg_write((addr - 0xFF10) as u8, 0, t);
            }
        }
    }

    /// Generic write helper for most 2-byte VGM commands (addr, data)
    fn generic_chip_write(&mut self, cmd_type: VgmCommandType, addr: u8, data: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: cmd_type,
            data: vec![addr, data],
            time,
        });
    }

    /// Generic write helper for 3-byte VGM commands (port, addr, data)
    fn generic_chip_write_ported(&mut self, cmd_type: VgmCommandType, port: u8, addr: u8, data: u8, time: u64) {
        self.commands.push(VgmCommand {
            command_type: cmd_type,
            data: vec![port, addr, data],
            time,
        });
    }

    /// Write YM2151 (OPM) register
    fn ym2151_write_reg(&mut self, addr: u8, val: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::Ym2151Write, addr, val, time);
    }

    /// Write YM2413 (OPLL) register
    fn ym2413_write_reg(&mut self, addr: u8, val: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::Ym2413Write, addr, val, time);
    }

    /// Write AY8910 register
    fn ay8910_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::Ay8910Write, addr, data, time);
    }

    /// Write HuC6280 (PC Engine) register
    fn huc6280_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::HuC6280Write, addr, data, time);
    }

    /// Write RF5C164 (Sega CD) register
    fn rf5c164_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::Rf5c164Write, addr, data, time);
    }

    /// Write SegaPCM bank/address
    fn segapcm_write(&mut self, bank: u8, addr: u8, data: u8, time: u64) {
        self.generic_chip_write_ported(VgmCommandType::SegaPcmWrite, bank, addr, data, time);
    }

    /// Write C140 (Namco arcade) register
    fn c140_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::C140Write, addr, data, time);
    }

    /// Write C352 (Namco System 21/22) register
    fn c352_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::C352Write, addr, data, time);
    }

    /// Write K053260 (Konami arcade PCM) register
    fn k053260_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::K053260Write, addr, data, time);
    }

    /// Write K054539 (Konami arcade PCM) register with port
    fn k054539_write_ported(&mut self, port: u8, addr: u8, data: u8, time: u64) {
        self.generic_chip_write_ported(VgmCommandType::K054539Write, port, addr, data, time);
    }

    /// Write POKEY (Atari 8-bit) register
    fn pokey_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::PokeyWrite, addr, data, time);
    }

    /// Write VRC6 (Konami NES) register
    fn vrc6_write(&mut self, addr: u16, data: u8, time: u64) {
        // VRC6 addresses are 16-bit but VGM only uses lower byte typically
        self.generic_chip_write(VgmCommandType::Vrc6Write, (addr & 0xFF) as u8, data, time);
    }

    /// Write QSound (Capcom CPS) register
    fn qsound_write(&mut self, addr: u8, data: u8, time: u64) {
        self.generic_chip_write(VgmCommandType::QSoundWrite, addr, data, time);
    }

    // ── Phase 9: Chip-Specific Command Handlers ────────────────────────────

    /// Route chip commands to appropriate handlers
    fn handle_chip_command(
        &mut self,
        command: &str,
        args: &[u32],
        state: &mut PartCodegenState,
        time: u64,
    ) -> MmlResult<()> {
        let cmd_upper = command.to_uppercase();
        
        // Determine chip type from state
        let chip = state.chip.as_deref().unwrap_or("Generic");
        
        // Route to chip-specific or command-specific handlers
        match cmd_upper.as_str() {
            // FM Operator Commands (works for most FM chips)
            "AR" | "DR" | "SR" | "RR" | "SL" | "TL" | "KS" | "ML" | "DT" => {
                self.handle_fm_operator_command(&cmd_upper, args, state, time, chip)?;
            }
            // FM Control Commands
            "AL" | "FB" => {
                self.handle_fm_control_command(&cmd_upper, args, state, time, chip)?;
            }
            // YMF262/OPL3 mode commands
            "OPL3MODE" | "4OP" => {
                self.handle_ymf262_mode_command(&cmd_upper, args, time)?;
            }
            // PSG/AY8910 Commands
            "EN" | "MIX" | "NOISE" => {
                self.handle_ay8910_command(&cmd_upper, args, state, time)?;
            }
            // POKEY Commands
            "FILTER" | "DIST" | "HPOLY" => {
                self.handle_pokey_command(&cmd_upper, args, state, time)?;
            }
            // Wavetable Commands
            "WAVE" | "SW" | "KEYON" | "KEYOFF" | "NW" | "P" => {
                self.handle_wavetable_command(&cmd_upper, args, state, time)?;
            }
            // PCM Commands
            "BANK" | "LOOP" | "START" | "END" |
            "VOLUME" | "REVERSE" | "PAN" | "REVERB" => {
                self.handle_pcm_command(&cmd_upper, args, state, time)?;
            }
            // Special/Meta commands
            "EON" => {
                state.ym2612_port = if args.first().map_or(false, |&a| a != 0) { 1 } else { 0 };
            }
            _ => {
                // Unknown command - silently ignore (can log if needed)
            }
        }
        
        Ok(())
    }

    /// Handle FM operator commands (AR, DR, SR, RR, SL, TL, KS, ML, DT)
    fn handle_fm_operator_command(
        &mut self,
        command: &str,
        args: &[u32],
        state: &PartCodegenState,
        time: u64,
        chip: &str,
    ) -> MmlResult<()> {
        if args.is_empty() {
            return Ok(());
        }
        
        let value = (args[0].min(255)) as u8;
        
        // Map command to register offset within operator data
        // Operator register layout (per YM FM chips):
        // 0x30: AR, 0x31: DR, 0x32: SR, 0x33: RR, 0x34: SL, 0x35: TL, 0x36: KS, 0x37: ML, 0x38: DT
        let reg_offset = match command {
            "AR" => 0x30,
            "DR" => 0x31,
            "SR" => 0x32,
            "RR" => 0x33,
            "SL" => 0x34,
            "TL" => 0x35,
            "KS" => 0x36,
            "ML" => 0x37,
            "DT" => 0x38,
            _ => return Ok(()),
        };
        
        // Write to appropriate chip
        match chip {
            "YM2608" | "OPNA" => {
                self.ym2608_write_reg(0, reg_offset, value, time);
            }
            "YM2151" | "OPM" => {
                self.ym2151_write_reg(reg_offset, value, time);
            }
            "YM2203" | "OPN" => {
                self.ym2203_write_reg(reg_offset, value, time);
            }
            "YM2413" | "OPLL" => {
                self.ym2413_write_reg(reg_offset, value, time);
            }
            "YM3526" | "OPL" | "YM3812" | "OPL2" | "Y8950" | "YMF262" | "OPL3" => {
                // OPL chips use similar register structure
                self.generic_chip_write(VgmCommandType::Ym3812Write, reg_offset, value, time);
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Handle FM control commands (AL, FB)
    fn handle_fm_control_command(
        &mut self,
        command: &str,
        args: &[u32],
        state: &PartCodegenState,
        time: u64,
        chip: &str,
    ) -> MmlResult<()> {
        if args.is_empty() {
            return Ok(());
        }
        
        let value = (args[0].min(255)) as u8;
        
        match command {
            "AL" => {
                // Algorithm selection (0x04 register)
                match chip {
                    "YM2608" | "OPNA" => self.ym2608_write_reg(0, 0x04, value, time),
                    "YM2151" | "OPM" => self.ym2151_write_reg(0x04, value, time),
                    _ => {}
                }
            }
            "FB" => {
                // Feedback level (0x05 register, bits 3-5)
                let feedback = (value & 0x7) << 3;
                match chip {
                    "YM2608" | "OPNA" => self.ym2608_write_reg(0, 0x05, feedback, time),
                    "YM2151" | "OPM" => self.ym2151_write_reg(0x05, feedback, time),
                    "YM3526" | "OPL" | "YM3812" | "OPL2" => {
                        // OPL chips use 0xC0 + channel for feedback
                        let fb_byte = 0xC0 + (state.ym2612_ch & 0x8);
                        self.generic_chip_write(VgmCommandType::Ym3812Write, fb_byte, feedback, time);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Handle AY8910 PSG commands
    fn handle_ay8910_command(
        &mut self,
        command: &str,
        args: &[u32],
        _state: &PartCodegenState,
        time: u64,
    ) -> MmlResult<()> {
        if args.is_empty() {
            return Ok(());
        }
        
        let value = (args[0].min(255)) as u8;
        
        match command {
            "EN" => {
                // Envelope enable (register 0x0D, bit 4)
                let envelope_enable = if value != 0 { 0x10 } else { 0x00 };
                self.ay8910_write(0x0D, envelope_enable, time);
            }
            "MIX" => {
                // Mixer control (register 0x07)
                // Format: TME (bit 7=tone, bit 6=mix_enable, bit 5=envelope)
                let mixer = value & 0xE0;
                self.ay8910_write(0x07, mixer, time);
            }
            "NOISE" => {
                // Noise period (register 0x06, bits 0-4)
                let noise_period = value & 0x1F;
                self.ay8910_write(0x06, noise_period, time);
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Handle POKEY commands
    fn handle_pokey_command(
        &mut self,
        command: &str,
        args: &[u32],
        _state: &PartCodegenState,
        time: u64,
    ) -> MmlResult<()> {
        if args.is_empty() {
            return Ok(());
        }
        
        let value = (args[0].min(255)) as u8;
        
        match command {
            "FILTER" => {
                // Lowpass filter mode (0x2A register, bits 0-1)
                let filter_mode = value & 0x03;
                self.pokey_write(0x2A, filter_mode, time);
            }
            "DIST" => {
                // Distortion mode (0x2B register, bits 0-1)
                let dist_mode = value & 0x03;
                self.pokey_write(0x2B, dist_mode, time);
            }
            "HPOLY" => {
                // High-bit polyphone (AUDCTL @ 0x08, bit 7 = 9-bit poly select).
                // Arg 0 disables, non-zero enables.
                let bit = if value != 0 { 0x80 } else { 0x00 };
                self.pokey_write(0x08, bit, time);
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle wavetable commands
    fn handle_wavetable_command(
        &mut self,
        command: &str,
        args: &[u32],
        state: &PartCodegenState,
        time: u64,
    ) -> MmlResult<()> {
        match command {
            "WAVE" => {
                if args.is_empty() {
                    return Ok(());
                }
                let wave_num = (args[0].min(255)) as u8;
                
                // For HuC6280: waveform select via DRR register
                if state.chip.as_deref() == Some("HuC6280") {
                    self.huc6280_write(0x04, wave_num, time);
                }
                // For K051649: waveform select via register
                else if state.chip.as_deref().map_or(false, |c| c.contains("K051649") || c.contains("SCC")) {
                    self.k051649_write(0, 0x06, wave_num, time);
                }
            }
            "KEYON" | "KEYOFF" => {
                // Manual key control
                let key_on = command == "KEYON";
                let key_byte = if key_on { 0xF0 } else { 0x00 };

                if state.chip.as_deref() == Some("K051649") {
                    self.k051649_write(0, 0x08, key_byte, time);
                }
            }
            "NW" => {
                // HuC6280 noise mode/period for channels 4-5.
                // Reg 0x07 (DDA / noise control on noise-capable channels):
                //   bit 7 = noise enable, bits 0-4 = noise frequency.
                if args.is_empty() { return Ok(()); }
                if state.chip.as_deref() == Some("HuC6280") {
                    let val = args[0] as u8;
                    let enable = if val != 0 { 0x80 } else { 0x00 };
                    let period = val & 0x1F;
                    self.huc6280_write(0x07, enable | period, time);
                }
            }
            "SW" => {
                // DMG NR10 sweep register ($FF10).
                // Args: [time(0-7), direction(0=inc/1=dec), shift(0-7)].
                // NR10 layout: bit 7=0, bits 6-4 = sweep time,
                //              bit 3 = 1 for decrease, 0 for increase,
                //              bits 2-0 = shift.
                if state.chip.as_deref() == Some("DMG") {
                    let sweep_time = args.get(0).copied().unwrap_or(0) as u8 & 0x07;
                    let direction  = args.get(1).copied().unwrap_or(0) as u8 & 0x01;
                    let shift      = args.get(2).copied().unwrap_or(0) as u8 & 0x07;
                    let nr10 = (sweep_time << 4) | (direction << 3) | shift;
                    self.dmg_write(0x00, nr10, time);
                }
            }
            "P" => {
                // DMG NR43 LFSR width selector ($FF22), bit 3.
                // 0 = 15-bit (long noise), 1 = 7-bit (short / metallic).
                if state.chip.as_deref() == Some("DMG") {
                    if args.is_empty() { return Ok(()); }
                    let lfsr = args[0] as u8 & 0x01;
                    self.dmg_write(0x22 - 0x10, lfsr << 3, time);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle YMF262 (OPL3) mode commands: OPL3MODE, 4OP
    fn handle_ymf262_mode_command(
        &mut self,
        command: &str,
        args: &[u32],
        time: u64,
    ) -> MmlResult<()> {
        match command {
            "OPL3MODE" => {
                // Port 1 reg 0x05 bit 0: enable OPL3 (4-op + 18 channel) mode.
                let enable = args.first().map_or(1, |&a| if a != 0 { 1 } else { 0 });
                self.ymf262_write_reg(1, 0x05, enable as u8, time);
            }
            "4OP" => {
                // Port 1 reg 0x04: 4-operator channel-pair connection enables.
                // Bits 0-5 enable 4-op mode for channel pairs 0-5.
                // Arg is the bitmask (0..0x3F).
                let mask = args.first().copied().unwrap_or(0) as u8 & 0x3F;
                self.ymf262_write_reg(1, 0x04, mask, time);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle PCM commands
    fn handle_pcm_command(
        &mut self,
        command: &str,
        args: &[u32],
        state: &PartCodegenState,
        time: u64,
    ) -> MmlResult<()> {
        let chip = state.chip.as_deref().unwrap_or("Generic");
        // First arg as a clamped byte (where applicable). Commands that
        // need 0 args or multiple args validate locally.
        let value = args.first().copied().unwrap_or(0).min(255) as u8;

        match command {
            "BANK" => {
                if args.is_empty() { return Ok(()); }
                match chip {
                    "SegaPCM" => {
                        // Bank select via address high byte
                        self.segapcm_write(value, 0x00, 0x00, time);
                    }
                    "C140" => {
                        self.c140_write(0x1E, value, time);
                    }
                    _ => {}
                }
            }
            "LOOP" => {
                if args.is_empty() { return Ok(()); }
                // Loop enable flag
                let loop_enable = if value != 0 { 0x10 } else { 0x00 };
                match chip {
                    "C140" => self.c140_write(0x1F, loop_enable, time),
                    "C352" => self.c352_write(0x1F, loop_enable, time),
                    "K054539" => self.k054539_write_ported(0, 0x1F, loop_enable, time),
                    _ => {}
                }
            }
            "START" => {
                // Start address. Args: [low] or [low, mid, high]; missing bytes default to 0.
                if args.is_empty() { return Ok(()); }
                let lo = args.first().copied().unwrap_or(0) as u8;
                let mid = args.get(1).copied().unwrap_or(0) as u8;
                let hi = args.get(2).copied().unwrap_or(0) as u8;
                match chip {
                    "RF5C164" => {
                        // Channel start-address registers (high byte at 0x06).
                        self.rf5c164_write(0x06, hi, time);
                    }
                    "SegaPCM" => {
                        // SegaPCM start = (addr_lo, addr_hi) at offsets 0x06/0x07 of channel.
                        self.segapcm_write(0, 0x06, lo, time);
                        self.segapcm_write(0, 0x07, mid, time);
                    }
                    "C140" => {
                        self.c140_write(0x06, lo, time);
                        self.c140_write(0x07, mid, time);
                    }
                    _ => {}
                }
            }
            "END" => {
                if args.is_empty() { return Ok(()); }
                let lo = args.first().copied().unwrap_or(0) as u8;
                let mid = args.get(1).copied().unwrap_or(0) as u8;
                match chip {
                    "SegaPCM" => {
                        self.segapcm_write(0, 0x04, lo, time);
                        self.segapcm_write(0, 0x05, mid, time);
                    }
                    "C140" => {
                        self.c140_write(0x08, lo, time);
                        self.c140_write(0x09, mid, time);
                    }
                    _ => {}
                }
            }
            "VOLUME" => {
                // Stereo volume. Args: [left, right] (0-255). Defaults: right=left.
                if args.is_empty() { return Ok(()); }
                let left = args.first().copied().unwrap_or(0) as u8;
                let right = args.get(1).copied().unwrap_or(args[0]) as u8;
                match chip {
                    "RF5C164" => {
                        // Per-channel envelope (0x00) and pan (0x01).
                        self.rf5c164_write(0x00, left, time);
                        // Pan: high nibble = left, low nibble = right.
                        let pan = (left & 0xF0) | (right >> 4);
                        self.rf5c164_write(0x01, pan, time);
                    }
                    "SegaPCM" => {
                        // SegaPCM channel volume registers 0x02/0x03 (left/right).
                        self.segapcm_write(0, 0x02, left, time);
                        self.segapcm_write(0, 0x03, right, time);
                    }
                    _ => {}
                }
            }
            "REVERSE" => {
                // Play sample in reverse (toggle / explicit).
                let on = if args.is_empty() { true } else { value != 0 };
                let flag = if on { 0x40 } else { 0x00 };
                match chip {
                    "C140" => self.c140_write(0x05, flag, time),
                    "C352" => self.c352_write(0x05, flag, time),
                    "K054539" => self.k054539_write_ported(0, 0x22, flag, time),
                    _ => {}
                }
            }
            "PAN" => {
                if args.is_empty() { return Ok(()); }
                // QSound pan: 16-bit register 0x00 (channel-relative); arg is
                // the signed pan position [-64, +64] mapped to 0x80 center.
                let signed = args[0] as i32;
                let pan = (0x80 + signed.clamp(-128, 127)) as u8;
                match chip {
                    "QSound" => {
                        // Hi/lo pair for the pan register, latched via 0x03 (key-on).
                        self.qsound_write(0x00, 0x00, time);
                        self.qsound_write(0x01, pan, time);
                    }
                    "RF5C164" => {
                        // RF5C164 pan: high nibble = left, low nibble = right.
                        let l = if signed <= 0 { 0xF } else { (0xF as i32 - signed.min(0xF)) as u8 };
                        let r = if signed >= 0 { 0xF } else { (0xF as i32 + signed.max(-0xF)) as u8 };
                        self.rf5c164_write(0x01, (l << 4) | (r & 0xF), time);
                    }
                    _ => {}
                }
            }
            "REVERB" => {
                if args.is_empty() { return Ok(()); }
                // QSound: reverb level is set via global register 0xCD/0xCE.
                if chip == "QSound" {
                    self.qsound_write(0x00, 0x00, time);
                    self.qsound_write(0x01, value, time);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl CodeGenerator for VgmGenerator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        self.generate()
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Vgm
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}
