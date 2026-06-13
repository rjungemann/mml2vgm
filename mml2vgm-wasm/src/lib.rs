//! WebAssembly bindings for mml2vgm
//!
//! This crate provides WebAssembly bindings to expose the mml2vgm compiler
//! and chip emulators to JavaScript for use in a browser-based IDE.
//!
//! # Usage
//!
//! In JavaScript/TypeScript:
//! ```javascript
//! import init, { compile_mml, validate_mml, get_supported_chips } from 'mml2vgm-wasm';
//!
//! await init();
//!
//! // Compile MML to VGM
//! const mml = `@0 v10 o4 l4 cdefgab>c`;
//! const result = compile_mml(mml, { format: 'vgm' });
//! console.log(result);
//!
//! // Validate MML
//! const isValid = validate_mml(mml);
//! console.log(isValid);
//!
//! // Get supported chips
//! const chips = get_supported_chips();
//! console.log(chips);
//! ```

use mml2vgm::{
    ALL_OUTPUT_FORMATS, ALL_SOUND_CHIPS, CompileOptions, OutputFormat, SoundChip,
    compiler::compiler::MmlCompiler,
    compiler::lexer,
    compiler::sample_resolver::MemorySampleResolver,
    error::MmlError,
};
use std::collections::HashMap as StdHashMap;
use wasm_bindgen::prelude::*;

// ============================================================================
// Compilation Functions
// ============================================================================

/// Compile result structure for WASM
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsCompileResult {
    data: Vec<u8>, // Stored as base64 string in JavaScript
    part_count: usize,
    command_count: usize,
    duration_samples: u64,
    duration_seconds: f64,
    chips_used: String, // JSON array of chip names
    source_map_json: String, // JSON representation of the source map
}

// Implement Default for JsCompileResult
impl Default for JsCompileResult {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            part_count: 0,
            command_count: 0,
            duration_samples: 0,
            duration_seconds: 0.0,
            chips_used: String::new(),
            source_map_json: String::new(),
        }
    }
}

// Getter methods for JsCompileResult to expose fields to JavaScript
#[wasm_bindgen]
impl JsCompileResult {
    /// Get the compiled data as a byte array
    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Get the part count
    pub fn part_count(&self) -> usize {
        self.part_count
    }

    /// Get the command count
    pub fn command_count(&self) -> usize {
        self.command_count
    }

    /// Get the duration in samples
    pub fn duration_samples(&self) -> u64 {
        self.duration_samples
    }

    /// Get the duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        self.duration_seconds
    }

    /// Get the chips used as a JSON string
    pub fn chips_used(&self) -> String {
        self.chips_used.clone()
    }

    /// Get the source map as a JSON string
    pub fn source_map_json(&self) -> String {
        self.source_map_json.clone()
    }
}

/// Compile MML source code to a binary format (VGM/XGM/ZGM)
///
/// # Arguments
/// * `mml` - The MML source code as a string
/// * `options_json` - JSON string containing CompileOptions
///
/// # Returns
/// A Result containing the compiled data and metadata on success,
/// or an error message on failure.
#[wasm_bindgen(catch)]
pub fn compile_mml(mml: &str, options_json: &str) -> Result<JsCompileResult, JsValue> {
    // Parse options from JSON
    let options: CompileOptions = match serde_json::from_str(options_json) {
        Ok(opts) => opts,
        Err(e) => {
            return Err(JsValue::from_str(&format!("Invalid options: {}", e)));
        }
    };

    let compiler = MmlCompiler::new(options);
    
    match compiler.compile_from_source(mml) {
        Ok(result) => {
            let info = result.info;
            let chips_json = serde_json::to_string(&info.chips_used).unwrap();
            let source_map_json = serde_json::to_string(&result.source_map).unwrap_or_default();

            Ok(JsCompileResult {
                data: result.data,
                part_count: info.part_count,
                command_count: info.command_count,
                duration_samples: info.duration_samples,
                duration_seconds: info.duration_seconds,
                chips_used: chips_json,
                source_map_json,
            })
        }
        Err(e) => Err(JsValue::from_str(&format!("Compilation error: {}", e))),
    }
}

/// Compile MML source code with pre-decoded PCM samples.
///
/// # Arguments
/// * `mml` - The MML source code
/// * `options_json` - JSON string containing CompileOptions
/// * `samples_json` - JSON object mapping filename → array of f32 PCM samples,
///   e.g. `{ "str.wav": [0.0, 0.1, ...] }`.  Only VGM output embeds samples;
///   other formats ignore this argument.
///
/// # Returns
/// Same `JsCompileResult` as `compile_mml`.
#[wasm_bindgen(catch)]
pub fn compile_with_samples(
    mml: &str,
    options_json: &str,
    samples_json: &str,
) -> Result<JsCompileResult, JsValue> {
    let options: CompileOptions = match serde_json::from_str(options_json) {
        Ok(opts) => opts,
        Err(e) => return Err(JsValue::from_str(&format!("Invalid options: {}", e))),
    };

    // Deserialise samples map: { "filename.wav": [f32, f32, ...] }
    let raw: StdHashMap<String, Vec<f32>> = match serde_json::from_str(samples_json) {
        Ok(m) => m,
        Err(e) => return Err(JsValue::from_str(&format!("Invalid samples JSON: {}", e))),
    };

    let resolver = MemorySampleResolver::new(raw);
    let compiler = MmlCompiler::new(options);

    match compiler.compile_from_source_with_resolver(mml, &resolver) {
        Ok(result) => {
            let info = result.info;
            let chips_json = serde_json::to_string(&info.chips_used).unwrap();
            let source_map_json = serde_json::to_string(&result.source_map).unwrap_or_default();
            Ok(JsCompileResult {
                data: result.data,
                part_count: info.part_count,
                command_count: info.command_count,
                duration_samples: info.duration_samples,
                duration_seconds: info.duration_seconds,
                chips_used: chips_json,
                source_map_json,
            })
        }
        Err(e) => Err(JsValue::from_str(&format!("Compilation error: {}", e))),
    }
}

/// Validate MML source code without generating output
///
/// # Arguments
/// * `mml` - The MML source code as a string
///
/// # Returns
/// Ok(()) if valid, Err with error message if invalid
#[wasm_bindgen(catch)]
pub fn validate_mml(mml: &str) -> Result<(), JsValue> {
    let options = CompileOptions::default();
    let compiler = MmlCompiler::new(options);
    
    match compiler.validate_from_source(mml) {
        Ok(_) => Ok(()),
        Err(e) => Err(JsValue::from_str(&format!("Validation error: {}", e))),
    }
}

/// Tokenize MML source code for syntax highlighting
///
/// # Arguments
/// * `mml` - The MML source code as a string
///
/// # Returns
/// JSON array of tokens with type, value, and position information
#[wasm_bindgen(catch)]
pub fn tokenize(mml: &str) -> Result<String, JsValue> {
    match lexer::tokenize(mml) {
        Ok(tokens) => {
            let token_infos: Vec<_> = tokens.iter().map(|(token, pos)| {
                serde_json::json!({
                    "type": token_type_name(token),
                    "value": token_to_string(token),
                    "line": pos.line,
                    "column": pos.column
                })
            }).collect();
            Ok(serde_json::to_string(&token_infos).unwrap())
        }
        Err(e) => Err(JsValue::from_str(&format!("Tokenization error: {}", e))),
    }
}

/// Get the string name for a token type
fn token_type_name(token: &mml2vgm::compiler::lexer::Token) -> String {
    match token {
        mml2vgm::compiler::lexer::Token::Number(_) => "number",
        mml2vgm::compiler::lexer::Token::StringLiteral(_) => "string",
        mml2vgm::compiler::lexer::Token::Identifier(_) => "identifier",
        mml2vgm::compiler::lexer::Token::LeftBrace => "left_brace",
        mml2vgm::compiler::lexer::Token::RightBrace => "right_brace",
        mml2vgm::compiler::lexer::Token::Apostrophe => "apostrophe",
        mml2vgm::compiler::lexer::Token::Equals => "equals",
        mml2vgm::compiler::lexer::Token::Comma => "comma",
        mml2vgm::compiler::lexer::Token::LeftBracket => "left_bracket",
        mml2vgm::compiler::lexer::Token::RightBracket => "right_bracket",
        mml2vgm::compiler::lexer::Token::LeftParen => "left_paren",
        mml2vgm::compiler::lexer::Token::RightParen => "right_paren",
        mml2vgm::compiler::lexer::Token::Note(_) => "note",
        mml2vgm::compiler::lexer::Token::Sharp => "sharp",
        mml2vgm::compiler::lexer::Token::Flat => "flat",
        mml2vgm::compiler::lexer::Token::Rest => "rest",
        mml2vgm::compiler::lexer::Token::Duration(_) => "duration",
        mml2vgm::compiler::lexer::Token::Dot => "dot",
        mml2vgm::compiler::lexer::Token::Underscore => "tie",
        mml2vgm::compiler::lexer::Token::GreaterThan => "octave_up",
        mml2vgm::compiler::lexer::Token::LessThan => "octave_down",
        mml2vgm::compiler::lexer::Token::OctaveCommand => "octave_cmd",
        mml2vgm::compiler::lexer::Token::VolumeCommand => "volume_cmd",
        mml2vgm::compiler::lexer::Token::TempoCommand => "tempo_cmd",
        mml2vgm::compiler::lexer::Token::LengthCommand => "length_cmd",
        mml2vgm::compiler::lexer::Token::AtSign => "part_cmd",
        mml2vgm::compiler::lexer::Token::Bar => "bar",
        mml2vgm::compiler::lexer::Token::Comment(_) => "comment",
        mml2vgm::compiler::lexer::Token::Whitespace(_) => "whitespace",
        mml2vgm::compiler::lexer::Token::Eof => "eof",
        _ => "unknown",
    }
    .to_string()
}

/// Convert token to string representation
fn token_to_string(token: &mml2vgm::compiler::lexer::Token) -> String {
    match token {
        mml2vgm::compiler::lexer::Token::Number(n) => n.to_string(),
        mml2vgm::compiler::lexer::Token::StringLiteral(s) => s.clone(),
        mml2vgm::compiler::lexer::Token::Identifier(s) => s.clone(),
        mml2vgm::compiler::lexer::Token::Note(c) => c.to_string(),
        mml2vgm::compiler::lexer::Token::Duration(n) => n.to_string(),
        mml2vgm::compiler::lexer::Token::Comment(s) => s.clone(),
        mml2vgm::compiler::lexer::Token::Whitespace(s) => s.clone(),
        _ => "".to_string(),
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get a list of all supported sound chips
///
/// # Returns
/// JSON array of chip information objects
#[wasm_bindgen(catch)]
pub fn get_supported_chips() -> String {
    let chip_infos: Vec<_> = ALL_SOUND_CHIPS.iter().map(|chip| {
        serde_json::json!({
            "name": chip.name(),
            "variant": format!("{:?}", chip),
            "clockRate": chip.clock_rate(),
            "isPsg": chip.is_psg(),
            "isFm": chip.is_fm(),
            "supportsPcm": chip.supports_pcm(),
            "supportTier": chip.support_tier(),
            "browserCompileDefault": chip.browser_compile_default(),
        })
    }).collect();
    
    serde_json::to_string(&chip_infos).unwrap()
}

/// Get a list of all supported output formats
///
/// # Returns
/// JSON array of format information objects
#[wasm_bindgen(catch)]
pub fn get_supported_formats() -> String {
    let format_infos: Vec<_> = ALL_OUTPUT_FORMATS.iter().map(|fmt| {
        serde_json::json!({
            "name": fmt.to_string(),
            "extension": fmt.extension(),
            "supportTier": fmt.support_tier(),
        })
    }).collect();
    
    serde_json::to_string(&format_infos).unwrap()
}

/// Parse a sound chip name string into a SoundChip enum
///
/// # Arguments
/// * `chip_name` - The chip name (e.g., "YM2612", "OPN2", "SN76489")
///
/// # Returns
/// The SoundChip enum variant, or error if not found
#[wasm_bindgen(catch)]
pub fn parse_sound_chip(chip_name: &str) -> Result<JsValue, JsValue> {
    match chip_name.parse::<SoundChip>() {
        Ok(chip) => {
            Ok(JsValue::from_serde(&serde_json::json!({
                "name": chip.name(),
                "variant": format!("{:?}", chip),
                "clockRate": chip.clock_rate(),
                "isPsg": chip.is_psg(),
                "isFm": chip.is_fm(),
                "supportsPcm": chip.supports_pcm(),
                "supportTier": chip.support_tier(),
                "browserCompileDefault": chip.browser_compile_default(),
            })).unwrap())
        }
        Err(e) => Err(JsValue::from_str(&format!("Invalid chip name: {}", e))),
    }
}

/// Parse an output format string into an OutputFormat enum
///
/// # Arguments
/// * `format_name` - The format name (e.g., "vgm", "xgm")
///
/// # Returns
/// The OutputFormat enum variant, or error if not found
#[wasm_bindgen(catch)]
pub fn parse_output_format(format_name: &str) -> Result<JsValue, JsValue> {
    match format_name.parse::<OutputFormat>() {
        Ok(fmt) => {
            Ok(JsValue::from_serde(&serde_json::json!({
                "name": fmt.to_string(),
                "extension": fmt.extension(),
                "supportTier": fmt.support_tier(),
            })).unwrap())
        }
        Err(e) => Err(JsValue::from_str(&format!("Invalid format name: {}", e))),
    }
}

// ============================================================================
// CompileOptions serialization helpers
// ============================================================================

/// Create default compile options as JSON
#[wasm_bindgen(catch)]
pub fn default_compile_options() -> String {
    serde_json::to_string(&CompileOptions::default()).unwrap()
}

/// Create compile options for a specific format
///
/// # Arguments
/// * `format` - The output format ("vgm", "xgm", "xgm2", "zgm")
#[wasm_bindgen(catch)]
pub fn compile_options_for_format(format: &str) -> Result<String, JsValue> {
    let fmt: OutputFormat = format.parse().map_err(|e| {
        JsValue::from_str(&format!("Invalid format: {}", e))
    })?;
    
    let options = CompileOptions::new().with_output_format(fmt);
    Ok(serde_json::to_string(&options).unwrap())
}

// ============================================================================
// Chip Player Functions (for real-time audio)
// ============================================================================

use mml2vgm::player::chip_player::{ChipPlayer, ChipPlayerState};
use std::sync::Mutex;

/// Opaque handle to a chip player (for use in JavaScript)
#[wasm_bindgen]
pub struct JsChipPlayer {
    player: Mutex<ChipPlayer>,
    sample_rate: u32,
}

/// Create a new chip player for real-time audio generation
///
/// # Arguments
/// * `sample_rate` - The audio sample rate (e.g., 44100)
#[wasm_bindgen]
pub fn create_chip_player(sample_rate: u32) -> JsChipPlayer {
    JsChipPlayer {
        player: Mutex::new(ChipPlayer::new()),
        sample_rate,
    }
}

/// Add a sound chip to the player
///
/// # Arguments
/// * `player` - The chip player handle
/// * `chip_name` - The chip name (e.g., "YM2612")
#[wasm_bindgen(catch)]
pub fn chip_player_add_chip(player: &JsChipPlayer, chip_name: &str) -> Result<(), JsValue> {
    let chip: SoundChip = chip_name.parse().map_err(|e| {
        JsValue::from_str(&format!("Invalid chip name: {}", e))
    })?;
    
    let mut player_guard = player.player.lock().unwrap();
    player_guard.add_chip(chip).map_err(|e| {
        JsValue::from_str(&format!("Failed to add chip: {}", e))
    })?;
    
    Ok(())
}

/// Write to a chip register
///
/// # Arguments
/// * `player` - The chip player handle
/// * `chip_name` - The chip name
/// * `addr` - Register address
/// * `data` - Data to write
#[wasm_bindgen(catch)]
pub fn chip_player_write_register(
    player: &JsChipPlayer,
    chip_name: &str,
    addr: u8,
    data: u8,
) -> Result<(), JsValue> {
    let chip: SoundChip = chip_name.parse().map_err(|e| {
        JsValue::from_str(&format!("Invalid chip name: {}", e))
    })?;
    
    let mut player_guard = player.player.lock().unwrap();
    player_guard.write_register(chip, addr, data).map_err(|e| {
        JsValue::from_str(&format!("Failed to write register: {}", e))
    })?;
    
    Ok(())
}

/// Set the linear mix gain for a chip in the player.
///
/// `gain = 1.0` is unity, `0.0` mutes the chip (its emulator is still clocked,
/// so envelopes/timers stay coherent when the gain is restored). Values are
/// clamped to `[0.0, 4.0]` to bound speaker risk on runaway sliders.
#[wasm_bindgen(catch)]
pub fn chip_player_set_chip_gain(
    player: &JsChipPlayer,
    chip_name: &str,
    gain: f32,
) -> Result<(), JsValue> {
    let chip: SoundChip = chip_name.parse().map_err(|e| {
        JsValue::from_str(&format!("Invalid chip name: {}", e))
    })?;

    let mut player_guard = player.player.lock().unwrap();
    player_guard.set_chip_gain(chip, gain);
    Ok(())
}

/// Get the linear mix gain for a chip. Returns `1.0` if no explicit gain
/// has been set.
#[wasm_bindgen(catch)]
pub fn chip_player_get_chip_gain(
    player: &JsChipPlayer,
    chip_name: &str,
) -> Result<f32, JsValue> {
    let chip: SoundChip = chip_name.parse().map_err(|e| {
        JsValue::from_str(&format!("Invalid chip name: {}", e))
    })?;

    let player_guard = player.player.lock().unwrap();
    Ok(player_guard.get_chip_gain(chip))
}

/// Generate audio samples from the chip player
///
/// # Arguments
/// * `player` - The chip player handle
/// * `num_samples` - Number of stereo samples to generate (total, not per channel)
///
/// # Returns
/// Float32Array containing interleaved stereo samples (left, right, left, right, ...)
#[wasm_bindgen(catch)]
pub fn chip_player_generate_samples(
    player: &JsChipPlayer,
    num_samples: usize,
) -> Result<Vec<f32>, JsValue> {
    let mut player_guard = player.player.lock().unwrap();
    
    // Generate samples from the chip player
    match player_guard.generate_samples(num_samples) {
        Ok(samples) => Ok(samples),
        Err(e) => Err(JsValue::from_str(&format!("Sample generation error: {}", e))),
    }
}

/// Reset the chip player
#[wasm_bindgen(catch)]
pub fn chip_player_reset(player: &JsChipPlayer) -> Result<(), JsValue> {
    let mut player_guard = player.player.lock().unwrap();
    player_guard.reset_all();
    Ok(())
}

/// Get the current state of the chip player
#[wasm_bindgen]
pub fn chip_player_state(player: &JsChipPlayer) -> JsValue {
    let player_guard = player.player.lock().unwrap();
    match player_guard.state() {
        ChipPlayerState::Stopped => JsValue::from_str("stopped"),
        ChipPlayerState::Playing => JsValue::from_str("playing"),
        ChipPlayerState::Paused => JsValue::from_str("paused"),
    }
}

// ============================================================================
// VGM Player Functions
// ============================================================================

use mml2vgm::player::vgm_player::{VgmPlayer, PlayerState};

/// Opaque handle to a VGM player
#[wasm_bindgen]
pub struct JsVgmPlayer {
    player: Mutex<VgmPlayer>,
}

/// Create a new VGM player
#[wasm_bindgen]
pub fn create_vgm_player() -> JsVgmPlayer {
    JsVgmPlayer {
        player: Mutex::new(VgmPlayer::new()),
    }
}

/// Load VGM data into the player
///
/// # Arguments
/// * `player` - The VGM player handle
/// * `data` - The VGM binary data as a Uint8Array
#[wasm_bindgen(catch)]
pub fn vgm_player_load(player: &JsVgmPlayer, data: &[u8]) -> Result<(), JsValue> {
    let mut player_guard = player.player.lock().unwrap();
    player_guard.load(data).map_err(|e| {
        JsValue::from_str(&format!("Failed to load VGM: {}", e))
    })?;
    
    Ok(())
}

/// Play the loaded VGM
#[wasm_bindgen(catch)]
pub fn vgm_player_play(player: &JsVgmPlayer) -> Result<(), JsValue> {
    let mut player_guard = player.player.lock().unwrap();
    player_guard.play().map_err(|e| {
        JsValue::from_str(&format!("Failed to play: {}", e))
    })?;
    
    Ok(())
}

/// Stop playback
#[wasm_bindgen(catch)]
pub fn vgm_player_stop(player: &JsVgmPlayer) -> Result<(), JsValue> {
    let mut player_guard = player.player.lock().unwrap();
    player_guard.stop().map_err(|e| {
        JsValue::from_str(&format!("Failed to stop: {}", e))
    })?;
    
    Ok(())
}

/// Get the current state of the VGM player
#[wasm_bindgen]
pub fn vgm_player_state(player: &JsVgmPlayer) -> JsValue {
    let player_guard = player.player.lock().unwrap();
    match player_guard.state() {
        PlayerState::Stopped => JsValue::from_str("stopped"),
        PlayerState::Playing => JsValue::from_str("playing"),
        PlayerState::Paused => JsValue::from_str("paused"),
    }
}

/// Get information about the loaded VGM
///
/// # Returns
/// JSON object with VGM info (duration, sample rate, chips used, etc.)
#[wasm_bindgen(catch)]
pub fn vgm_player_get_info(player: &JsVgmPlayer) -> Result<String, JsValue> {
    let player_guard = player.player.lock().unwrap();
    
    // Create info object from player header
    let header = match player_guard.header() {
        Some(h) => h,
        None => return Err(JsValue::from_str("No VGM loaded")),
    };
    
    // Copy fields to avoid alignment issues with packed struct
    let total_samples = header.total_samples;
    let loop_samples = header.loop_samples;
    let rate = header.rate;
    let version = header.version;
    let sn76489_clock = header.sn76489_clock;
    let ym2413_clock = header.ym2413_clock;
    let ym2612_clock = header.ym2612_clock;
    let ym2151_clock = header.ym2151_clock;
    
    Ok(serde_json::json!({
        "total_samples": total_samples,
        "loop_samples": loop_samples,
        "sample_rate": rate,
        "version": version,
        "sn76489_clock": sn76489_clock,
        "ym2413_clock": ym2413_clock,
        "ym2612_clock": ym2612_clock,
        "ym2151_clock": ym2151_clock,
    }).to_string())
}

// ============================================================================
// External Driver Support
// ============================================================================

use mml2vgm::drivers::{
    DriverCompileOptions, DriverCompileResult, DriverDiagnostic, DriverInfo, DriverRegistry,
    DriverOutputFormat, DriverToken, ExternalDriver,
};
use std::collections::HashMap;
use std::sync::Arc;

/// WASM-compatible driver registry
#[wasm_bindgen]
pub struct JsDriverRegistry {
    registry: DriverRegistry,
}

#[wasm_bindgen]
impl JsDriverRegistry {
    /// Create a new driver registry
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut registry = DriverRegistry::new();

        // Register the native GWI driver
        registry.register_driver(Arc::new(mml2vgm::drivers::GwiDriver));

        // Register the M98 driver
        registry.register_driver(Arc::new(mml2vgm::drivers::m98::M98Driver));

        // Register the Mucom driver
        registry.register_driver(Arc::new(mml2vgm::drivers::mucom::MucomDriver));

        // Register the MoonDriver driver
        registry.register_driver(Arc::new(mml2vgm::drivers::moondriver::MoonDriver));

        // Register the PMD driver
        registry.register_driver(Arc::new(mml2vgm::drivers::pmd::PMDDriver));

        // Register the Muap driver
        registry.register_driver(Arc::new(mml2vgm::drivers::muap::MuapDriver));

        JsDriverRegistry { registry }
    }

    /// List all registered drivers as JSON
    #[wasm_bindgen(catch)]
    pub fn list_drivers(&self) -> Result<String, JsValue> {
        let infos = self.registry.get_driver_infos();
        let json: Vec<_> = infos
            .iter()
            .map(|info| {
                serde_json::json!({
                    "id": info.id,
                    "displayName": info.display_name,
                    "extensions": info.supported_extensions,
                    "description": info.description,
                    "version": info.version,
                    "targetPlatform": info.target_platform,
                })
            })
            .collect();
        Ok(serde_json::to_string(&json).unwrap())
    }

    /// Detect the format of the given content
    /// Returns JSON with driverId and confidence
    #[wasm_bindgen(catch)]
    pub fn detect_format(&self, content: &str, filename: Option<String>) -> Result<String, JsValue> {
        let file_ref = filename.as_deref();
        match self.registry.detect_format(content, file_ref) {
            Some((driver_id, confidence)) => {
                Ok(serde_json::json!({
                    "driverId": driver_id,
                    "confidence": confidence,
                })
                .to_string())
            }
            None => Ok(serde_json::json!({
                "driverId": null,
                "confidence": 0,
            })
            .to_string()),
        }
    }

    /// Get driver by file extension
    /// Returns JSON with driver info or null
    #[wasm_bindgen(catch)]
    pub fn get_driver_by_extension(&self, extension: &str) -> Result<String, JsValue> {
        match self.registry.get_driver_by_extension(extension) {
            Some(driver) => {
                let info = driver.info();
                Ok(serde_json::json!({
                    "id": info.id,
                    "displayName": info.display_name,
                    "extensions": info.supported_extensions,
                    "description": info.description,
                    "version": info.version,
                    "targetPlatform": info.target_platform,
                })
                .to_string())
            }
            None => Ok(serde_json::json!(null).to_string()),
        }
    }

    /// Check if a driver is available
    #[wasm_bindgen]
    pub fn has_driver(&self, id: &str) -> bool {
        self.registry.has_driver(id)
    }

    /// Get detailed information about a specific driver by ID
    /// Returns JSON with full driver info or null
    #[wasm_bindgen(catch)]
    pub fn get_driver_info(&self, id: &str) -> Result<String, JsValue> {
        match self.registry.get_driver(id) {
            Some(driver) => {
                let info = driver.info();
                Ok(serde_json::json!({
                    "id": info.id,
                    "displayName": info.display_name,
                    "extensions": info.supported_extensions,
                    "description": info.description,
                    "version": info.version,
                    "targetPlatform": info.target_platform,
                })
                .to_string())
            }
            None => Ok(serde_json::json!(null).to_string()),
        }
    }

    /// Get all drivers with their detailed information
    /// Returns JSON array of driver info objects
    #[wasm_bindgen(catch)]
    pub fn get_all_drivers_info(&self) -> Result<String, JsValue> {
        let drivers = self.registry.get_all_drivers();
        let infos: Vec<_> = drivers.iter()
            .map(|d| {
                let info = d.info();
                serde_json::json!({
                    "id": info.id,
                    "displayName": info.display_name,
                    "extensions": info.supported_extensions,
                    "description": info.description,
                    "version": info.version,
                    "targetPlatform": info.target_platform,
                })
            })
            .collect();
        Ok(serde_json::to_string(&infos).unwrap())
    }
}

/// WASM-compatible compile options
#[wasm_bindgen]
pub struct JsDriverCompileOptions {
    output_format: String,
    sample_rate: u32,
    verbose: bool,
    debug: bool,
    extra: HashMap<String, String>,
}

#[wasm_bindgen]
impl JsDriverCompileOptions {
    /// Create new compile options with defaults
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            output_format: "vgm".to_string(),
            sample_rate: 44100,
            verbose: false,
            debug: false,
            extra: HashMap::new(),
        }
    }

    /// Set output format
    #[wasm_bindgen]
    pub fn set_output_format(&mut self, format: &str) {
        self.output_format = format.to_string();
    }

    /// Set sample rate
    #[wasm_bindgen]
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.sample_rate = rate;
    }

    /// Set verbose mode
    #[wasm_bindgen]
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Set debug mode
    #[wasm_bindgen]
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Add extra option
    #[wasm_bindgen]
    pub fn add_extra(&mut self, key: &str, value: &str) {
        self.extra.insert(key.to_string(), value.to_string());
    }
}

/// Convert JS compile options to Rust compile options
fn js_to_driver_compile_options(js_options: &JsDriverCompileOptions) -> DriverCompileOptions {
    let output_format: DriverOutputFormat = js_options
        .output_format
        .parse()
        .unwrap_or(DriverOutputFormat::VGM);

    DriverCompileOptions {
        output_format,
        sample_rate: js_options.sample_rate,
        verbose: js_options.verbose,
        debug: js_options.debug,
        extra: js_options.extra.clone(),
    }
}

/// Compile MML using a specific driver
#[wasm_bindgen(catch)]
pub fn driver_compile(
    registry: &JsDriverRegistry,
    driver_id: &str,
    content: &str,
    options: &JsDriverCompileOptions,
) -> Result<Vec<u8>, JsValue> {
    let driver = registry
        .registry
        .get_driver(driver_id)
        .ok_or_else(|| JsValue::from_str(&format!("Driver '{}' not found", driver_id)))?;

    let compile_options = js_to_driver_compile_options(options);

    match driver.compile(content, &compile_options) {
        Ok(result) => {
            // Create a JSON result with data and metadata
            let metadata = serde_json::json!({
                "partCount": result.part_count,
                "commandCount": result.command_count,
                "durationSamples": result.duration_samples,
                "durationSeconds": result.duration_seconds,
                "chipsUsed": result.chips_used,
            });

            // Combine data and metadata
            let mut combined = Vec::new();
            combined.extend_from_slice(&metadata.to_string().into_bytes());
            combined.push(0); // null separator
            combined.extend(result.data);

            Ok(combined)
        }
        Err(e) => Err(JsValue::from_str(&format!("Compilation error: {}", e))),
    }
}

/// Validate MML using a specific driver
#[wasm_bindgen(catch)]
pub fn driver_validate(
    registry: &JsDriverRegistry,
    driver_id: &str,
    content: &str,
) -> Result<String, JsValue> {
    let driver = registry
        .registry
        .get_driver(driver_id)
        .ok_or_else(|| JsValue::from_str(&format!("Driver '{}' not found", driver_id)))?;

    match driver.validate(content) {
        Ok(diagnostics) => {
            let json: Vec<_> = diagnostics
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "message": d.message,
                        "severity": match d.severity {
                            mml2vgm::drivers::DiagnosticSeverity::Error => "error",
                            mml2vgm::drivers::DiagnosticSeverity::Warning => "warning",
                            mml2vgm::drivers::DiagnosticSeverity::Info => "info",
                            mml2vgm::drivers::DiagnosticSeverity::Hint => "hint",
                        },
                        "line": d.line,
                        "column": d.column,
                        "length": d.length,
                    })
                })
                .collect();
            Ok(serde_json::to_string(&json).unwrap())
        }
        Err(e) => Err(JsValue::from_str(&format!("Validation error: {}", e))),
    }
}

/// Tokenize MML using a specific driver
#[wasm_bindgen(catch)]
pub fn driver_tokenize(
    registry: &JsDriverRegistry,
    driver_id: &str,
    content: &str,
) -> Result<String, JsValue> {
    let driver = registry
        .registry
        .get_driver(driver_id)
        .ok_or_else(|| JsValue::from_str(&format!("Driver '{}' not found", driver_id)))?;

    match driver.tokenize(content) {
        Ok(tokens) => {
            let json: Vec<_> = tokens
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": t.token_type,
                        "value": t.value,
                        "line": t.line,
                        "column": t.column,
                        "length": t.length,
                    })
                })
                .collect();
            Ok(serde_json::to_string(&json).unwrap())
        }
        Err(e) => Err(JsValue::from_str(&format!("Tokenization error: {}", e))),
    }
}
