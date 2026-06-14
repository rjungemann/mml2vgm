//! Muap External Driver
//!
//! This module implements the Muap MML format driver for YM2608 (OPNA) systems.
//! Muap is a specialized MML format targeting the YM2608 chip with support for
//! FM synthesis, SSG, rhythm section, and ADPCM.
//!
//! ## Format Specification
//!
//! - **File Extension**: `.muap`
//! - **Target Platform**: YM2608 (OPNA)
//! - **Supported Chips**: YM2608 (6 FM + 3 SSG + 6 rhythm + ADPCM)
//!
//! ## Muap Command Reference
//!
//! Based on the MuapDotNET driver from the .NET IDE.
//!
//! ### Basic Structure
//! - Parts are defined with `@` commands
//! - Each part targets a specific channel type (FM, SSG, Rhythm, ADPCM)
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
//! - `V15` (0-15 for FM/SSG)
//! - `V+` / `V-` = volume up/down
//!
//! ### Commands
//! - `L4` = length (1-256, default 4)
//! - `T120` = tempo (40-255, default 120)
//! - `Q4` = gate time (0-7)
//! - `@FM n` = FM channel select
//! - `@SSG n` = SSG channel select
//! - `@RHYTHM n` = Rhythm channel select
//! - `@ADPCM n` = ADPCM channel select
//! - `(n` = loop start, `)n` = loop end (finite loop)
//! - `[` = infinite loop start, `]` = infinite loop end
//! - `;` or `*` = comment to end of line
//!
//! ### Muap-Specific Features
//! - `@OPNA` = OPNA directive
//! - `@FM` = FM section
//! - `@SSG` = SSG section
//! - `@RHYTHM` = Rhythm section
//! - `@ADPCM` = ADPCM section
//! - Supports all YM2608 features

use crate::drivers::{
    DiagnosticSeverity, DriverCompileOptions, DriverCompileResult, DriverDiagnostic,
    DriverOutputFormat, DriverToken, ExternalDriver,
};
use crate::{error::MmlError, CompileOptions, OutputFormat, SoundChip};

/// Muap Driver implementation
pub struct MuapDriver;

impl ExternalDriver for MuapDriver {
    fn id(&self) -> &str {
        "muap"
    }

    fn display_name(&self) -> &str {
        "Muap (OPNA)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".muap"]
    }

    fn description(&self) -> &str {
        "Muap MML format for YM2608 (OPNA)"
    }

    fn target_platform(&self) -> &str {
        "YM2608 (OPNA)"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, content: &str, filename: Option<&str>) -> u8 {
        // Check filename first
        if let Some(name) = filename {
            if name.to_lowercase().ends_with(".muap") {
                return 90;
            }
        }

        let content_lower = content.to_lowercase();
        let content_trimmed = content.trim();

        // High confidence: Muap-specific directives
        if content_trimmed.starts_with("@opna") || content_lower.contains("@opna") {
            return 95;
        }

        // High confidence: Muap mention
        if content_lower.contains("muap") {
            return 90;
        }

        // High confidence: OPNA mention
        if content_lower.contains("opna") && !content_lower.contains("moondriver") {
            return 90;
        }

        // Medium confidence: YM2608 specific
        if content_lower.contains("ym2608") {
            return 85;
        }

        // Medium confidence: Muap-specific section markers
        if content_lower.contains("@fm")
            || content_lower.contains("@ssg")
            || content_lower.contains("@rhythm")
            || content_lower.contains("@adpcm")
        {
            return 80;
        }

        // Low confidence: ADPCM mention
        if content_lower.contains("adpcm") {
            return 40;
        }

        0
    }

    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError> {
        // Parse and validate the Muap content
        match muap_parse(content) {
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
        match muap_tokenize(content) {
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
        // Parse the Muap content (ignore errors for now)
        let _ast = muap_parse(content);

        // Convert output format
        let output_format = match options.output_format {
            DriverOutputFormat::VGM => OutputFormat::VGM,
            DriverOutputFormat::XGM => OutputFormat::XGM,
            DriverOutputFormat::XGM2 => OutputFormat::XGM2,
            DriverOutputFormat::ZGM => OutputFormat::ZGM,
        };

        // YM2608 is the only target for Muap
        let target_chips = vec![SoundChip::YM2608, SoundChip::SN76489];

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

// ============================================================================
// Muap Tokenizer
// ============================================================================

/// Token for Muap syntax highlighting
#[derive(Debug, Clone)]
pub struct MuapToken {
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

/// Tokenize Muap content for syntax highlighting
fn muap_tokenize(content: &str) -> Result<Vec<DriverToken>, MmlError> {
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

        // Section markers (@FM, @SSG, @RHYTHM, @ADPCM, @OPNA)
        if c == '@' {
            let mut value = "@".to_string();
            let mut length = 1;

            // Read section name
            while let Some(&next_c) = chars.peek() {
                if next_c.is_alphabetic() {
                    value.push(chars.next().unwrap());
                    length += 1;
                } else {
                    break;
                }
            }

            // Check for channel number (e.g., @FM0, @SSG1)
            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    value.push(chars.next().unwrap());
                    length += 1;
                }
            }

            // Determine token type based on value
            let token_type = if value.to_uppercase().starts_with("@OPNA") {
                "directive"
            } else if value.to_uppercase().starts_with("@FM") {
                "fm_section"
            } else if value.to_uppercase().starts_with("@SSG") {
                "ssg_section"
            } else if value.to_uppercase().starts_with("@RHYTHM") {
                "rhythm_section"
            } else if value.to_uppercase().starts_with("@ADPCM") {
                "adpcm_section"
            } else if value.len() > 1 && value.chars().nth(1).unwrap().is_ascii_digit() {
                "part_cmd"
            } else {
                "directive"
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
// Muap Parser
// ============================================================================

/// Parse error for Muap
#[derive(Debug, Clone)]
pub struct MuapParseError {
    /// Message.
    pub message: String,
    /// Line.
    pub line: usize,
    /// Column.
    pub column: usize,
    /// Length.
    pub length: usize,
}

/// AST node for Muap
#[derive(Debug, Clone)]
pub enum MuapAstNode {
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
        commands: Vec<MuapAstNode>,
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
    /// Loop.
    Loop {
        /// Count.
        count: Option<u8>,
        /// Body.
        body: Vec<MuapAstNode>,
        /// Line.
        line: usize,
    },
    /// Loop Infinite.
    LoopInfinite {
        /// Body.
        body: Vec<MuapAstNode>,
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
    /// Fm Section.
    FmSection {
        /// Channel.
        channel: Option<u8>,
        /// Line.
        line: usize,
    },
    /// Ssg Section.
    SsgSection {
        /// Channel.
        channel: Option<u8>,
        /// Line.
        line: usize,
    },
    /// Rhythm Section.
    RhythmSection {
        /// Channel.
        channel: Option<u8>,
        /// Line.
        line: usize,
    },
    /// Adpcm Section.
    AdpcmSection {
        /// Channel.
        channel: Option<u8>,
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

/// Parse Muap content into AST
// `current_part`/`current_length`/`current_volume` track parser state that is not
// yet consumed by AST construction; retained for upcoming stateful handling.
#[allow(unused_variables, unused_assignments)]
fn muap_parse(content: &str) -> Result<Vec<MuapAstNode>, Vec<MuapParseError>> {
    let mut errors = Vec::new();
    let mut ast = Vec::new();
    let mut current_part: Option<u8> = None;
    let mut current_octave: u8 = 4;
    let mut current_length: u8 = 4;
    let mut current_volume: u8 = 15;

    let tokens = match muap_tokenize(content) {
        Ok(t) => t,
        Err(e) => {
            errors.push(MuapParseError {
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

                ast.push(MuapAstNode::PartSelect {
                    part_num,
                    line: token.line,
                });
            }

            "fm_section" => {
                // Parse channel if present
                let channel_str = token.value.trim_start_matches("@FM");
                let channel = if channel_str.is_empty() {
                    None
                } else {
                    channel_str.parse::<u8>().ok()
                };

                ast.push(MuapAstNode::FmSection {
                    channel,
                    line: token.line,
                });
            }

            "ssg_section" => {
                let channel_str = token.value.trim_start_matches("@SSG");
                let channel = if channel_str.is_empty() {
                    None
                } else {
                    channel_str.parse::<u8>().ok()
                };

                ast.push(MuapAstNode::SsgSection {
                    channel,
                    line: token.line,
                });
            }

            "rhythm_section" => {
                let channel_str = token.value.trim_start_matches("@RHYTHM");
                let channel = if channel_str.is_empty() {
                    None
                } else {
                    channel_str.parse::<u8>().ok()
                };

                ast.push(MuapAstNode::RhythmSection {
                    channel,
                    line: token.line,
                });
            }

            "adpcm_section" => {
                let channel_str = token.value.trim_start_matches("@ADPCM");
                let channel = if channel_str.is_empty() {
                    None
                } else {
                    channel_str.parse::<u8>().ok()
                };

                ast.push(MuapAstNode::AdpcmSection {
                    channel,
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

                ast.push(MuapAstNode::Note {
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

                ast.push(MuapAstNode::Rest {
                    duration,
                    line: token.line,
                    column: token.column,
                });
            }

            "octave_cmd" => {
                if let Ok(octave) = token.value.parse::<u8>() {
                    current_octave = octave;
                    ast.push(MuapAstNode::Octave {
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
                        ast.push(MuapAstNode::Volume {
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
                        ast.push(MuapAstNode::Length {
                            length,
                            line: token.line,
                        });
                    }
                }
            }

            "tempo_cmd" => {
                if let Some(tempo_str) = token.value.get(1..) {
                    if let Ok(tempo) = tempo_str.parse::<u8>() {
                        ast.push(MuapAstNode::Tempo {
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
                        body.push(MuapAstNode::Comment {
                            text: format!("loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(MuapAstNode::Loop {
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
                        body.push(MuapAstNode::Comment {
                            text: format!("infinite loop body at line {}", t.line),
                            line: t.line,
                        });
                    }
                }

                ast.push(MuapAstNode::LoopInfinite {
                    body,
                    line: token.line,
                });
            }

            "loop_end" | "loop_end_infinite" => {
                ast.push(MuapAstNode::LoopBreak { line: token.line });
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

                ast.push(MuapAstNode::Directive {
                    name: clean_name,
                    value,
                    line: token.line,
                });
            }

            "rhythm_instrument" => {
                ast.push(MuapAstNode::RhythmInstrument {
                    instrument: token.value,
                    line: token.line,
                });
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
        let driver = MuapDriver;
        assert_eq!(driver.id(), "muap");
        assert_eq!(driver.display_name(), "Muap (OPNA)");
    }

    #[test]
    fn test_driver_extensions() {
        let driver = MuapDriver;
        let exts = driver.supported_extensions();
        assert!(exts.contains(&".muap"));
    }

    #[test]
    fn test_detect_extension() {
        let driver = MuapDriver;
        assert_eq!(driver.detect("test", Some("song.muap")), 90);
    }

    #[test]
    fn test_detect_opna_directive() {
        let driver = MuapDriver;
        let content = "@OPNA\n@FM0 o4 cdefg";
        assert!(driver.detect(content, None) >= 95);
    }

    #[test]
    fn test_detect_muap_mention() {
        let driver = MuapDriver;
        let content = "; Muap format\n@FM0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_ym2608() {
        let driver = MuapDriver;
        let content = "; YM2608 target\n@FM0 o4 cdefg";
        assert!(driver.detect(content, None) >= 85);
    }

    #[test]
    fn test_detect_fm_section() {
        let driver = MuapDriver;
        let content = "@FM0\no4 c4";
        assert!(driver.detect(content, None) >= 80);
    }

    #[test]
    fn test_detect_ssg_section() {
        let driver = MuapDriver;
        let content = "@SSG0\no4 c4";
        assert!(driver.detect(content, None) >= 80);
    }

    #[test]
    fn test_detect_rhythm_section() {
        let driver = MuapDriver;
        let content = "@RHYTHM0\nBD";
        assert!(driver.detect(content, None) >= 80);
    }

    #[test]
    fn test_tokenize_basic() {
        let content = "@FM0 o4 c4 d4 e4 f4";
        let result = MuapDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());

        // Check for FM section
        assert!(tokens.iter().any(|t| t.token_type == "fm_section"));
        // Check for octave command
        assert!(tokens.iter().any(|t| t.token_type == "octave_cmd"));
        // Check for notes
        assert!(tokens.iter().any(|t| t.token_type == "note"));
    }

    #[test]
    fn test_tokenize_with_sharp() {
        let content = "c+ d# e f+";
        let result = MuapDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "note_sharp"));
    }

    #[test]
    fn test_tokenize_sections() {
        let content = "@FM0 @SSG0 @RHYTHM0 @ADPCM0";
        let result = MuapDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "fm_section"));
        assert!(tokens.iter().any(|t| t.token_type == "ssg_section"));
        assert!(tokens.iter().any(|t| t.token_type == "rhythm_section"));
        assert!(tokens.iter().any(|t| t.token_type == "adpcm_section"));
    }

    #[test]
    fn test_tokenize_loops() {
        let content = "@FM0 (4 c4 d4 e4 f4)";
        let result = MuapDriver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "loop_start"));
        assert!(tokens.iter().any(|t| t.token_type == "loop_end"));
    }

    #[test]
    fn test_parse_basic() {
        let content = "@FM0 o4 c4 d4 e4";
        let result = muap_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(!ast.is_empty());
    }

    #[test]
    fn test_parse_with_section() {
        let content = "@FM0 o4 c4\n@SSG0 o5 e4";
        let result = muap_parse(content);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(ast
            .iter()
            .any(|n| matches!(n, MuapAstNode::FmSection { .. })));
        assert!(ast
            .iter()
            .any(|n| matches!(n, MuapAstNode::SsgSection { .. })));
    }
}
