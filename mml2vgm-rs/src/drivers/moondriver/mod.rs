//! MoonDriver External Driver
//!
//! This module implements the MoonDriver MML format driver for multi-platform
//! OPN2/OPNA/OPN3 chip support.
//!
//! MoonDriver is a flexible MML format that can target different OPN variants:
//! - OPN2 (YM2612) - Sega Mega Drive
//! - OPNA (YM2608) - NEC PC-9801
//! - OPN3 (YM3438) - MSX-AUDIO, etc.
//!
//! ## Format Specification
//!
//! - **File Extension**: `.mdl`
//! - **Target Platform**: Multi-platform (OPN2/OPNA/OPN3)
//! - **Directives**: `#MD`, `#OPN2`, `#OPNA`, `#OPN3`, `#TEMPO`, `#INCLUDE`
//! - **Chip Support**: YM2612, YM2608, YM3438
//!
//! ## MoonDriver Command Reference
//!
//! Based on the MoonDriverDotNET driver from the .NET IDE.
//!
//! ### Basic Structure
//! - Parts are defined with `@n` where n is the channel number
//! - Each part can target a specific chip
//!
//! ### Note Format
//! - Notes: `C`, `C#`, `D`, `D#`, `E`, `F`, `F#`, `G`, `G#`, `A`, `A#`, `B`
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
//! - `V15` (0-15)
//! - `V+` / `V-` = volume up/down
//!
//! ### Commands
//! - `L4` = length (default 4)
//! - `T120` = tempo (default 120)
//! - `Q4` = gate time
//! - `@n` = part/channel select (n = 0-255)
//! - `Yn` = volume mode
//! - `(n` = loop start, `)n` = loop end (finite loop)
//! - `[` = infinite loop start, `]` = infinite loop end
//! - `Sn` = instrument/voice
//! - `Pn` = pan (OPN2 only)
//! - `Mn` = modulation
//! - `Nn` = note shift
//! - `Wn` = pitch bend
//! - `&n` = detune
//!
//! ### Directives
//! - `#MD` = MoonDriver directive
//! - `#OPN2` = Set target to YM2612
//! - `#OPNA` = Set target to YM2608
//! - `#OPN3` = Set target to YM3438
//! - `#TEMPO n` = Set default tempo
//! - `#VOLUME n` = Set default volume
//! - `#INCLUDE "file.mdl"` = Include another file
//!
//! ### Special
//! - `;` or `*` = comment to end of line
//! - `|` = bar line
//! - `"` = string literal (for #INCLUDE)

use crate::drivers::{
    DiagnosticSeverity, DriverCompileOptions, DriverCompileResult, DriverDiagnostic,
    DriverOutputFormat, DriverToken, ExternalDriver,
};
use crate::{error::MmlError, CompileOptions, OutputFormat, SoundChip};

/// MoonDriver implementation
pub struct MoonDriver;

impl ExternalDriver for MoonDriver {
    fn id(&self) -> &str {
        "moondriver"
    }

    fn display_name(&self) -> &str {
        "MoonDriver (MDL)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".mdl"]
    }

    fn description(&self) -> &str {
        "MoonDriver MML format for multi-platform (OPN2/OPNA/OPN3)"
    }

    fn target_platform(&self) -> &str {
        "Multi-platform (OPN2/OPNA/OPN3)"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, content: &str, filename: Option<&str>) -> u8 {
        // Check filename first
        if let Some(name) = filename {
            if name.to_lowercase().ends_with(".mdl") {
                return 85;
            }
        }

        let content_lower = content.to_lowercase();
        let content_trimmed = content.trim();

        // High confidence: MoonDriver directive
        if content_trimmed.starts_with("#md") || content_lower.contains("#md") {
            return 95;
        }

        // High confidence: OPN variant directives
        if content_lower.contains("#opn2")
            || content_lower.contains("#opna")
            || content_lower.contains("#opn3")
        {
            return 90;
        }

        // High confidence: MoonDriver mention
        if content_lower.contains("moondriver") {
            return 90;
        }

        // Medium confidence: OPN chip mentions
        if content_lower.contains("opn2")
            || content_lower.contains("opna")
            || content_lower.contains("opn3")
        {
            return 70;
        }

        // Medium confidence: YM chip mentions specific to MoonDriver targets
        if content_lower.contains("ym2612")
            || content_lower.contains("ym2608")
            || content_lower.contains("ym3438")
        {
            return 60;
        }

        // Low confidence: part commands with MDL-style usage
        if content.contains('@') {
            let at_count = content.matches('@').count();
            let digit_count = content.chars().filter(|c| c.is_ascii_digit()).count();
            if at_count > 0 && digit_count > at_count {
                return 40;
            }
        }

        // Low confidence: common MDL patterns
        if content_lower.contains("#tempo")
            || content_lower.contains("#volume")
            || content_lower.contains("#include")
        {
            return 30;
        }

        0
    }

    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError> {
        // Parse and validate the MoonDriver content
        match moondriver_parse(content) {
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
        match moondriver_tokenize(content) {
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
        // Parse the MoonDriver content (ignore errors for now)
        let _ast = moondriver_parse(content);

        // Convert output format
        let output_format = match options.output_format {
            DriverOutputFormat::VGM => OutputFormat::VGM,
            DriverOutputFormat::XGM => OutputFormat::XGM,
            DriverOutputFormat::XGM2 => OutputFormat::XGM2,
            DriverOutputFormat::ZGM => OutputFormat::ZGM,
        };

        // Detect target chip from content or use YM2608 as default
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

/// Detect target chips from MoonDriver content
fn detect_target_chips(content: &str) -> Vec<SoundChip> {
    let content_lower = content.to_lowercase();

    // Check for OPN3 (YM2609 is the closest match for OPN3 in this codebase)
    if content_lower.contains("#opn3") || content_lower.contains("ym3438") {
        return vec![SoundChip::YM2609];
    }

    // Check for OPNA (YM2608)
    if content_lower.contains("#opna") || content_lower.contains("ym2608") {
        return vec![SoundChip::YM2608, SoundChip::SN76489];
    }

    // Check for OPN2 (YM2612)
    if content_lower.contains("#opn2") || content_lower.contains("ym2612") {
        return vec![SoundChip::YM2612, SoundChip::SN76489];
    }

    // Default to OPNA (YM2608) as it's the most common for MoonDriver
    vec![SoundChip::YM2608, SoundChip::SN76489]
}

// ============================================================================
// MoonDriver Tokenizer
// ============================================================================

/// Token for MoonDriver syntax highlighting
#[derive(Debug, Clone)]
pub struct MoonDriverToken {
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

/// Tokenize MoonDriver content for syntax highlighting
fn moondriver_tokenize(content: &str) -> Result<Vec<DriverToken>, MmlError> {
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

        // Notes: A-G, a-g, with optional #
        if ('A'..='G').contains(&c) || ('a'..='g').contains(&c) {
            let note_upper = c.to_ascii_uppercase();
            let mut note_str = note_upper.to_string();
            let mut token_type = "note".to_string();
            let mut length = 1;

            // Check for sharp
            if let Some(&next_c) = chars.peek() {
                if next_c == '#' {
                    note_str.push('#');
                    token_type = "note_sharp".to_string();
                    chars.next();
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

        // Part command (@)
        if c == '@' {
            let mut value = "@".to_string();
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
                "part_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Directives (#)
        if c == '#' {
            let mut directive = "#".to_string();
            let mut length = 1;

            while let Some(&next_c) = chars.peek() {
                if next_c.is_alphabetic() {
                    directive.push(chars.next().unwrap());
                    length += 1;
                } else {
                    break;
                }
            }

            tokens.push(create_driver_token(
                "directive".to_string(),
                directive,
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

        // Instrument/Voice (S or s)
        if c == 'S' || c == 's' {
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
                "instrument_cmd".to_string(),
                value,
                start_line,
                start_column,
                length,
            ));
            column += length;
            continue;
        }

        // Other commands: Y, P, M, W, N, &
        if "YPWMN&".contains(c) {
            let mut value = c.to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c == '+' || next_c == '-' || next_c.is_ascii_digit() {
                    value.push(chars.next().unwrap());
                    length += 1;

                    // Continue reading digits for multi-digit values
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

            let token_type = match c {
                'Y' => "volume_mode_cmd",
                'P' => "pan_cmd",
                'M' => "modulation_cmd",
                'W' => "pitch_bend_cmd",
                'N' => "note_shift_cmd",
                '&' => "detune_cmd",
                _ => "command",
            };

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

        // String literals (for #INCLUDE)
        if c == '"' {
            let mut string_val = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == '"' {
                    chars.next(); // consume closing quote
                    break;
                }
                string_val.push(chars.next().unwrap());
            }
            let string_len = string_val.len();
            tokens.push(create_driver_token(
                "string".to_string(),
                string_val,
                start_line,
                start_column,
                string_len,
            ));
            column += string_len + 2; // +2 for quotes
            continue;
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
// MoonDriver Parser
// ============================================================================

/// Parse error for MoonDriver
#[derive(Debug, Clone)]
pub struct MoonDriverParseError {
    /// Message.
    pub message: String,
    /// Line.
    pub line: usize,
    /// Column.
    pub column: usize,
    /// Length.
    pub length: usize,
}

/// AST node for MoonDriver
#[derive(Debug, Clone)]
pub enum MoonDriverAstNode {
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
        commands: Vec<MoonDriverAstNode>,
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
    /// Instrument.
    Instrument {
        /// Instrument.
        instrument: u8,
        /// Line.
        line: usize,
    },
    /// Loop.
    Loop {
        /// Count.
        count: Option<u8>,
        /// Body.
        body: Vec<MoonDriverAstNode>,
        /// Line.
        line: usize,
    },
    /// Loop Infinite.
    LoopInfinite {
        /// Body.
        body: Vec<MoonDriverAstNode>,
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

/// Parse MoonDriver content into AST
// `current_part`/`current_length`/`current_volume` track parser state that is not
// yet consumed by AST construction; retained for upcoming stateful handling.
#[allow(unused_variables, unused_assignments)]
fn moondriver_parse(content: &str) -> Result<Vec<MoonDriverAstNode>, Vec<MoonDriverParseError>> {
    let mut errors = Vec::new();
    let mut ast = Vec::new();
    let mut current_part: Option<u8> = None;
    let mut current_octave: u8 = 4;
    let mut current_length: u8 = 4;
    let mut current_volume: u8 = 15;

    let tokens = match moondriver_tokenize(content) {
        Ok(t) => t,
        Err(e) => {
            errors.push(MoonDriverParseError {
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
                // Parse part number
                let part_str = token.value.trim_start_matches('@');
                let part_num = part_str.parse::<u8>().unwrap_or(0);
                current_part = Some(part_num);

                ast.push(MoonDriverAstNode::PartSelect {
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

                ast.push(MoonDriverAstNode::Note {
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

                ast.push(MoonDriverAstNode::Rest {
                    duration,
                    line: token.line,
                    column: token.column,
                });
            }

            "octave_cmd" => {
                if let Ok(octave) = token.value.parse::<u8>() {
                    current_octave = octave;
                    ast.push(MoonDriverAstNode::Octave {
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
                        ast.push(MoonDriverAstNode::Volume {
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
                        ast.push(MoonDriverAstNode::Length {
                            length,
                            line: token.line,
                        });
                    }
                }
            }

            "tempo_cmd" => {
                if let Some(tempo_str) = token.value.get(1..) {
                    if let Ok(tempo) = tempo_str.parse::<u8>() {
                        ast.push(MoonDriverAstNode::Tempo {
                            tempo,
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
                        body.push(MoonDriverAstNode::Comment {
                            text: format!("loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(MoonDriverAstNode::Loop {
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
                        body.push(MoonDriverAstNode::Comment {
                            text: format!("infinite loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(MoonDriverAstNode::LoopInfinite {
                    body,
                    line: token.line,
                });
            }

            "loop_end" | "loop_end_infinite" => {
                ast.push(MoonDriverAstNode::LoopBreak { line: token.line });
            }

            "directive" => {
                let name = token
                    .value
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                let value = token.value.get(name.len()..).map(|s| s.trim().to_string());

                // Clean up the name by removing #
                let clean_name = name.trim_start_matches('#').to_string();

                ast.push(MoonDriverAstNode::Directive {
                    name: clean_name,
                    value,
                    line: token.line,
                });
            }

            "instrument_cmd" => {
                if let Some(instr_str) = token.value.get(1..) {
                    if let Ok(instrument) = instr_str.parse::<u8>() {
                        ast.push(MoonDriverAstNode::Instrument {
                            instrument,
                            line: token.line,
                        });
                    }
                }
            }

            "comment" | "bar" => {
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
        let driver = MoonDriver;
        assert_eq!(driver.id(), "moondriver");
        assert_eq!(driver.display_name(), "MoonDriver (MDL)");
    }

    #[test]
    fn test_driver_extensions() {
        let driver = MoonDriver;
        let exts = driver.supported_extensions();
        assert!(exts.contains(&".mdl"));
    }

    #[test]
    fn test_detect_extension() {
        let driver = MoonDriver;
        assert_eq!(driver.detect("test", Some("song.mdl")), 85);
    }

    #[test]
    fn test_detect_md_directive() {
        let driver = MoonDriver;
        let content = "#MD\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 95);
    }

    #[test]
    fn test_detect_opn2_directive() {
        let driver = MoonDriver;
        let content = "#OPN2\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_opna_directive() {
        let driver = MoonDriver;
        let content = "#OPNA\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_moondriver_mention() {
        let driver = MoonDriver;
        let content = "; MoonDriver format\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_ym2612() {
        let driver = MoonDriver;
        let content = "; YM2612 target\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 60);
    }

    #[test]
    fn test_tokenize_basic() {
        let content = "@0 o4 c4 d4 e4 f4";
        let result = MoonDriver.tokenize(content);
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
        let content = "c# d e f#";
        let result = MoonDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "note_sharp"));
    }

    #[test]
    fn test_tokenize_directive() {
        let content = "#MD\n#OPN2\n#TEMPO 120";
        let result = MoonDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "directive"));
    }

    #[test]
    fn test_tokenize_include_directive() {
        let content = "#INCLUDE \"test.mdl\"";
        let result = MoonDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "directive"));
        assert!(tokens.iter().any(|t| t.token_type == "string"));
    }

    #[test]
    fn test_tokenize_loops() {
        let content = "@0 (4 c4 d4 e4 f4)";
        let result = MoonDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "loop_start"));
        assert!(tokens.iter().any(|t| t.token_type == "loop_end"));
    }

    #[test]
    fn test_tokenize_infinite_loops() {
        let content = "@0 [ c4 d4 e4 f4 ]";
        let result = MoonDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "loop_start_infinite"));
        assert!(tokens.iter().any(|t| t.token_type == "loop_end_infinite"));
    }

    #[test]
    fn test_parse_basic() {
        let content = "@0 o4 c4 d4 e4";
        let result = moondriver_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(!ast.is_empty());
    }

    #[test]
    fn test_parse_with_part() {
        let content = "@0 o4 c4\n@1 o5 e4";
        let result = moondriver_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(ast
            .iter()
            .any(|n| matches!(n, MoonDriverAstNode::PartSelect { .. })));
    }

    #[test]
    fn test_detect_target_chips_opn2() {
        let content = "#OPN2\n@0 o4 c4";
        let chips = detect_target_chips(content);
        assert!(chips.contains(&SoundChip::YM2612));
        assert!(chips.contains(&SoundChip::SN76489));
    }

    #[test]
    fn test_detect_target_chips_opna() {
        let content = "#OPNA\n@0 o4 c4";
        let chips = detect_target_chips(content);
        assert!(chips.contains(&SoundChip::YM2608));
        assert!(chips.contains(&SoundChip::SN76489));
    }

    #[test]
    fn test_detect_target_chips_opn3() {
        let content = "#OPN3\n@0 o4 c4";
        let chips = detect_target_chips(content);
        // OPN3 uses YM2609 as the closest match
        assert!(chips.contains(&SoundChip::YM2609));
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
