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
    CompileOptions, OutputFormat, SoundChip, 
    compiler::compiler::MmlCompiler,
    compiler::lexer,
    error::MmlError,
};
use wasm_bindgen::prelude::*;

// ============================================================================
// Compilation Functions
// ============================================================================

/// Compile result structure for WASM
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsCompileResult {
    data: Vec<u8>,
    part_count: usize,
    command_count: usize,
    duration_samples: u64,
    duration_seconds: f64,
    chips_used: String, // JSON array of chip names
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
        }
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
            
            Ok(JsCompileResult {
                data: result.data,
                part_count: info.part_count,
                command_count: info.command_count,
                duration_samples: info.duration_samples,
                duration_seconds: info.duration_seconds,
                chips_used: chips_json,
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
    let chips = vec![
        SoundChip::YM2612, SoundChip::YM2612X, SoundChip::YM2612X2,
        SoundChip::SN76489, SoundChip::SN76489X2,
        SoundChip::YM2608,
        SoundChip::YM2609,
        SoundChip::YM2610B,
        SoundChip::YM2151,
        SoundChip::YM3526,
        SoundChip::Y8950,
        SoundChip::YM3812,
        SoundChip::YMF262,
        SoundChip::YM2413,
        SoundChip::YM2203,
        SoundChip::RF5C164,
        SoundChip::SegaPCM,
        SoundChip::HuC6280,
        SoundChip::C140,
        SoundChip::C352,
        SoundChip::AY8910,
        SoundChip::K051649,
        SoundChip::K053260,
        SoundChip::K054539,
        SoundChip::QSound,
        SoundChip::NES,
        SoundChip::DMG,
        SoundChip::VRC6,
        SoundChip::POKEY,
        SoundChip::MIDI,
    ];
    
    let chip_infos: Vec<_> = chips.iter().map(|chip| {
        serde_json::json!({
            "name": chip.name(),
            "variant": format!("{:?}", chip),
            "clockRate": chip.clock_rate(),
            "isPsg": chip.is_psg(),
            "isFm": chip.is_fm(),
            "supportsPcm": chip.supports_pcm(),
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
    let formats = vec![
        OutputFormat::VGM,
        OutputFormat::XGM,
        OutputFormat::XGM2,
        OutputFormat::ZGM,
    ];
    
    let format_infos: Vec<_> = formats.iter().map(|fmt| {
        serde_json::json!({
            "name": format!("{:?}", fmt),
            "extension": fmt.to_string(),
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
                "name": format!("{:?}", fmt),
                "extension": fmt.to_string(),
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
