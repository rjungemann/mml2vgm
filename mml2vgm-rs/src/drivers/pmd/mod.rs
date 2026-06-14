//! PMD External Driver
//!
//! This module implements the PMD (PMDotNET) MML format driver for NEC PC-9801 systems.
//! PMD is a popular MML compiler for the Japanese PC-9801 scene, supporting
//! YM2203 and YM2608 sound chips with advanced features like rhythm sections,
//! ADPCM samples, and complex part management.
//!
//! ## Format Specification
//!
//! - **File Extensions**: `.mdl`, `.mus`
//! - **Target Platform**: NEC PC-9801
//! - **Supported Chips**: YM2203 (3 FM + 3 SSG), YM2608 (6 FM + 3 SSG + 6 rhythm + ADPCM)
//!
//! ## PMD Command Reference
//!
//! Based on the PMDotNET driver from the .NET IDE.
//!
//! ### Basic Structure
//! - Parts are defined with `@` commands
//! - Each file can have multiple parts for different channels
//! - Supports both YM2203 and YM2608
//!
//! ### Note Format
//! - Notes: `C`, `C+`, `C#`, `D`, `D+`, `D#`, `E`, `F`, `F+`, `F#`, `G`, `G+`, `G#`, `A`, `A+`, `A#`, `B`
//! - Rest: `R`
//! - Duration: Number after note (e.g., `C4` = quarter note)
//! - Dotted notes: `C4.`
//! - Tied notes: `C4_`
//!
//! ### Octave
//! - `O3`, `O4`, `O5`, etc. (default O4)
//! - `>` = octave up, `<` = octave down
//!
//! ### Volume
//! - `V15` (0-15 for FM, 0-15 for SSG)
//! - `V+` / `V-` = volume up/down
//!
//! ### Commands
//! - `L4` = length (1-256, default 4)
//! - `T120` = tempo (40-255, default 120)
//! - `Q4` = gate time (0-7)
//! - `@n` = part/channel select (n = 0-255)
//! - `(n` = loop start, `)n` = loop end (finite loop)
//! - `[` = infinite loop start, `]` = infinite loop end
//! - `Nn` = note shift
//! - `@@` = part end
//! - `;` or `*` = comment to end of line
//!
//! ### PMD-Specific Features
//! - `@MUSIC` = Begin music data
//! - `@PPZ` = PPZ format marker
//! - `@PARTn` = Define part n
//! - `@VOICE n` = Set voice/instrument
//! - `@RHYTHM` = Rhythm section definition
//! - `@ADPCM` = ADPCM sample definition
//! - `@TEMPO n` = Set tempo
//! - `@VOLUME n` = Set volume
//!
//! ### Rhythm Section
//! - PMD supports a dedicated rhythm section with 6 channels on YM2608
//! - Rhythm instruments: BD, SD, TOM, HH, CYM, RIM
//!
//! ### ADPCM Support
//! - YM2608 can play ADPCM samples
//! - Samples are referenced by number

use crate::drivers::{
    DiagnosticSeverity, DriverCompileOptions, DriverCompileResult, DriverDiagnostic,
    DriverOutputFormat, DriverToken, ExternalDriver,
};
use crate::{error::MmlError, CompileOptions, OutputFormat, SoundChip};

/// PMD Driver implementation
pub struct PMDDriver;

impl ExternalDriver for PMDDriver {
    fn id(&self) -> &str {
        "pmd"
    }

    fn display_name(&self) -> &str {
        "PMD (PC-9801)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".mdl", ".mus"]
    }

    fn description(&self) -> &str {
        "PMD MML format for NEC PC-9801 (YM2203/YM2608)"
    }

    fn target_platform(&self) -> &str {
        "NEC PC-9801"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, content: &str, filename: Option<&str>) -> u8 {
        // Check filename first
        if let Some(name) = filename {
            let name_lower = name.to_lowercase();
            if name_lower.ends_with(".mus") {
                return 90;
            }
            if name_lower.ends_with(".mdl") {
                return 80; // .mdl can also be MoonDriver, so slightly lower confidence
            }
        }

        let content_lower = content.to_lowercase();
        let content_trimmed = content.trim();

        // High confidence: PMD-specific directives
        if content_trimmed.starts_with("@music") || content_lower.contains("@music") {
            return 95;
        }

        // High confidence: PPZ format marker
        if content_lower.contains("@ppz") {
            return 95;
        }

        // High confidence: PMD mention
        if content_lower.contains("pmd") && !content_lower.contains("mucompmd") {
            return 90;
        }

        // Medium confidence: PMD-specific patterns
        if content_lower.contains("@part") || content_lower.contains("@voice") {
            return 80;
        }

        // Medium confidence: rhythm section
        if content_lower.contains("@rhythm") || content_lower.contains("@adpcm") {
            return 80;
        }

        // Medium confidence: YM2203/YM2608 specific
        if content_lower.contains("ym2203") || content_lower.contains("ym2608") {
            return 70;
        }

        // Medium confidence: PC-9801 mention
        if content_lower.contains("pc-98") || content_lower.contains("pc98") {
            return 65;
        }

        // Low confidence: rhythm instrument names
        if content_lower.contains("bd")
            || content_lower.contains("sd")
            || content_lower.contains("tom")
            || content_lower.contains("hh")
            || content_lower.contains("cym")
            || content_lower.contains("rim")
        {
            return 30;
        }

        0
    }

    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError> {
        // Parse and validate the PMD content
        match pmd_parse(content) {
            Ok(_) => Ok(Vec::new()),
            Err(errors) => {
                let diagnostics: Vec<DriverDiagnostic> = errors
                    .into_iter()
                    .map(|e| DriverDiagnostic {
                        message: e.message,
                        severity: DiagnosticSeverity::Error,
                        line: e.line,
                        column: e.column,
                        length: e.length,
                    })
                    .collect();
                Ok(diagnostics)
            }
        }
    }

    fn tokenize(&self, content: &str) -> Result<Vec<DriverToken>, MmlError> {
        match pmd_tokenize(content) {
            Ok(tokens) => Ok(tokens
                .into_iter()
                .map(|t| DriverToken {
                    token_type: t.token_type,
                    value: t.value,
                    line: t.line,
                    column: t.column,
                    length: t.length,
                })
                .collect()),
            Err(_) => Ok(Vec::new()),
        }
    }

    fn compile(
        &self,
        content: &str,
        options: &DriverCompileOptions,
    ) -> Result<DriverCompileResult, MmlError> {
        // Parse the PMD content (ignore errors for now)
        let _ast = pmd_parse(content);

        // Convert output format
        let output_format = match options.output_format {
            DriverOutputFormat::VGM => OutputFormat::VGM,
            DriverOutputFormat::XGM => OutputFormat::XGM,
            DriverOutputFormat::XGM2 => OutputFormat::XGM2,
            DriverOutputFormat::ZGM => OutputFormat::ZGM,
        };

        // Detect target chip from content or use YM2608 as default (most common for PMD)
        let target_chips = detect_target_chips(content);

        // Create compile options
        let compile_options = CompileOptions {
            format: output_format,
            target_chips: Some(target_chips),
            ..Default::default()
        };

        let compiler = crate::compiler::compiler::MmlCompiler::new(compile_options);

        match compiler.compile_from_source(content) {
            Ok(result) => {
                let info = result.info;
                Ok(DriverCompileResult {
                    data: result.data,
                    part_count: info.part_count,
                    command_count: info.command_count,
                    duration_samples: info.duration_samples,
                    duration_seconds: info.duration_seconds,
                    chips_used: info
                        .chips_used
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect(),
                    warnings: Vec::new(),
                })
            }
            Err(e) => Err(e),
        }
    }
}

/// Detect target chips from PMD content
fn detect_target_chips(content: &str) -> Vec<SoundChip> {
    let content_lower = content.to_lowercase();

    // Check for YM2608 (most common for PMD with rhythm/ADPCM)
    if content_lower.contains("ym2608") || content_lower.contains("opna") {
        return vec![SoundChip::YM2608, SoundChip::SN76489];
    }

    // Check for YM2203
    if content_lower.contains("ym2203") {
        return vec![SoundChip::YM2203, SoundChip::SN76489];
    }

    // Default to YM2608 for PMD (most PMD files target YM2608)
    vec![SoundChip::YM2608, SoundChip::SN76489]
}

// ============================================================================
// PMD Tokenizer
// ============================================================================

/// Token for PMD syntax highlighting
#[derive(Debug, Clone)]
pub struct PMDToken {
    /// Token type.
    pub token_type: String,
    /// Value.
    pub value: String,
    /// Line.
    pub line: usize,
    /// Column.
    pub column: usize,
    /// Length.
    pub length: usize,
}

/// Tokenize PMD content for syntax highlighting
fn pmd_tokenize(content: &str) -> Result<Vec<DriverToken>, MmlError> {
    let mut tokens = Vec::new();
    let mut chars = content.chars().peekable();
    let mut line = 1usize;
    let mut column = 1usize;

    while let Some(c) = chars.next() {
        let start_line = line;
        let start_column = column;

        // Handle newlines
        if c == '\n' {
            line += 1;
            column = 1;
            continue;
        }

        if c == '\r' {
            continue;
        }

        // Skip whitespace
        if c.is_whitespace() {
            column += 1;
            continue;
        }

        // Notes: A-G, a-g, with optional + or #
        if ('A'..='G').contains(&c) || ('a'..='g').contains(&c) {
            let note_upper = c.to_ascii_uppercase();
            let mut note_str = note_upper.to_string();
            let mut token_type = "note".to_string();
            let mut length = 1;

            // Check for + or # (sharp)
            if let Some(&next_c) = chars.peek() {
                if next_c == '+' || next_c == '#' {
                    note_str.push(chars.next().unwrap());
                    token_type = "note_sharp".to_string();
                    length += 1;
                }
            }

            tokens.push(create_driver_token(
                token_type,
                note_str,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Rest
        if c == 'R' || c == 'r' {
            let mut value = "r".to_string();
            let mut length = 1;

            // Check for duration number
            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            tokens.push(create_driver_token(
                "rest".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Part command (@) or part end (@@)
        if c == '@' {
            // Check for @@ (part end marker) first
            if let Some(&next_c) = chars.peek() {
                if next_c == '@' {
                    chars.next(); // consume the second @
                    tokens.push(create_driver_token(
                        "part_end".to_string(),
                        "@@".to_string(),
                        start_line,
                        start_column,
                        2,
                    ));
                    column += 2;
                    continue;
                }
            }

            // Single @ - check for directives or part number
            let mut value = "@".to_string();
            let mut length = 1;

            // Check for special directives
            if let Some(&next_c) = chars.peek() {
                if next_c.is_alphabetic() {
                    // Read directive name
                    while let Some(&d) = chars.peek() {
                        if d.is_alphabetic() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                    // Check for @MUSIC, @PPZ, @PART, @VOICE, @RHYTHM, @ADPCM, @TEMPO, @VOLUME
                    let directive_upper = value.to_uppercase();
                    if directive_upper == "@MUSIC"
                        || directive_upper == "@PPZ"
                        || directive_upper == "@RHYTHM"
                        || directive_upper == "@ADPCM"
                        || directive_upper == "@TEMPO"
                        || directive_upper == "@VOLUME"
                    {
                        // Skip whitespace and read argument if present
                        while let Some(&next_c) = chars.peek() {
                            if next_c.is_whitespace() {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        // Read number argument
                        if let Some(&next_c) = chars.peek() {
                            if next_c.is_ascii_digit() {
                                while let Some(&d) = chars.peek() {
                                    if d.is_ascii_digit() {
                                        value.push(chars.next().unwrap());
                                        length += 1;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        tokens.push(create_driver_token(
                            "directive".to_string(),
                            value,
                            start_line,
                            start_column,
                            length,
                        ));
                        column += length;
                        continue;
                    }
                } else if next_c.is_ascii_digit() {
                    // Read part number
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                    tokens.push(create_driver_token(
                        "part_cmd".to_string(),
                        value,
                        start_line,
                        start_column,
                        length,
                    ));
                    column += length;
                    continue;
                }
            }

            // Just @ by itself
            tokens.push(create_driver_token(
                "part_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Octave command (O or o)
        if c == 'O' || c == 'o' {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    value.push(chars.next().unwrap());
                    length += 1;
                }
            }

            tokens.push(create_driver_token(
                "octave_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Volume command (V or v)
        if c == 'V' || c == 'v' {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c == '+' || next_c == '-' {
                    value.push(chars.next().unwrap());
                    length += 1;
                } else if next_c.is_ascii_digit() {
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            tokens.push(create_driver_token(
                "volume_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Length command (L or l)
        if c == 'L' || c == 'l' {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            tokens.push(create_driver_token(
                "length_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Tempo command (T or t)
        if c == 'T' || c == 't' {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            tokens.push(create_driver_token(
                "tempo_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Quantize (Q or q)
        if c == 'Q' || c == 'q' {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    value.push(chars.next().unwrap());
                    length += 1;
                }
            }

            tokens.push(create_driver_token(
                "quantize_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Loop commands
        if c == '(' || c == ')' || c == '[' || c == ']' {
            let token_type = match c {
                '(' => "loop_start",
                ')' => "loop_end",
                '[' => "loop_start_infinite",
                ']' => "loop_end_infinite",
                _ => "unknown",
            };

            // Check for loop count on (
            let mut value = c.to_string();
            let mut length = 1;

            if c == '(' {
                if let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() {
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() {
                                value.push(chars.next().unwrap());
                                length += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            tokens.push(create_driver_token(
                token_type.to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Note shift (N or n)
        if c == 'N' || c == 'n' {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c == '+' || next_c == '-' || next_c.is_ascii_digit() {
                    value.push(chars.next().unwrap());
                    length += 1;
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            value.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            tokens.push(create_driver_token(
                "note_shift_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Tie (_)
        if c == '_' {
            tokens.push(create_driver_token(
                "tie".to_string(),
                "_".to_string(),
                start_line,
                start_column,
                1,
            ));
            column += 1;
            continue;
        }

        // Octave up/down
        if c == '>' || c == '<' {
            let token_type = if c == '>' { "octave_up" } else { "octave_down" };
            tokens.push(create_driver_token(
                token_type.to_string(),
                c.to_string(),
                start_line,
                start_column,
                1,
            ));
            column += 1;
            continue;
        }

        // Dot
        if c == '.' {
            tokens.push(create_driver_token(
                "dot".to_string(),
                ".".to_string(),
                start_line,
                start_column,
                1,
            ));
            column += 1;
            continue;
        }

        // Bar
        if c == '|' {
            tokens.push(create_driver_token(
                "bar".to_string(),
                "|".to_string(),
                start_line,
                start_column,
                1,
            ));
            column += 1;
            continue;
        }

        // Comments (; or *)
        if c == ';' || c == '*' {
            let mut comment = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == '\n' {
                    break;
                }
                comment.push(chars.next().unwrap());
            }
            tokens.push(create_driver_token(
                "comment".to_string(),
                comment.clone(),
                start_line,
                start_column,
                comment.len() + 1,
            ));
            column += comment.len() + 1;
            continue;
        }

        // Numbers (standalone)
        if c.is_ascii_digit() {
            let mut num_str = c.to_string();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    num_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(create_driver_token(
                "number".to_string(),
                num_str.clone(),
                start_line,
                start_column,
                num_str.len(),
            ));
            column += num_str.len();
            continue;
        }

        // Rhythm instruments (BD, SD, TOM, HH, CYM, RIM)
        // These are typically in PMD rhythm sections
        if c == 'B' || c == 'b' {
            let mut value = c.to_string();
            if let Some(&next_c) = chars.peek() {
                if next_c == 'D' || next_c == 'd' {
                    value.push(chars.next().unwrap());
                    tokens.push(create_driver_token(
                        "rhythm_instrument".to_string(),
                        value,
                        start_line,
                        start_column,
                        2,
                    ));
                    column += 2;
                    continue;
                }
            }
        }

        // Default: unknown token
        tokens.push(create_driver_token(
            "unknown".to_string(),
            c.to_string(),
            start_line,
            start_column,
            c.len_utf8(),
        ));
        column += 1;
    }

    Ok(tokens)
}

/// Helper to create a DriverToken
fn create_driver_token(
    token_type: String,
    value: String,
    line: usize,
    column: usize,
    length: usize,
) -> DriverToken {
    DriverToken {
        token_type,
        value,
        line,
        column,
        length,
    }
}

// ============================================================================
// PMD Parser
// ============================================================================

/// Parse error for PMD
#[derive(Debug, Clone)]
pub struct PMDParseError {
    /// Message.
    pub message: String,
    /// Line.
    pub line: usize,
    /// Column.
    pub column: usize,
    /// Length.
    pub length: usize,
}

/// AST node for PMD
#[derive(Debug, Clone)]
pub enum PMDAstNode {
    /// Note.
    Note {
        /// Note.
        note: char,
        /// Sharp.
        sharp: bool,
        /// Octave.
        octave: Option<u8>,
        /// Duration.
        duration: Option<u8>,
        /// Tie.
        tie: bool,
        /// Dotted.
        dotted: bool,
        /// Line.
        line: usize,
        /// Column.
        column: usize,
    },
    /// Rest.
    Rest {
        /// Duration.
        duration: Option<u8>,
        /// Line.
        line: usize,
        /// Column.
        column: usize,
    },
    /// Part.
    Part {
        /// Part num.
        part_num: u8,
        /// Commands.
        commands: Vec<PMDAstNode>,
        /// Line.
        line: usize,
    },
    /// Part Select.
    PartSelect {
        /// Part num.
        part_num: u8,
        /// Line.
        line: usize,
    },
    /// Part End.
    PartEnd {
        /// Line.
        line: usize,
    },
    /// Octave.
    Octave {
        /// Octave.
        octave: u8,
        /// Line.
        line: usize,
    },
    /// Volume.
    Volume {
        /// Volume.
        volume: u8,
        /// Line.
        line: usize,
    },
    /// Volume Change.
    VolumeChange {
        /// Delta.
        delta: i8,
        /// Line.
        line: usize,
    },
    /// Length.
    Length {
        /// Length.
        length: u8,
        /// Line.
        line: usize,
    },
    /// Tempo.
    Tempo {
        /// Tempo.
        tempo: u8,
        /// Line.
        line: usize,
    },
    /// Note Shift.
    NoteShift {
        /// Shift.
        shift: i8,
        /// Line.
        line: usize,
    },
    /// Loop.
    Loop {
        /// Count.
        count: Option<u8>,
        /// Body.
        body: Vec<PMDAstNode>,
        /// Line.
        line: usize,
    },
    /// Loop Infinite.
    LoopInfinite {
        /// Body.
        body: Vec<PMDAstNode>,
        /// Line.
        line: usize,
    },
    /// Loop Break.
    LoopBreak {
        /// Line.
        line: usize,
    },
    /// Directive.
    Directive {
        /// Name.
        name: String,
        /// Value.
        value: Option<String>,
        /// Line.
        line: usize,
    },
    /// Rhythm Instrument.
    RhythmInstrument {
        /// Instrument.
        instrument: String,
        /// Line.
        line: usize,
    },
    /// Comment.
    Comment {
        /// Text.
        text: String,
        /// Line.
        line: usize,
    },
    /// Bar.
    Bar {
        /// Line.
        line: usize,
    },
}

/// Parse PMD content into AST
// `current_part`/`current_length`/`current_volume` track parser state that is not
// yet consumed by AST construction; retained for upcoming stateful handling.
#[allow(unused_variables, unused_assignments)]
fn pmd_parse(content: &str) -> Result<Vec<PMDAstNode>, Vec<PMDParseError>> {
    let mut errors = Vec::new();
    let mut ast = Vec::new();
    let mut current_part: Option<u8> = None;
    let mut current_octave: u8 = 4;
    let mut current_length: u8 = 4;
    let mut current_volume: u8 = 15;

    let tokens = match pmd_tokenize(content) {
        Ok(t) => t,
        Err(e) => {
            errors.push(PMDParseError {
                message: e.to_string(),
                line: 0,
                column: 0,
                length: 0,
            });
            return Err(errors);
        }
    };

    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match token.token_type.as_str() {
            "part_cmd" => {
                // Check for @@ (part end)
                if token.value == "@@" {
                    ast.push(PMDAstNode::PartEnd { line: token.line });
                    continue;
                }

                // Parse part number
                let part_str = token.value.trim_start_matches('@');
                let part_num = part_str.parse::<u8>().unwrap_or(0);
                current_part = Some(part_num);

                ast.push(PMDAstNode::PartSelect {
                    part_num,
                    line: token.line,
                });
            }

            "note" | "note_sharp" => {
                let note_char = token.value.chars().next().unwrap();
                let sharp = token.token_type == "note_sharp";
                let mut duration: Option<u8> = None;
                let mut dotted = false;
                let mut tie = false;

                // Check for duration number
                if let Some(next) = iter.peek() {
                    if next.token_type == "number" {
                        if let Ok(d) = next.value.parse::<u8>() {
                            duration = Some(d);
                            iter.next();
                        }
                    }
                }

                // Check for dot
                if let Some(next) = iter.peek() {
                    if next.token_type == "dot" {
                        dotted = true;
                        iter.next();
                    }
                }

                // Check for tie
                if let Some(next) = iter.peek() {
                    if next.token_type == "tie" {
                        tie = true;
                        iter.next();
                    }
                }

                ast.push(PMDAstNode::Note {
                    note: note_char,
                    sharp,
                    octave: Some(current_octave),
                    duration,
                    tie,
                    dotted,
                    line: token.line,
                    column: token.column,
                });
            }

            "rest" => {
                let mut duration: Option<u8> = None;

                if let Some(next) = iter.peek() {
                    if next.token_type == "number" {
                        if let Ok(d) = next.value.parse::<u8>() {
                            duration = Some(d);
                            iter.next();
                        }
                    }
                }

                ast.push(PMDAstNode::Rest {
                    duration,
                    line: token.line,
                    column: token.column,
                });
            }

            "octave_cmd" => {
                if let Ok(octave) = token.value.parse::<u8>() {
                    current_octave = octave;
                    ast.push(PMDAstNode::Octave {
                        octave,
                        line: token.line,
                    });
                }
            }

            "volume_cmd" => {
                if token.value.len() > 1 {
                    let val_str = token.value.get(1..).unwrap_or("");
                    if val_str.starts_with('+') || val_str.starts_with('-') {
                        // Relative volume change
                    } else if let Ok(volume) = val_str.parse::<u8>() {
                        current_volume = volume;
                        ast.push(PMDAstNode::Volume {
                            volume,
                            line: token.line,
                        });
                    }
                }
            }

            "length_cmd" => {
                if let Some(len_str) = token.value.get(1..) {
                    if let Ok(length) = len_str.parse::<u8>() {
                        current_length = length;
                        ast.push(PMDAstNode::Length {
                            length,
                            line: token.line,
                        });
                    }
                }
            }

            "tempo_cmd" => {
                if let Some(tempo_str) = token.value.get(1..) {
                    if let Ok(tempo) = tempo_str.parse::<u8>() {
                        ast.push(PMDAstNode::Tempo {
                            tempo,
                            line: token.line,
                        });
                    }
                }
            }

            "note_shift_cmd" => {
                if let Some(shift_str) = token.value.get(1..) {
                    if let Ok(shift) = shift_str.parse::<i8>() {
                        ast.push(PMDAstNode::NoteShift {
                            shift,
                            line: token.line,
                        });
                    }
                }
            }

            "loop_start" => {
                // Handle (n or (
                let count = if token.value.len() > 1
                    && token.value.chars().nth(1).unwrap().is_ascii_digit()
                {
                    let c: u8 = token.value.get(1..).unwrap().parse().unwrap_or(1);
                    Some(c)
                } else {
                    // Check if next token is a number
                    if let Some(next) = iter.peek() {
                        if next.token_type == "number" {
                            let c = next.value.parse::<u8>().unwrap_or(1);
                            iter.next();
                            Some(c)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                // Collect loop body
                let mut body = Vec::new();
                while let Some(next) = iter.peek() {
                    if next.token_type == "loop_end" {
                        iter.next();
                        break;
                    }
                    if let Some(t) = iter.next() {
                        body.push(PMDAstNode::Comment {
                            text: format!("loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(PMDAstNode::Loop {
                    count,
                    body,
                    line: token.line,
                });
            }

            "loop_start_infinite" => {
                let mut body = Vec::new();
                while let Some(next) = iter.peek() {
                    if next.token_type == "loop_end_infinite" {
                        iter.next();
                        break;
                    }
                    if let Some(t) = iter.next() {
                        body.push(PMDAstNode::Comment {
                            text: format!("infinite loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(PMDAstNode::LoopInfinite {
                    body,
                    line: token.line,
                });
            }

            "loop_end" | "loop_end_infinite" => {
                ast.push(PMDAstNode::LoopBreak { line: token.line });
            }

            "directive" => {
                let name = token
                    .value
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                let value = token.value.get(name.len()..).map(|s| s.trim().to_string());

                // Clean up the name by removing @
                let clean_name = name.trim_start_matches('@').to_string();

                ast.push(PMDAstNode::Directive {
                    name: clean_name,
                    value,
                    line: token.line,
                });
            }

            "rhythm_instrument" => {
                ast.push(PMDAstNode::RhythmInstrument {
                    instrument: token.value,
                    line: token.line,
                });
            }

            "comment" | "bar" | "part_end" => {
                // Skip for now
            }

            _ => {
                // Unknown or unhandled token
            }
        }
    }

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_id() {
        let driver = PMDDriver;
        assert_eq!(driver.id(), "pmd");
        assert_eq!(driver.display_name(), "PMD (PC-9801)");
    }

    #[test]
    fn test_driver_extensions() {
        let driver = PMDDriver;
        let exts = driver.supported_extensions();
        assert!(exts.contains(&".mus"));
        assert!(exts.contains(&".mdl"));
    }

    #[test]
    fn test_detect_extension_mus() {
        let driver = PMDDriver;
        assert_eq!(driver.detect("test", Some("song.mus")), 90);
    }

    #[test]
    fn test_detect_extension_mdl() {
        let driver = PMDDriver;
        assert_eq!(driver.detect("test", Some("song.mdl")), 80);
    }

    #[test]
    fn test_detect_music_directive() {
        let driver = PMDDriver;
        let content = "@MUSIC\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 95);
    }

    #[test]
    fn test_detect_ppz_directive() {
        let driver = PMDDriver;
        let content = "@PPZ\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 95);
    }

    #[test]
    fn test_detect_pmd_mention() {
        let driver = PMDDriver;
        let content = "; PMD format\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_ym2608() {
        let driver = PMDDriver;
        let content = "; YM2608 target\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 70);
    }

    #[test]
    fn test_detect_pc98() {
        let driver = PMDDriver;
        let content = "; PC-9801\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 65);
    }

    #[test]
    fn test_tokenize_basic() {
        let content = "@0 o4 c4 d4 e4 f4";
        let result = PMDDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());

        // Check for part command
        assert!(tokens.iter().any(|t| t.token_type == "part_cmd"));
        // Check for octave command
        assert!(tokens.iter().any(|t| t.token_type == "octave_cmd"));
        // Check for notes
        assert!(tokens.iter().any(|t| t.token_type == "note"));
    }

    #[test]
    fn test_tokenize_with_sharp() {
        let content = "c+ d# e f+";
        let result = PMDDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "note_sharp"));
    }

    #[test]
    fn test_tokenize_directive() {
        let content = "@MUSIC\n@TEMPO 120\n@0 o4 c4";
        let result = PMDDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "directive"));
    }

    #[test]
    fn test_tokenize_loops() {
        let content = "@0 (4 c4 d4 e4 f4)";
        let result = PMDDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "loop_start"));
        assert!(tokens.iter().any(|t| t.token_type == "loop_end"));
    }

    #[test]
    fn test_tokenize_part_end() {
        let content = "@0 o4 c4 @@";
        let result = PMDDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "part_end"));
    }

    #[test]
    fn test_parse_basic() {
        let content = "@0 o4 c4 d4 e4";
        let result = pmd_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(!ast.is_empty());
    }

    #[test]
    fn test_parse_with_part() {
        let content = "@0 o4 c4\n@1 o5 e4";
        let result = pmd_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(ast
            .iter()
            .any(|n| matches!(n, PMDAstNode::PartSelect { .. })));
    }

    #[test]
    fn test_detect_target_chips_ym2608() {
        let content = "@MUSIC\n@0 o4 c4";
        let chips = detect_target_chips(content);
        assert!(chips.contains(&SoundChip::YM2608));
        assert!(chips.contains(&SoundChip::SN76489));
    }

    #[test]
    fn test_detect_target_chips_ym2203() {
        let content = "ym2203\n@0 o4 c4";
        let chips = detect_target_chips(content);
        assert!(chips.contains(&SoundChip::YM2203));
        assert!(chips.contains(&SoundChip::SN76489));
    }

    #[test]
    fn test_detect_target_chips_default() {
        let content = "@0 o4 c4";
        let chips = detect_target_chips(content);
        // Default should be YM2608 + SN76489
        assert!(chips.contains(&SoundChip::YM2608));
        assert!(chips.contains(&SoundChip::SN76489));
    }
}
