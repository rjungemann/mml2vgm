//! External Driver Support Module
//!
//! This module provides the infrastructure for external MML format drivers.
//! Each driver implements the `ExternalDriver` trait and can be registered
//! with the `DriverRegistry` for use in the browser IDE via WASM.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::MmlError;
use crate::compiler::lexer;

// Re-export driver modules
pub mod m98;
pub use m98::M98Driver;

pub mod mucom;
pub use mucom::MucomDriver;

pub mod moondriver;
pub use moondriver::MoonDriver;

pub mod pmd;
pub use pmd::PMDDriver;

pub mod muap;
pub use muap::MuapDriver;

/// Supported output formats for compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverOutputFormat {
    /// VGM (Video Game Music) format
    VGM,
    /// XGM (eXtended Game Music) format
    XGM,
    /// XGM2 format
    XGM2,
    /// ZGM format
    ZGM,
}

impl std::fmt::Display for DriverOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverOutputFormat::VGM => write!(f, "vgm"),
            DriverOutputFormat::XGM => write!(f, "xgm"),
            DriverOutputFormat::XGM2 => write!(f, "xgm2"),
            DriverOutputFormat::ZGM => write!(f, "zgm"),
        }
    }
}

impl std::str::FromStr for DriverOutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vgm" => Ok(DriverOutputFormat::VGM),
            "xgm" => Ok(DriverOutputFormat::XGM),
            "xgm2" => Ok(DriverOutputFormat::XGM2),
            "zgm" => Ok(DriverOutputFormat::ZGM),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Compilation options specific to external drivers
#[derive(Debug, Clone)]
pub struct DriverCompileOptions {
    /// Output format for compilation
    pub output_format: DriverOutputFormat,
    /// Target sample rate
    pub sample_rate: u32,
    /// Verbose output
    pub verbose: bool,
    /// Debug mode
    pub debug: bool,
    /// Additional driver-specific options as key-value pairs
    pub extra: HashMap<String, String>,
}

impl Default for DriverCompileOptions {
    fn default() -> Self {
        Self {
            output_format: DriverOutputFormat::VGM,
            sample_rate: 44100,
            verbose: false,
            debug: false,
            extra: HashMap::new(),
        }
    }
}

/// Diagnostic information for validation errors
#[derive(Debug, Clone)]
pub struct DriverDiagnostic {
    /// Error or warning message
    pub message: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Length of the problematic text
    pub length: usize,
}

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Token information for syntax highlighting
#[derive(Debug, Clone)]
pub struct DriverToken {
    /// Token type (e.g., "note", "command", "number", "string")
    pub token_type: String,
    /// Token value/text
    pub value: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Length of the token
    pub length: usize,
}

/// Result of compiling MML with an external driver
#[derive(Debug, Clone)]
pub struct DriverCompileResult {
    /// Compiled binary data (VGM, XGM, etc.)
    pub data: Vec<u8>,
    /// Number of parts/channels used
    pub part_count: usize,
    /// Number of commands processed
    pub command_count: usize,
    /// Duration in samples
    pub duration_samples: u64,
    /// Duration in seconds
    pub duration_seconds: f64,
    /// List of chips used in the compilation
    pub chips_used: Vec<String>,
    /// Any warnings generated during compilation
    pub warnings: Vec<DriverDiagnostic>,
}

/// Information about a driver
#[derive(Debug, Clone)]
pub struct DriverInfo {
    /// Unique identifier for the driver
    pub id: String,
    /// Display name
    pub display_name: String,
    /// List of file extensions supported (e.g., [".muc", ".mml"])
    pub supported_extensions: Vec<String>,
    /// Description of the driver
    pub description: String,
    /// Version string
    pub version: String,
    /// Target platform (e.g., "Sega Mega Drive", "NEC PC-9801")
    pub target_platform: String,
}

/// Common trait for all external MML drivers
pub trait ExternalDriver: Send + Sync {
    /// Get the unique identifier for this driver
    fn id(&self) -> &str;

    /// Get the display name for this driver
    fn display_name(&self) -> &str;

    /// Get the list of supported file extensions
    fn supported_extensions(&self) -> &[&str];

    /// Get a description of this driver
    fn description(&self) -> &str;

    /// Get the target platform for this driver
    fn target_platform(&self) -> &str;

    /// Get the version of this driver
    fn version(&self) -> &str;

    /// Detect if the given content is likely in this format
    /// Returns a confidence score from 0-100
    fn detect(&self, content: &str, filename: Option<&str>) -> u8;

    /// Validate the given MML content
    /// Returns a list of diagnostics (errors, warnings, etc.)
    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError>;

    /// Tokenize the given MML content for syntax highlighting
    /// Returns a list of tokens
    fn tokenize(&self, content: &str) -> Result<Vec<DriverToken>, MmlError>;

    /// Compile the given MML content
    /// Returns the compiled binary data and metadata
    fn compile(
        &self,
        content: &str,
        options: &DriverCompileOptions,
    ) -> Result<DriverCompileResult, MmlError>;

    /// Get information about this driver
    fn info(&self) -> DriverInfo {
        DriverInfo {
            id: self.id().to_string(),
            display_name: self.display_name().to_string(),
            supported_extensions: self
                .supported_extensions()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            description: self.description().to_string(),
            version: self.version().to_string(),
            target_platform: self.target_platform().to_string(),
        }
    }
}

/// Registry for managing external drivers
pub struct DriverRegistry {
    drivers: HashMap<String, Arc<dyn ExternalDriver>>,
}

impl DriverRegistry {
    /// Create a new, empty driver registry
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }

    /// Register a driver with the registry
    pub fn register_driver(&mut self, driver: Arc<dyn ExternalDriver>) {
        let id = driver.id().to_string();
        self.drivers.insert(id, driver);
    }

    /// Get a driver by its ID
    pub fn get_driver(&self, id: &str) -> Option<Arc<dyn ExternalDriver>> {
        self.drivers.get(id).cloned()
    }

    /// Get all registered drivers
    pub fn get_all_drivers(&self) -> Vec<Arc<dyn ExternalDriver>> {
        self.drivers.values().cloned().collect()
    }

    /// Get information about all registered drivers
    pub fn get_driver_infos(&self) -> Vec<DriverInfo> {
        self.drivers.values().map(|d| d.info()).collect()
    }

    /// Detect the format of the given content
    /// Returns the driver ID and confidence score for the best match
    pub fn detect_format(&self, content: &str, filename: Option<&str>) -> Option<(String, u8)> {
        let mut best_match: Option<(String, u8)> = None;

        for (id, driver) in &self.drivers {
            let confidence = driver.detect(content, filename);
            if confidence > 0 {
                match &best_match {
                    Some((_, best_confidence)) if confidence > *best_confidence => {
                        best_match = Some((id.clone(), confidence));
                    }
                    None => {
                        best_match = Some((id.clone(), confidence));
                    }
                    _ => {}
                }
            }
        }

        // Only return a match if confidence is at least 30
        best_match.filter(|(_, confidence)| *confidence >= 30)
    }

    /// Get a driver by file extension
    pub fn get_driver_by_extension(&self, extension: &str) -> Option<Arc<dyn ExternalDriver>> {
        let ext = extension.trim_start_matches('.').to_lowercase();
        for driver in self.drivers.values() {
            for supported_ext in driver.supported_extensions() {
                if supported_ext.trim_start_matches('.').to_lowercase() == ext {
                    return Some(driver.clone());
                }
            }
        }
        None
    }

    /// Check if a driver is registered for a given format ID
    pub fn has_driver(&self, id: &str) -> bool {
        self.drivers.contains_key(id)
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Native GWI Driver Implementation
// ============================================================================

/// Native GWI format driver using the existing mml2vgm compiler
pub struct GwiDriver;

impl ExternalDriver for GwiDriver {
    fn id(&self) -> &str {
        "gwi"
    }

    fn display_name(&self) -> &str {
        "mml2vgm (GWI)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".gwi", ".txt"]
    }

    fn description(&self) -> &str {
        "mml2vgm native MML format"
    }

    fn target_platform(&self) -> &str {
        "Multi-platform"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn detect(&self, content: &str, _filename: Option<&str>) -> u8 {
        // Check for @ commands which are specific to mml2vgm format
        let at_commands = content.matches('@').count();
        let total_commands: usize = content.chars().filter(|c| c.is_alphabetic()).count();

        if at_commands > 0 && total_commands > 0 {
            // Calculate confidence based on ratio of @ commands to total commands
            let ratio = at_commands as f32 / total_commands as f32;
            (50.0 + ratio * 50.0).min(100.0) as u8
        } else {
            0
        }
    }

    fn validate(&self, content: &str) -> Result<Vec<DriverDiagnostic>, MmlError> {
        // Use the existing lexer to validate
        match lexer::tokenize(content) {
            Ok(_) => Ok(Vec::new()),
            Err(e) => Err(MmlError::Compilation(e.to_string())),
        }
    }

    fn tokenize(&self, content: &str) -> Result<Vec<DriverToken>, MmlError> {
        match lexer::tokenize(content) {
            Ok(tokens) => {
                let result: Vec<DriverToken> = tokens
                    .into_iter()
                    .map(|(token, pos)| DriverToken {
                        token_type: token_type_to_string(&token),
                        value: token_to_string(&token),
                        line: pos.line,
                        column: pos.column,
                        length: token_to_length(&token),
                    })
                    .collect();
                Ok(result)
            }
            Err(e) => Err(MmlError::Compilation(e.to_string())),
        }
    }

    fn compile(
        &self,
        content: &str,
        options: &DriverCompileOptions,
    ) -> Result<DriverCompileResult, MmlError> {
        use crate::CompileOptions;
        use crate::OutputFormat;

        // Convert driver options to core compile options
        let mut core_options = CompileOptions::default();
        core_options.format = match options.output_format {
            DriverOutputFormat::VGM => OutputFormat::VGM,
            DriverOutputFormat::XGM => OutputFormat::XGM,
            DriverOutputFormat::XGM2 => OutputFormat::XGM2,
            DriverOutputFormat::ZGM => OutputFormat::ZGM,
        };

        let compiler = crate::compiler::compiler::MmlCompiler::new(core_options);
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

/// Convert a lexer token to a string type
fn token_type_to_string(token: &lexer::Token) -> String {
    match token {
        lexer::Token::Number(_) => "number".to_string(),
        lexer::Token::StringLiteral(_) => "string".to_string(),
        lexer::Token::Identifier(_) => "identifier".to_string(),
        lexer::Token::LeftBrace => "left_brace".to_string(),
        lexer::Token::RightBrace => "right_brace".to_string(),
        lexer::Token::Apostrophe => "apostrophe".to_string(),
        lexer::Token::Equals => "equals".to_string(),
        lexer::Token::Comma => "comma".to_string(),
        lexer::Token::LeftBracket => "left_bracket".to_string(),
        lexer::Token::RightBracket => "right_bracket".to_string(),
        lexer::Token::LeftParen => "left_paren".to_string(),
        lexer::Token::RightParen => "right_paren".to_string(),
        lexer::Token::Note(_) => "note".to_string(),
        lexer::Token::Sharp => "sharp".to_string(),
        lexer::Token::Flat => "flat".to_string(),
        lexer::Token::Rest => "rest".to_string(),
        lexer::Token::Duration(_) => "duration".to_string(),
        lexer::Token::Dot => "dot".to_string(),
        lexer::Token::Underscore => "tie".to_string(),
        lexer::Token::GreaterThan => "octave_up".to_string(),
        lexer::Token::LessThan => "octave_down".to_string(),
        lexer::Token::OctaveCommand => "octave_cmd".to_string(),
        lexer::Token::VolumeCommand => "volume_cmd".to_string(),
        lexer::Token::TempoCommand => "tempo_cmd".to_string(),
        lexer::Token::LengthCommand => "length_cmd".to_string(),
        lexer::Token::AtSign => "part_cmd".to_string(),
        lexer::Token::Bar => "bar".to_string(),
        lexer::Token::Comment(_) => "comment".to_string(),
        lexer::Token::Whitespace(_) => "whitespace".to_string(),
        lexer::Token::Eof => "eof".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Convert a lexer token to its string representation
fn token_to_string(token: &lexer::Token) -> String {
    match token {
        lexer::Token::Number(n) => n.to_string(),
        lexer::Token::StringLiteral(s) => s.clone(),
        lexer::Token::Identifier(s) => s.clone(),
        lexer::Token::Note(c) => c.to_string(),
        lexer::Token::Duration(n) => n.to_string(),
        lexer::Token::Comment(s) => s.clone(),
        lexer::Token::Whitespace(s) => s.clone(),
        lexer::Token::Sharp => "#".to_string(),
        lexer::Token::Flat => "b".to_string(),
        lexer::Token::Rest => "r".to_string(),
        lexer::Token::Dot => ".".to_string(),
        lexer::Token::Underscore => "_".to_string(),
        lexer::Token::GreaterThan => ">".to_string(),
        lexer::Token::LessThan => "<".to_string(),
        lexer::Token::AtSign => "@".to_string(),
        lexer::Token::Bar => "|".to_string(),
        lexer::Token::LeftBrace => "{".to_string(),
        lexer::Token::RightBrace => "}".to_string(),
        lexer::Token::Apostrophe => "'".to_string(),
        lexer::Token::Equals => "=".to_string(),
        lexer::Token::Comma => ",".to_string(),
        lexer::Token::LeftBracket => "[".to_string(),
        lexer::Token::RightBracket => "]".to_string(),
        lexer::Token::LeftParen => "(".to_string(),
        lexer::Token::RightParen => ")".to_string(),
        lexer::Token::OctaveCommand => "o".to_string(),
        lexer::Token::VolumeCommand => "v".to_string(),
        lexer::Token::TempoCommand => "t".to_string(),
        lexer::Token::LengthCommand => "l".to_string(),
        lexer::Token::Eof => "".to_string(),
        _ => "".to_string(),
    }
}

/// Get the length of a token's text representation
fn token_to_length(token: &lexer::Token) -> usize {
    token_to_string(token).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_registry() {
        let mut registry = DriverRegistry::new();
        let gwi_driver = Arc::new(GwiDriver);
        let m98_driver = Arc::new(M98Driver);
        let mucom_driver = Arc::new(MucomDriver);
        let moondriver_driver = Arc::new(MoonDriver);
        let pmd_driver = Arc::new(PMDDriver);
        registry.register_driver(gwi_driver.clone());
        registry.register_driver(m98_driver.clone());
        registry.register_driver(mucom_driver.clone());
        registry.register_driver(moondriver_driver.clone());
        registry.register_driver(pmd_driver.clone());

        assert!(registry.has_driver("gwi"));
        assert!(registry.has_driver("m98"));
        assert!(registry.has_driver("mucom"));
        assert!(registry.has_driver("moondriver"));
        assert!(registry.has_driver("pmd"));

        let driver = registry.get_driver("gwi");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().id(), "gwi");

        let driver = registry.get_driver("m98");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().id(), "m98");

        let driver = registry.get_driver("mucom");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().id(), "mucom");

        let driver = registry.get_driver("moondriver");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().id(), "moondriver");

        let driver = registry.get_driver("pmd");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().id(), "pmd");
    }

    #[test]
    fn test_gwi_driver_detect() {
        let driver = GwiDriver;
        let content = "@0 v100 o4 cdefgab>c";
        let confidence = driver.detect(content, None);
        assert!(confidence > 0);
    }

    #[test]
    fn test_gwi_driver_tokenize() {
        let driver = GwiDriver;
        let content = "@0 v100 o4 c4 d4 e4 f4";
        let result = driver.tokenize(content);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.len() > 0);
    }
}
