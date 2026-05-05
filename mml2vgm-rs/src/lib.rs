//! # mml2vgm
//!
//! A library for compiling MML (Music Macro Language) files to VGM/XGM/ZGM formats
//! and playing them back.
//!
//! ## Usage
//!
//! ```no_run
//! use mml2vgm::{OutputFormat, CompileOptions};
//!
//! // MmlCompiler will be implemented in Phase 2
//! // For now, this demonstrates the intended API
//! // let options = CompileOptions::default()
//! //     .with_output_format(OutputFormat::VGM);
//! // let compiler = MmlCompiler::new(options);
//! // let result = compiler.compile("song.gwi")?;
//! // std::fs::write("song.vgm", &result.data)?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::needless_doctest_main)]

pub mod error;
pub mod utils;

// Re-export commonly used types
pub use error::{ErrorContext, MmlError, MmlResult, Position};

// Compiler module (will be implemented in Phase 2)
pub mod compiler;

// Chips module (will be implemented in Phase 4)
pub mod chips;

// Audio module (will be implemented in Phase 5)
pub mod audio;

// Player module (will be implemented in Phase 5)
pub mod player;

// Drivers module (external driver support)
pub mod drivers;

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Standard VGM format
    VGM,
    /// XGM format (Mega Drive)
    XGM,
    /// XGM2 format (Mega Drive with extended features)
    XGM2,
    /// ZGM format (Extended VGM with YM2609 and MIDI support)
    ZGM,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::VGM => write!(f, "vgm"),
            OutputFormat::XGM => write!(f, "xgm"),
            OutputFormat::XGM2 => write!(f, "xgm2"),
            OutputFormat::ZGM => write!(f, "zgm"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = MmlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vgm" => Ok(OutputFormat::VGM),
            "xgm" => Ok(OutputFormat::XGM),
            "xgm2" => Ok(OutputFormat::XGM2),
            "zgm" => Ok(OutputFormat::ZGM),
            _ => Err(MmlError::UnsupportedCommand(format!(
                "Unknown output format: {}. Supported: vgm, xgm, xgm2, zgm",
                s
            ))),
        }
    }
}

/// Supported sound chips
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SoundChip {
    // Mega Drive / Genesis
    YM2612,
    YM2612X,
    YM2612X2,
    
    // PSG
    SN76489,
    SN76489X2,
    
    // PC Engine / TurboGrafx-16
    YM2608,
    
    // YM2609 (Extended)
    YM2609,
    
    // YM2610 / YM2610B
    YM2610B,
    
    // OPM Series
    YM2151,
    
    // OPL Series
    YM3526,
    Y8950,
    YM3812,
    YMF262,
    
    // OPLL
    YM2413,
    
    // Other FM chips
    YM2203,
    
    // PCM chips
    RF5C164,
    SegaPCM,
    
    // Other
    HuC6280,
    C140,
    C352,
    AY8910,
    K051649,
    K053260,
    K054539,
    QSound,
    
    // Console chips
    NES,
    DMG,
    VRC6,
    
    // POKEY (Atari)
    POKEY,
    
    // MIDI
    MIDI,
    
    // Special
    CONDUCTOR,
}

impl SoundChip {
    /// Get the chip name as a string
    pub fn name(&self) -> &'static str {
        match self {
            SoundChip::YM2612 => "YM2612 (OPN2)",
            SoundChip::YM2612X => "YM2612X",
            SoundChip::YM2612X2 => "YM2612X2",
            SoundChip::SN76489 => "SN76489 (DCSG)",
            SoundChip::SN76489X2 => "SN76489X2",
            SoundChip::YM2608 => "YM2608 (OPNA)",
            SoundChip::YM2609 => "YM2609 (OPNA2)",
            SoundChip::YM2610B => "YM2610B (OPNB)",
            SoundChip::YM2151 => "YM2151 (OPM)",
            SoundChip::YM3526 => "YM3526 (OPL)",
            SoundChip::Y8950 => "Y8950",
            SoundChip::YM3812 => "YM3812 (OPL2)",
            SoundChip::YMF262 => "YMF262 (OPL3)",
            SoundChip::YM2413 => "YM2413 (OPLL)",
            SoundChip::YM2203 => "YM2203 (OPN)",
            SoundChip::RF5C164 => "RF5C164",
            SoundChip::SegaPCM => "SegaPCM",
            SoundChip::HuC6280 => "HuC6280",
            SoundChip::C140 => "C140",
            SoundChip::C352 => "C352",
            SoundChip::AY8910 => "AY8910",
            SoundChip::K051649 => "K051649",
            SoundChip::K053260 => "K053260",
            SoundChip::K054539 => "K054539",
            SoundChip::QSound => "QSound",
            SoundChip::NES => "NES APU",
            SoundChip::DMG => "DMG (Game Boy)",
            SoundChip::VRC6 => "VRC6",
            SoundChip::POKEY => "POKEY",
            SoundChip::MIDI => "MIDI",
            SoundChip::CONDUCTOR => "CONDUCTOR",
        }
    }

    /// Get the default clock rate for the chip
    pub fn clock_rate(&self) -> u32 {
        match self {
            SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2 => 7670454,
            SoundChip::SN76489 | SoundChip::SN76489X2 => 3579545,
            SoundChip::YM2608 => 7987200,
            SoundChip::YM2609 => 7987200,
            SoundChip::YM2610B => 8000000,
            SoundChip::YM2151 => 3579545,
            SoundChip::YM3526 => 3579545,
            SoundChip::Y8950 => 3579545,
            SoundChip::YM3812 => 3579545,
            SoundChip::YMF262 => 14318180,
            SoundChip::YM2413 => 3579545,
            SoundChip::YM2203 => 3993600,
            SoundChip::RF5C164 => 12500000,
            SoundChip::SegaPCM => 4000000,
            SoundChip::HuC6280 => 3579545,
            SoundChip::C140 => 8000000,
            SoundChip::C352 => 24192000,
            SoundChip::AY8910 => 1789750,
            SoundChip::K051649 => 1789772,
            SoundChip::K053260 => 3579545,
            SoundChip::K054539 => 18432000,
            SoundChip::QSound => 4000000,
            SoundChip::NES => 1789772,
            SoundChip::DMG => 4194304,
            SoundChip::VRC6 => 1789772,
            SoundChip::POKEY => 1789772,
            SoundChip::MIDI => 0,
            SoundChip::CONDUCTOR => 0,
        }
    }

    /// Check if this chip is a PSG/SSG type
    pub fn is_psg(&self) -> bool {
        matches!(
            self,
            SoundChip::SN76489 | SoundChip::SN76489X2 | SoundChip::AY8910
        )
    }

    /// Check if this chip is an FM synthesis type
    pub fn is_fm(&self) -> bool {
        matches!(
            self,
            SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2
                | SoundChip::YM2608 | SoundChip::YM2609 | SoundChip::YM2610B
                | SoundChip::YM2151 | SoundChip::YM3526 | SoundChip::Y8950
                | SoundChip::YM3812 | SoundChip::YMF262 | SoundChip::YM2413
                | SoundChip::YM2203
        )
    }

    /// Check if this chip supports PCM
    pub fn supports_pcm(&self) -> bool {
        matches!(
            self,
            SoundChip::YM2608 | SoundChip::YM2609 | SoundChip::YM2610B
                | SoundChip::YM2612 | SoundChip::YM2612X | SoundChip::YM2612X2
                | SoundChip::RF5C164 | SoundChip::SegaPCM
                | SoundChip::C140 | SoundChip::C352
                | SoundChip::K053260 | SoundChip::K054539
                | SoundChip::QSound | SoundChip::Y8950
        )
    }
}

impl std::fmt::Display for SoundChip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for SoundChip {
    type Err = MmlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_upper = s.to_uppercase();
        
        // Try to match by various name formats
        if s_upper == "YM2612" || s_upper == "OPN2" {
            Ok(SoundChip::YM2612)
        } else if s_upper == "SN76489" || s_upper == "DCSG" {
            Ok(SoundChip::SN76489)
        } else if s_upper == "YM2608" || s_upper == "OPNA" {
            Ok(SoundChip::YM2608)
        } else if s_upper == "YM2609" || s_upper == "OPNA2" {
            Ok(SoundChip::YM2609)
        } else if s_upper == "YM2610B" || s_upper == "OPNB" {
            Ok(SoundChip::YM2610B)
        } else if s_upper == "YM2151" || s_upper == "OPM" {
            Ok(SoundChip::YM2151)
        } else if s_upper == "YM3526" || s_upper == "OPL" {
            Ok(SoundChip::YM3526)
        } else if s_upper == "YM3812" || s_upper == "OPL2" {
            Ok(SoundChip::YM3812)
        } else if s_upper == "YMF262" || s_upper == "OPL3" {
            Ok(SoundChip::YMF262)
        } else if s_upper == "YM2413" || s_upper == "OPLL" {
            Ok(SoundChip::YM2413)
        } else if s_upper == "YM2203" || s_upper == "OPN" {
            Ok(SoundChip::YM2203)
        } else if s_upper == "RF5C164" {
            Ok(SoundChip::RF5C164)
        } else if s_upper == "SEGAPCM" {
            Ok(SoundChip::SegaPCM)
        } else {
            Err(MmlError::UnsupportedChip(s.to_string()))
        }
    }
}

/// Compilation options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompileOptions {
    /// Output format
    pub format: OutputFormat,

    /// Target sound chips (None = auto-detect from MML)
    #[serde(default)]
    pub target_chips: Option<Vec<SoundChip>>,

    /// Enable verbose output
    #[serde(default)]
    pub verbose: bool,

    /// Enable debug output
    #[serde(default)]
    pub debug: bool,

    /// Output trace information file
    #[serde(default)]
    pub output_trace: bool,

    /// Output VGM file
    #[serde(default = "default_output_vgm")]
    pub output_vgm: bool,

    /// Compression level (0-9, 0 = no compression)
    #[serde(default)]
    pub compression: u8,

    /// Input file encoding (default: UTF-8 with BOM)
    #[serde(default = "default_encoding")]
    pub encoding: String,

    /// Search paths for include files
    #[serde(default)]
    pub include_paths: Vec<String>,

    /// Clock count override (0 = use default from MML)
    #[serde(default)]
    pub clock_count: u32,
}

// Helper functions for default values
fn default_output_vgm() -> bool {
    true
}

fn default_encoding() -> String {
    "utf-8-bom".to_string()
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::VGM,
            target_chips: None,
            verbose: false,
            debug: false,
            output_trace: false,
            output_vgm: true,
            compression: 0,
            encoding: "utf-8-bom".to_string(),
            include_paths: Vec::new(),
            clock_count: 0,
        }
    }
}

impl CompileOptions {
    /// Create new options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Get output format
    pub fn output_format(&self) -> OutputFormat {
        self.format
    }

    /// Set target chips
    pub fn with_target_chips(mut self, chips: Vec<SoundChip>) -> Self {
        self.target_chips = Some(chips);
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable debug output
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable trace output
    pub fn output_trace(mut self, output_trace: bool) -> Self {
        self.output_trace = output_trace;
        self
    }

    /// Set compression level
    pub fn compression(mut self, level: u8) -> Self {
        self.compression = level.clamp(0, 9);
        self
    }

    /// Add include path
    pub fn add_include_path(mut self, path: impl Into<String>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    /// Set clock count
    pub fn clock_count(mut self, count: u32) -> Self {
        self.clock_count = count;
        self
    }
}

/// Compilation result
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// Output data (VGM/XGM/ZGM binary)
    pub data: Vec<u8>,
    
    /// Output file path (if written)
    pub output_path: Option<String>,
    
    /// Warnings generated during compilation
    pub warnings: Vec<ErrorContext>,
    
    /// Information about the compilation
    pub info: CompileInfo,
}

/// Information about the compilation
#[derive(Debug, Clone, Default)]
pub struct CompileInfo {
    /// Number of parts compiled
    pub part_count: usize,
    
    /// Total number of commands generated
    pub command_count: usize,
    
    /// Estimated duration in samples
    pub duration_samples: u64,
    
    /// Estimated duration in seconds
    pub duration_seconds: f64,
    
    /// Chips used
    pub chips_used: Vec<SoundChip>,
    
    /// File format version
    pub format_version: String,
}

/// VGM header structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct VgmHeader {
    /// "Vgm " identifier
    pub identifier: [u8; 4],
    
    /// EOF offset
    pub eof_offset: u32,
    
    /// Version number
    pub version: u32,
    
    /// SN76489 clock
    pub sn76489_clock: u32,
    
    /// YM2413 clock
    pub ym2413_clock: u32,
    
    /// GD3 offset
    pub gd3_offset: u32,
    
    /// Total samples
    pub total_samples: u32,
    
    /// Loop offset
    pub loop_offset: u32,
    
    /// Loop samples
    pub loop_samples: u32,
    
    /// Rate
    pub rate: u32,
    
    /// SN76489 feedback
    pub sn76489_feedback: u16,
    
    /// SN76489 shift register width
    pub sn76489_shift: u8,
    
    /// SN76489 flags
    pub sn76489_flags: u8,
    
    /// YM2612 clock
    pub ym2612_clock: u32,
    
    /// YM2151 clock
    pub ym2151_clock: u32,
    
    // Additional clocks for other chips
    // ...
}

impl VgmHeader {
    /// Create a new VGM header with defaults
    pub fn new() -> Self {
        Self {
            identifier: *b"Vgm ",
            eof_offset: 0,
            version: 0x171, // Version 1.71
            sn76489_clock: 3579545,
            ym2413_clock: 3579545,
            gd3_offset: 0,
            total_samples: 0,
            loop_offset: 0,
            loop_samples: 0,
            rate: 44100,
            sn76489_feedback: 0x0009,
            sn76489_shift: 16,
            sn76489_flags: 0,
            ym2612_clock: 7670454,
            ym2151_clock: 3579545,
        }
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.identifier);
        bytes.extend_from_slice(&self.eof_offset.to_le_bytes());
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.sn76489_clock.to_le_bytes());
        bytes.extend_from_slice(&self.ym2413_clock.to_le_bytes());
        bytes.extend_from_slice(&self.gd3_offset.to_le_bytes());
        bytes.extend_from_slice(&self.total_samples.to_le_bytes());
        bytes.extend_from_slice(&self.loop_offset.to_le_bytes());
        bytes.extend_from_slice(&self.loop_samples.to_le_bytes());
        bytes.extend_from_slice(&self.rate.to_le_bytes());
        bytes.extend_from_slice(&self.sn76489_feedback.to_le_bytes());
        bytes.push(self.sn76489_shift);
        bytes.push(self.sn76489_flags);
        bytes.extend_from_slice(&self.ym2612_clock.to_le_bytes());
        bytes.extend_from_slice(&self.ym2151_clock.to_le_bytes());
        // Reserve space for additional clocks
        bytes.resize(0x80, 0);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_chip_from_str() {
        assert_eq!("YM2612".parse::<SoundChip>().unwrap(), SoundChip::YM2612);
        assert_eq!("OPN2".parse::<SoundChip>().unwrap(), SoundChip::YM2612);
        assert_eq!("SN76489".parse::<SoundChip>().unwrap(), SoundChip::SN76489);
        assert_eq!("DCSG".parse::<SoundChip>().unwrap(), SoundChip::SN76489);
    }

    #[test]
    fn test_sound_chip_name() {
        assert_eq!(SoundChip::YM2612.name(), "YM2612 (OPN2)");
        assert_eq!(SoundChip::SN76489.name(), "SN76489 (DCSG)");
    }

    #[test]
    fn test_sound_chip_clock() {
        assert_eq!(SoundChip::YM2612.clock_rate(), 7670454);
        assert_eq!(SoundChip::SN76489.clock_rate(), 3579545);
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("vgm".parse::<OutputFormat>().unwrap(), OutputFormat::VGM);
        assert_eq!("xgm".parse::<OutputFormat>().unwrap(), OutputFormat::XGM);
        assert_eq!("xgm2".parse::<OutputFormat>().unwrap(), OutputFormat::XGM2);
        assert_eq!("zgm".parse::<OutputFormat>().unwrap(), OutputFormat::ZGM);
    }

    #[test]
    fn test_compile_options_builder() {
        let options = CompileOptions::new()
            .with_output_format(OutputFormat::XGM)
            .verbose(true)
            .compression(6);

        assert_eq!(options.format, OutputFormat::XGM);
        assert!(options.verbose);
        assert_eq!(options.compression, 6);
    }

    #[test]
    fn test_vgm_header_to_bytes() {
        let header = VgmHeader::new();
        let bytes = header.to_bytes();
        
        assert_eq!(&bytes[0..4], b"Vgm ");
        assert_eq!(bytes.len(), 0x80);
    }
}
