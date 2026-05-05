//! M98 External Driver
//!
//! This module implements the M98 MML format driver for NEC PC-9801 systems.
//! M98 is a simplified MML format targeting YM2203 and YM2608 sound chips.

use crate::drivers::{
    DiagnosticSeverity, DriverCompileOptions, DriverCompileResult, DriverDiagnostic, DriverInfo,
    DriverOutputFormat, DriverToken, ExternalDriver,
};
use crate::{CompileOptions, OutputFormat, SoundChip, error::MmlError};

/// M98 Driver implementation
pub struct M98Driver;

impl ExternalDriver for M98Driver {
    fn id(&self) -> &str {
        "m98"
    }

    fn display_name(&self) -> &str {
        "M98 (PC-9801)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".m98"]
    }

    fn description(&self) -> &str {
        "M98 MML format for NEC PC-9801 (YM2203/YM2608)"
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
            if name.to_lowercase().ends_with(".m98") {
                return 80;
            }
        }

        // Check for M98-specific patterns in content
        let content_lower = content.to_lowercase();

        // High confidence: M98 directive or comment
        if content_lower.contains("m98") || content_lower.contains("m-98") || content_lower.contains("pc-98")
        {
            return 90;
        }

        // Medium confidence: YM2203/YM2608 specific commands
        if content_lower.contains("@") {
            // Check if @ commands look like M98 part selectors
            if content.matches('@').count() > 0 && content.chars().filter(|c| c.is_digit(10)).count() > 0 {
                return 60;
            }
        }

        // Low confidence: generic MML with YM2203/YM2608 mention
        if content_lower.contains("ym2203") || content_lower.contains("ym2608") || content_lower.contains("opna")
        {
            return 40;
        }

        0
    }

    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError> {
        // Parse and validate the M98 content
        // For now, just do basic validation
        // A proper implementation would parse and validate the AST
        
        // Check for invalid characters
        for (line_idx, line) in content.lines().enumerate() {
            for (col_idx, c) in line.chars().enumerate() {
                // Allow alphanumeric, spaces, and MML-specific characters
                if !c.is_alphanumeric() && !c.is_whitespace() && 
                   !"#@\\|;,=+-[].<>/?{}".contains(c) {
                    return Ok(vec![DriverDiagnostic {
                        message: format!("Invalid character: {}", c),
                        severity: DiagnosticSeverity::Error,
                        line: line_idx + 1,
                        column: col_idx + 1,
                        length: c.len_utf8(),
                    }]);
                }
            }
        }
        
        Ok(Vec::new())
    }

    fn tokenize(&self, content: &str) -> Result<Vec<DriverToken>, MmlError> {
        m98_tokenize(content)
    }

    fn compile(
        &self,
        content: &str,
        options: &DriverCompileOptions,
    ) -> Result<DriverCompileResult, MmlError> {
        // For now, use the existing mml2vgm compiler with YM2608 target
        // In the future, implement a dedicated M98 compiler
        
        // Convert output format
        let output_format = match options.output_format {
            DriverOutputFormat::VGM => OutputFormat::VGM,
            DriverOutputFormat::XGM => OutputFormat::XGM,
            DriverOutputFormat::XGM2 => OutputFormat::XGM2,
            DriverOutputFormat::ZGM => OutputFormat::ZGM,
        };

        // Create compile options with YM2608 as default (most common for PC-9801)
        let mut compile_options = CompileOptions::default();
        compile_options.format = output_format;
        compile_options.target_chips = Some(vec![SoundChip::YM2608, SoundChip::SN76489]);

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
// M98 Tokenizer
// ============================================================================

/// Token for M98 syntax highlighting
#[derive(Debug, Clone)]
pub struct M98Token {
    pub token_type: String,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

/// Tokenize M98 content for syntax highlighting
fn m98_tokenize(content: &str) -> Result<Vec<DriverToken>, MmlError> {
    let mut tokens = Vec::new();
    let mut chars = content.chars().peekable();
    let mut line = 1usize;
    let mut column = 1usize;
    let mut pos = 0usize;

    while let Some(c) = chars.next() {
        let start_line = line;
        let start_column = column;
        let start_pos = pos;

        // Handle newlines
        if c == '\n' {
            line += 1;
            column = 1;
            pos += 1;
            continue;
        }

        if c == '\r' {
            pos += 1;
            continue;
        }

        // Handle whitespace
        if c.is_whitespace() {
            pos += c.len_utf8();
            column += 1;
            continue;
        }

        // Notes: A-G, a-g, with optional # or - (flat)
        // In MML, 'b' is a note (B), not a flat indicator
        // Flat is represented by '-' after the note (e.g., C-)
        if ('A'..='G').contains(&c) || ('a'..='g').contains(&c) {
            let note_upper = c.to_ascii_uppercase();
            let mut note_str = note_upper.to_string();
            let mut token_type = "note".to_string();
            let mut length = 1;

            // Check for sharp or flat
            if let Some(&next_c) = chars.peek() {
                if next_c == '#' {
                    note_str.push('#');
                    token_type = "note_sharp".to_string();
                    chars.next();
                    length += 1;
                } else if next_c == '-' {
                    note_str.push('-');
                    token_type = "note_flat".to_string();
                    chars.next();
                    length += 1;
                }
            }

            tokens.push(DriverToken {
                token_type: token_type.clone(),
                value: note_str,
                line: start_line,
                column: start_column,
                length,
            });
            pos += c.len_utf8();
            column += 1;
            continue;
        }

        // Rest
        if c == 'R' || c == 'r' {
            tokens.push(DriverToken {
                token_type: "rest".to_string(),
                value: "r".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Part command (@)
        if c == '@' {
            tokens.push(DriverToken {
                token_type: "part_cmd".to_string(),
                value: "@".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Octave command (O or o)
        if c == 'O' || c == 'o' {
            tokens.push(DriverToken {
                token_type: "octave_cmd".to_string(),
                value: "o".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Volume command (V or v)
        if c == 'V' || c == 'v' {
            tokens.push(DriverToken {
                token_type: "volume_cmd".to_string(),
                value: "v".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Length command (L or l)
        if c == 'L' || c == 'l' {
            tokens.push(DriverToken {
                token_type: "length_cmd".to_string(),
                value: "l".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Tempo command (T or t)
        if c == 'T' || c == 't' {
            tokens.push(DriverToken {
                token_type: "tempo_cmd".to_string(),
                value: "t".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Bar (|)
        if c == '|' {
            tokens.push(DriverToken {
                token_type: "bar".to_string(),
                value: "|".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Comment (;)
        if c == ';' {
            let start_pos = pos;
            let mut comment_chars: Vec<char> = Vec::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == '\n' {
                    break;
                }
                comment_chars.push(chars.next().unwrap());
            }
            let comment: String = comment_chars.into_iter().collect();
            tokens.push(DriverToken {
                token_type: "comment".to_string(),
                value: comment.clone(),
                line: start_line,
                column: start_column,
                length: comment.len(),
            });
            pos += comment.len() + 1; // +1 for the semicolon
            column += comment.len() + 1;
            continue;
        }

        // Numbers
        if c.is_digit(10) {
            let start_pos = pos;
            let mut num_chars: Vec<char> = vec![c];
            while let Some(&next_c) = chars.peek() {
                if next_c.is_digit(10) {
                    num_chars.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            let num_str: String = num_chars.into_iter().collect();
            tokens.push(DriverToken {
                token_type: "number".to_string(),
                value: num_str.clone(),
                line: start_line,
                column: start_column,
                length: num_str.len(),
            });
            pos += num_str.len();
            column += num_str.len();
            continue;
        }

        // Tie (_)
        if c == '_' {
            tokens.push(DriverToken {
                token_type: "tie".to_string(),
                value: "_".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Octave up (>)
        if c == '>' {
            tokens.push(DriverToken {
                token_type: "octave_up".to_string(),
                value: ">".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Octave down (<)
        if c == '<' {
            tokens.push(DriverToken {
                token_type: "octave_down".to_string(),
                value: "<".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Dot (.)
        if c == '.' {
            tokens.push(DriverToken {
                token_type: "dot".to_string(),
                value: ".".to_string(),
                line: start_line,
                column: start_column,
                length: 1,
            });
            pos += 1;
            column += 1;
            continue;
        }

        // Default: unknown token
        tokens.push(DriverToken {
            token_type: "unknown".to_string(),
            value: c.to_string(),
            line: start_line,
            column: start_column,
            length: c.len_utf8(),
        });
        pos += c.len_utf8();
        column += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_id() {
        let driver = M98Driver;
        assert_eq!(driver.id(), "m98");
        assert_eq!(driver.display_name(), "M98 (PC-9801)");
    }

    #[test]
    fn test_driver_extensions() {
        let driver = M98Driver;
        let exts = driver.supported_extensions();
        assert!(exts.contains(&".m98"));
    }

    #[test]
    fn test_detect_extension() {
        let driver = M98Driver;
        assert_eq!(driver.detect("test", Some("song.m98")), 80);
    }

    #[test]
    fn test_detect_m98_directive() {
        let driver = M98Driver;
        let content = "M98\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 90);
    }

    #[test]
    fn test_detect_pc98() {
        let driver = M98Driver;
        let content = "; PC-9801 MML\n@0 o4 cdefg";
        assert!(driver.detect(content, None) >= 70);
    }

    #[test]
    fn test_tokenize_basic() {
        let content = "@0 o4 c4 d4 e4 f4";
        let result = M98Driver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.len() > 0);

        // Check for part command
        assert!(tokens.iter().any(|t| t.token_type == "part_cmd"));
        // Check for notes
        assert!(tokens.iter().any(|t| t.token_type == "note"));
    }

    #[test]
    fn test_tokenize_with_sharp_flat() {
        let content = "c# d- e";
        let result = M98Driver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.iter().any(|t| t.token_type == "note_sharp"));
        assert!(tokens.iter().any(|t| t.token_type == "note_flat"));
    }
}
