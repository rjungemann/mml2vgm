//! Mucom88 External Driver
//!
//! This module implements the mucom88 MML format driver for Sega Mega Drive.
//! mucom88 is a popular MML compiler targeting YM2612 (FM) + SN76489 (PSG).
//!
//! ## Format Specification
//!
//! - **Target Platform**: Sega Mega Drive / Genesis
//! - **Supported Chips**: YM2612 (6 FM channels), SN76489 (4 PSG channels)
//! - **File Extension**: `.muc`
//! - **Complexity**: High
//!
//! ## Mucom88 Command Reference
//!
//! Based on the mucom88 format used in the .NET IDE's MUCOMDotNET driver.
//!
//! ### Basic Structure
//! - Parts are defined with `@n` where n is 0-255 (typically 0-12)
//! - `@0` = FM channel 1, `@1` = FM channel 2, ... `@5` = FM channel 6
//! - `@6` = PSG channel 1, `@7` = PSG channel 2, `@8` = PSG channel 3, `@9` = PSG channel 4 (noise)
//!
//! ### Note Format
//! - Notes: `C`, `C#`, `D`, `D#`, `E`, `F`, `F#`, `G`, `G#`, `A`, `A#`, `B`
//! - Rest: `R`
//! - Duration: Number after note (e.g., `C4` = quarter note)
//! - Dotted notes: `C4.` = dotted quarter
//! - Tied notes: `C4_` or `C4__`
//!
//! ### Octave
//! - `O3`, `O4`, `O5`, etc. (default O4)
//! - `>` = octave up, `<` = octave down
//!
//! ### Volume
//! - `V15` (0-15 for FM, 0-15 for PSG)
//! - `V+` / `V-` = volume up/down
//!
//! ### Commands
//! - `L4` = length (1-256, default 4)
//! - `T120` = tempo (40-255, default 120)
//! - `Q4` = gate time (0-7)
//! - `Yn` = volume mode (0 = FM volume, 1 = PSG volume)
//! - `@n` = part/channel select
//! - `(n` = loop start, `)n` = loop end (n = count)
//! - `[` = loop start, `]` = loop end (infinite until `\[`)
//! - `Nn` = note shift
//! - `Sn` = instrument/voice number
//! - `&n` = detune
//! - `Mn` = modulation
//! - `Pn` = pan (YM2612 only)
//! - `Xn` = expression
//!
//! ### Special
//! - `;` or `*` = comment to end of line
//! - `|` = bar line
//! - ` ` (space) = separator
//!
//! ### Instruments (Voices)
//! - `#W` = wave form select
//! - `#ADPCM` = ADPCM sample (Mega Drive only)
//! - Voice files: `. voice.dat` or `#VOICE filename`

use crate::drivers::{
    DiagnosticSeverity, DriverCompileOptions, DriverCompileResult, DriverDiagnostic,
    DriverOutputFormat, DriverToken, ExternalDriver,
};
use crate::{error::MmlError, CompileOptions, OutputFormat, SoundChip};

/// Mucom88 Driver implementation
pub struct MucomDriver;

impl ExternalDriver for MucomDriver {
    fn id(&self) -> &str {
        "mucom"
    }

    fn display_name(&self) -> &str {
        "mucom88 (MUC)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".muc"]
    }

    fn description(&self) -> &str {
        "mucom88 MML format for Sega Mega Drive (YM2612 + SN76489)"
    }

    fn target_platform(&self) -> &str {
        "Sega Mega Drive / Genesis"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, content: &str, filename: Option<&str>) -> u8 {
        // Check filename first
        if let Some(name) = filename {
            if name.to_lowercase().ends_with(".muc") {
                return 85;
            }
        }

        let content_lower = content.to_lowercase();
        let content_trimmed = content.trim();

        // High confidence: mucom88 directive or version
        if content_lower.contains("mucom88")
            || content_lower.contains("mucom") && content_lower.contains("88")
        {
            return 95;
        }

        // High confidence: #MUCOM directive
        if content_trimmed.starts_with("#mucom") || content_lower.contains("#mucom") {
            return 95;
        }

        // High confidence: YM2612 specific
        if content_lower.contains("ym2612") || content_lower.contains("opn2") {
            return 90;
        }

        // Medium confidence: Sega/Genesis mention
        if content_lower.contains("sega")
            || content_lower.contains("genesis")
            || content_lower.contains("mega drive")
        {
            return 75;
        }

        // Medium confidence: PSG mention
        if content_lower.contains("psg") || content_lower.contains("sn76489") {
            return 70;
        }

        // Low confidence: FM channel patterns (@0-@9)
        if content.contains('@') {
            let at_count = content.matches('@').count();
            let digit_count = content.chars().filter(|c| c.is_ascii_digit()).count();
            if at_count > 0 && digit_count > at_count {
                return 50;
            }
        }

        // Low confidence: instrument/voice commands
        if content_lower.contains("#voice")
            || content_lower.contains("#w")
            || content_lower.contains("s")
        {
            return 30;
        }

        0
    }

    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError> {
        // Parse and validate the mucom content
        match mucom_parse(content) {
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
        match mucom_tokenize(content) {
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
        // Parse the mucom content (ignore errors for now)
        let _ast = mucom_parse(content);

        // Convert output format
        let output_format = match options.output_format {
            DriverOutputFormat::VGM => OutputFormat::VGM,
            DriverOutputFormat::XGM => OutputFormat::XGM,
            DriverOutputFormat::XGM2 => OutputFormat::XGM2,
            DriverOutputFormat::ZGM => OutputFormat::ZGM,
        };

        // Create compile options with YM2612 + SN76489 for Mega Drive
        let compile_options = CompileOptions {
            format: output_format,
            target_chips: Some(vec![SoundChip::YM2612, SoundChip::SN76489]),
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

// ============================================================================
// Mucom88 Tokenizer
// ============================================================================

/// Token for mucom88 syntax highlighting
#[derive(Debug, Clone)]
pub struct MucomToken {
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

/// Tokenize mucom88 content for syntax highlighting
fn mucom_tokenize(content: &str) -> Result<Vec<DriverToken>, MmlError> {
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
            tokens.push(create_driver_token(
                "rest".to_string(),
                "r".to_string(),
                start_line,
                start_column,
                1,
            ));
            column += 1;
            continue;
        }

        // Part command (@)
        if c == '@' {
            // Read part number if present
            let mut value = "@".to_string();
            let mut length = 1;

            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    let mut num_str = String::new();
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            num_str.push(chars.next().unwrap());
                            length += 1;
                        } else {
                            break;
                        }
                    }
                    value.push_str(&num_str);
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

            // Check for + or -
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
// Mucom88 Parser
// ============================================================================

/// Parse error for mucom88
#[derive(Debug, Clone)]
pub struct MucomParseError {
    /// Message.
    pub message: String,
    /// Line.
    pub line: usize,
    /// Column.
    pub column: usize,
    /// Length.
    pub length: usize,
}

/// AST node for mucom88
#[derive(Debug, Clone)]
pub enum MucomAstNode {
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
        commands: Vec<MucomAstNode>,
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
        body: Vec<MucomAstNode>,
        /// Line.
        line: usize,
    },
    /// Loop Infinite.
    LoopInfinite {
        /// Body.
        body: Vec<MucomAstNode>,
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

/// Parse mucom88 content into AST
// `current_part`/`current_length`/`current_volume` track parser state that is not
// yet consumed by AST construction; retained for upcoming stateful handling.
#[allow(unused_variables, unused_assignments)]
fn mucom_parse(content: &str) -> Result<Vec<MucomAstNode>, Vec<MucomParseError>> {
    // For now, do basic parsing - full implementation will come later
    let mut errors = Vec::new();
    let mut ast = Vec::new();
    let mut current_part: Option<u8> = None;
    let mut current_octave: u8 = 4; // Default octave
    let mut current_length: u8 = 4; // Default length
    let mut current_volume: u8 = 15; // Default volume

    let tokens = match mucom_tokenize(content) {
        Ok(t) => t,
        Err(e) => {
            errors.push(MucomParseError {
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
                // Parse part number from value (e.g., "@0", "@12")
                let part_str = token.value.trim_start_matches('@');
                let part_num = part_str.parse::<u8>().unwrap_or(0);
                current_part = Some(part_num);

                ast.push(MucomAstNode::PartSelect {
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

                ast.push(MucomAstNode::Note {
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

                ast.push(MucomAstNode::Rest {
                    duration,
                    line: token.line,
                    column: token.column,
                });
            }

            "octave_cmd" => {
                if let Ok(octave) = token.value.parse::<u8>() {
                    current_octave = octave;
                    ast.push(MucomAstNode::Octave {
                        octave,
                        line: token.line,
                    });
                }
            }

            "volume_cmd" => {
                if token.value.len() > 1 {
                    // Could be V+, V-, or Vn
                    if let Some(val_str) = token.value.get(1..) {
                        if val_str.starts_with('+') || val_str.starts_with('-') {
                            // Relative volume change
                            // For now, skip detailed parsing
                        } else if let Ok(volume) = val_str.parse::<u8>() {
                            current_volume = volume;
                            ast.push(MucomAstNode::Volume {
                                volume,
                                line: token.line,
                            });
                        }
                    }
                }
            }

            "length_cmd" => {
                if let Some(len_str) = token.value.get(1..) {
                    if let Ok(length) = len_str.parse::<u8>() {
                        current_length = length;
                        ast.push(MucomAstNode::Length {
                            length,
                            line: token.line,
                        });
                    }
                }
            }

            "tempo_cmd" => {
                if let Some(tempo_str) = token.value.get(1..) {
                    if let Ok(tempo) = tempo_str.parse::<u8>() {
                        ast.push(MucomAstNode::Tempo {
                            tempo,
                            line: token.line,
                        });
                    }
                }
            }

            "loop_start" => {
                // Handle (n or (
                let count = if let Some(next) = iter.peek() {
                    if next.token_type == "number" {
                        let c = next.value.parse::<u8>().unwrap_or(1);
                        iter.next();
                        Some(c)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Collect loop body
                let mut body = Vec::new();
                while let Some(next) = iter.peek() {
                    if next.token_type == "loop_end" {
                        iter.next(); // Consume the )
                        break;
                    }
                    if let Some(t) = iter.next() {
                        // For now, just count - full parsing would recurse
                        body.push(MucomAstNode::Comment {
                            text: format!("loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(MucomAstNode::Loop {
                    count,
                    body,
                    line: token.line,
                });
            }

            "loop_start_infinite" => {
                // Handle [ ... ]
                let mut body = Vec::new();
                while let Some(next) = iter.peek() {
                    if next.token_type == "loop_end_infinite" {
                        iter.next(); // Consume the ]
                        break;
                    }
                    if let Some(t) = iter.next() {
                        body.push(MucomAstNode::Comment {
                            text: format!("infinite loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(MucomAstNode::LoopInfinite {
                    body,
                    line: token.line,
                });
            }

            "loop_end" | "loop_end_infinite" => {
                // These should be handled by loop_start, but add a break for safety
                ast.push(MucomAstNode::LoopBreak { line: token.line });
            }

            "directive" => {
                let name = token.value.trim_start_matches('#').to_string();
                ast.push(MucomAstNode::Directive {
                    name,
                    value: None,
                    line: token.line,
                });
            }

            "instrument_cmd" => {
                if let Some(instr_str) = token.value.get(1..) {
                    if let Ok(instrument) = instr_str.parse::<u8>() {
                        ast.push(MucomAstNode::Instrument {
                            instrument,
                            line: token.line,
                        });
                    }
                }
            }

            "comment" | "bar" | "whitespace" => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_id() {
        let driver = MucomDriver;
        assert_eq!(driver.id(), "mucom");
        assert_eq!(driver.display_name(), "mucom88 (MUC)");
    }

    #[test]
    fn test_driver_extensions() {
        let driver = MucomDriver;
        let exts = driver.supported_extensions();
        assert!(exts.contains(&".muc"));
    }

    #[test]
    fn test_detect_extension() {
        let driver = MucomDriver;
        assert_eq!(driver.detect("test", Some("song.muc")), 85);
    }

    #[test]
    fn test_detect_mucom_directive() {
        let driver = MucomDriver;
        let content = "#MUCOM88\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 95);
    }

    #[test]
    fn test_detect_ym2612() {
        let driver = MucomDriver;
        let content = "; YM2612 MML\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_sega() {
        let driver = MucomDriver;
        let content = "; Sega Mega Drive\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 75);
    }

    #[test]
    fn test_tokenize_basic() {
        let content = "@0 o4 c4 d4 e4 f4";
        let result = MucomDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());

        // Check for part command
        assert!(tokens.iter().any(|t| t.token_type == "part_cmd"));
        // Check for notes
        assert!(tokens.iter().any(|t| t.token_type == "note"));
    }

    #[test]
    fn test_tokenize_with_sharp() {
        let content = "c# d e f#";
        let result = MucomDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "note_sharp"));
    }

    #[test]
    fn test_tokenize_loops() {
        let content = "@0 (4 c4 d4 e4 f4)";
        let result = MucomDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "loop_start"));
        assert!(tokens.iter().any(|t| t.token_type == "loop_end"));
    }

    #[test]
    fn test_tokenize_directive() {
        let content = "#MUCOM88\n#VOICE voice.dat";
        let result = MucomDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "directive"));
    }

    #[test]
    fn test_parse_basic() {
        let content = "@0 o4 c4 d4 e4";
        let result = mucom_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(!ast.is_empty());
    }

    #[test]
    fn test_parse_with_part() {
        let content = "@0 o4 c4\n@1 o5 e4";
        let result = mucom_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        // Should have at least one PartSelect node
        assert!(ast
            .iter()
            .any(|n| matches!(n, MucomAstNode::PartSelect { .. })));
    }
}
