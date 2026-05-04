//! Error types for mml2vgm-rs

use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Primary error type for the mml2vgm library
#[derive(Debug, Error)]
pub enum MmlError {
    // IO Errors
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    // Parse Errors
    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        line: usize,
        column: usize,
        message: String,
    },

    // File Errors
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),

    // Compilation Errors
    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Unsupported command: {0}")]
    UnsupportedCommand(String),

    #[error("Invalid instrument reference: {0}")]
    InvalidInstrument(usize),

    #[error("Invalid part name: {0}")]
    InvalidPartName(String),

    // Chip Errors
    #[error("Unsupported sound chip: {0}")]
    UnsupportedChip(String),

    #[error("Chip initialization failed: {0}")]
    ChipInitFailed(String),

    // Audio Errors
    #[error("Audio error: {0}")]
    AudioError(#[from] crate::audio::AudioError),

    // VGM Format Errors
    #[error("Invalid VGM header: {0}")]
    InvalidVgmHeader(String),

    #[error("VGM version not supported: {0}")]
    UnsupportedVgmVersion(u32),

    #[error("Invalid output format: {0}")]
    InvalidOutputFormat(String),

    // PCM Errors
    #[error("PCM format not supported: {0}")]
    UnsupportedPcmFormat(String),

    #[error("PCM data too large: {0} bytes (max {1})")]
    PcmTooLarge(usize, usize),

    // Internal Errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience type for results that can return MmlError
pub type MmlResult<T> = Result<T, MmlError>;

/// Position information for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

impl Position {
    /// Create a new position (1-indexed)
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create position from 0-indexed byte offset and source
    pub fn from_offset(offset: usize, source: &str) -> Self {
        let mut line = 1;
        let mut column = 1;
        let mut current_offset = 0;

        for c in source.chars() {
            if current_offset >= offset {
                break;
            }
            match c {
                '\n' => {
                    line += 1;
                    column = 1;
                }
                '\r' => {
                    // Don't increment line for \r, it may be followed by \n
                    if current_offset + 1 < source.len() && source.chars().nth(current_offset + 1) != Some('\n') {
                        line += 1;
                        column = 1;
                    }
                }
                _ => {
                    column += 1;
                }
            }
            current_offset += c.len_utf8();
        }

        Self { line, column }
    }
}

/// Error context for better error messages
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub position: Position,
    pub message: String,
    pub source: Option<String>,
    pub help: Option<String>,
}

impl ErrorContext {
    pub fn new(position: Position, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
            source: None,
            help: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Convenience trait for converting common errors to MmlError
pub trait IntoMmlError<T> {
    fn into_mml_error(self) -> MmlError;
    fn into_mml_result(self) -> MmlResult<T>;
}

impl<T: std::fmt::Debug, E: Into<MmlError>> IntoMmlError<T> for Result<T, E> {
    fn into_mml_error(self) -> MmlError {
        self.unwrap_err().into()
    }

    fn into_mml_result(self) -> MmlResult<T> {
        self.map_err(Into::into)
    }
}

impl IntoMmlError<()> for io::Error {
    fn into_mml_error(self) -> MmlError {
        MmlError::Io(self)
    }

    fn into_mml_result(self) -> MmlResult<()> {
        Err(MmlError::Io(self))
    }
}

impl IntoMmlError<()> for ParseIntError {
    fn into_mml_error(self) -> MmlError {
        MmlError::Parse {
            line: 0,
            column: 0,
            message: format!("Invalid number: {}", self),
        }
    }

    fn into_mml_result(self) -> MmlResult<()> {
        Err(self.into_mml_error())
    }
}

impl IntoMmlError<String> for FromUtf8Error {
    fn into_mml_error(self) -> MmlError {
        MmlError::InvalidFileFormat(format!("Invalid UTF-8: {}", self))
    }

    fn into_mml_result(self) -> MmlResult<String> {
        Err(self.into_mml_error())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_offset() {
        let source = "line1\nline2\nline3";
        
        let pos = Position::from_offset(0, source);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        let pos = Position::from_offset(6, source); // After "line1\n"
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);

        let pos = Position::from_offset(12, source); // After "line1\nline2\n"
        assert_eq!(pos.line, 3);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new(
            Position::new(10, 5),
            "Test error"
        ).with_source("source code")
        .with_help("Try this");

        assert_eq!(ctx.position.line, 10);
        assert_eq!(ctx.position.column, 5);
        assert_eq!(ctx.message, "Test error");
        assert_eq!(ctx.source, Some(String::from("source code")));
        assert_eq!(ctx.help, Some(String::from("Try this")));
    }
}
