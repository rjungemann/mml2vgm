//! MML Compiler
//!
//! Orchestrates the full compilation pipeline from MML source to VGM/XGM/ZGM output.
//! Handles tokenization, parsing, semantic analysis, and code generation.

use crate::compiler::ast::{MmlAst, PartDefinition};
use crate::compiler::codegen::CodeGenerator;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::sample_resolver::{NoopSampleResolver, SampleResolver};
use crate::compiler::sema::Sema;
use crate::{CompileOptions, CompileResult, MmlError, MmlResult, OutputFormat};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Result of pre-processing the `'{...}` song info block.
struct PreprocessResult {
    /// Source with the song info block removed.
    source: String,
    /// Key-value metadata from the block.
    metadata: HashMap<String, String>,
    /// Maps part letter (uppercase) → chip name (e.g. 'A' → "YM2612").
    chip_map: HashMap<char, String>,
}

/// MML Compiler for converting MML source files to game music formats
pub struct MmlCompiler {
    options: CompileOptions,
}

impl MmlCompiler {
    /// Create a new compiler with the given options
    pub fn new(options: CompileOptions) -> Self {
        Self { options }
    }

    /// Compile an MML source file to the specified output format
    pub fn compile(&self, input_path: &Path) -> MmlResult<CompileResult> {
        // 1. Read + normalise source
        let source = self.read_file(input_path)?;

        // 2. Pre-process: extract '{...}' song info block before tokenisation
        let pre = self.preprocess_song_info(&source);

        // 3. Tokenize (Lexer)
        let tokens = self.lex(&pre.source)?;

        // 4. Parse (Parser)
        let mut ast = self.parse(tokens)?;

        // 5. Inject metadata and chip assignments from the header block
        for (k, v) in pre.metadata {
            ast.metadata.entry(k).or_insert(v);
        }
        self.apply_chip_assignments(&mut ast, &pre.chip_map);

        // 6. Code Generation
        let part_count = ast.parts.len();
        let output_data = self.generate_code(&ast)?;
        let info = Self::info_from_vgm(&output_data, part_count);

        // 7. Create result
        let output_path = self.determine_output_path(input_path);

        Ok(CompileResult {
            data: output_data,
            output_path: Some(output_path.to_string_lossy().to_string()),
            warnings: Vec::new(),
            info,
        })
    }

    /// Validate an MML file without generating output
    pub fn validate(&self, input_path: &Path) -> MmlResult<()> {
        let source = self.read_file(input_path)?;
        let pre = self.preprocess_song_info(&source);
        let tokens = self.lex(&pre.source)?;
        let _ast = self.parse(tokens)?;
        Ok(())
    }

    /// Read input file
    fn read_file(&self, path: &Path) -> MmlResult<String> {
        let source = fs::read_to_string(path).map_err(|e| {
            MmlError::UnsupportedCommand(format!("Failed to read file {}: {}", path.display(), e))
        })?;
        Ok(self.normalize_source(&source))
    }

    /// Normalize source before tokenization.
    fn normalize_source(&self, source: &str) -> String {
        // Strip UTF-8 BOM if present; without this the lexer loops forever on it.
        let source = source.strip_prefix('\u{FEFF}').unwrap_or(source);
        source
            .replace("\r\n", "\n")
            .split('\n')
            .map(|line| {
                if line.trim_start().starts_with(';') {
                    ""
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract the `'{...}` song info block from the source using raw text parsing.
    ///
    /// Returns the source with the block removed plus the extracted metadata and
    /// chip-to-letter mappings.  The lexer cannot handle the block's unquoted free-text
    /// values, so we parse it here before tokenisation.
    fn preprocess_song_info(&self, source: &str) -> PreprocessResult {
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut chip_map: HashMap<char, String> = HashMap::new();
        let mut out_lines: Vec<&str> = Vec::new();
        let mut in_block = false;

        for line in source.lines() {
            let trimmed = line.trim();

            if !in_block {
                // `'{` or bare `{` starts the block.
                let block_start = if trimmed.starts_with("'{") {
                    Some(trimmed[2..].trim())
                } else if trimmed == "{" {
                    Some("")
                } else {
                    None
                };

                if let Some(content) = block_start {
                    if content.ends_with('}') {
                        // Entire block on one line: '{ key = val }
                        let inner = content.trim_end_matches('}').trim();
                        Self::process_song_info_line(inner, &mut metadata, &mut chip_map);
                    } else {
                        in_block = true;
                        if !content.is_empty() {
                            Self::process_song_info_line(content, &mut metadata, &mut chip_map);
                        }
                    }
                    continue;
                }
            }

            if in_block {
                if trimmed == "}" {
                    in_block = false;
                } else {
                    Self::process_song_info_line(trimmed, &mut metadata, &mut chip_map);
                }
                continue;
            }

            out_lines.push(line);
        }

        PreprocessResult {
            source: out_lines.join("\n"),
            metadata,
            chip_map,
        }
    }

    /// Parse one line from inside the `'{...}` block.
    fn process_song_info_line(
        line: &str,
        metadata: &mut HashMap<String, String>,
        chip_map: &mut HashMap<char, String>,
    ) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            // PartYM2612 = A  →  chip_map['A'] = "YM2612"
            if key.starts_with("Part") && key[4..].chars().all(|c| c.is_alphanumeric()) {
                let chip_name = &key[4..];
                if let Some(letter) = value.chars().next() {
                    if letter.is_ascii_alphabetic() {
                        chip_map.insert(letter.to_ascii_uppercase(), chip_name.to_string());
                    }
                }
            } else {
                metadata.insert(key.to_string(), value.to_string());
            }
        } else {
            // Flag without value, e.g. ForcedMonoPartYM2612
            metadata.insert(line.to_string(), "true".to_string());
        }
    }

    /// Apply chip assignments from the header's `Part*` mappings to parsed parts.
    fn apply_chip_assignments(&self, ast: &mut MmlAst, chip_map: &HashMap<char, String>) {
        let forced_ym2612 = ast.metadata
            .get("ForcedMonoPartYM2612")
            .map_or(false, |v| v == "true");

        for (name, part) in &mut ast.parts {
            if part.chip.is_none() {
                if let Some(first_char) = name.chars().next() {
                    if let Some(chip) = chip_map.get(&first_char.to_ascii_uppercase()) {
                        part.chip = Some(chip.clone());
                    } else if forced_ym2612 {
                        part.chip = Some("YM2612".to_string());
                    }
                }
            }
        }
    }

    /// Tokenize source code
    fn lex(&self, source: &str) -> MmlResult<Vec<(crate::compiler::lexer::Token, crate::Position)>> {
        crate::compiler::lexer::tokenize(source)
    }

    /// Parse tokens into AST
    fn parse(&self, tokens: Vec<(crate::compiler::lexer::Token, crate::Position)>) -> MmlResult<MmlAst> {
        let parser = Parser::new(tokens);
        parser.parse()
    }

    /// Generate code for the specified output format
    fn generate_code(&self, ast: &MmlAst) -> MmlResult<Vec<u8>> {
        match self.options.format {
            OutputFormat::VGM => {
                let generator = crate::compiler::codegen::vgm::VgmGenerator::from_ast(ast, &self.options)?;
                generator.generate()
            }
            OutputFormat::XGM => {
                let generator = crate::compiler::codegen::xgm::XgmGenerator::from_ast(ast, &self.options)?;
                generator.generate()
            }
            OutputFormat::XGM2 => {
                let generator = crate::compiler::codegen::xgm::Xgm2Generator::from_ast(ast, &self.options)?;
                generator.generate()
            }
            OutputFormat::ZGM => {
                let generator = crate::compiler::codegen::zgm::ZgmGenerator::from_ast(ast, &self.options)?;
                generator.generate()
            }
        }
    }

    /// Compile MML source code directly from a string
    ///
    /// This is useful for WASM/browser integration where file system access
    /// is not available.
    pub fn compile_from_source(&self, source: &str) -> MmlResult<CompileResult> {
        let source = self.normalize_source(source);

        // Pre-process: extract '{...}' song info block before tokenisation
        let pre = self.preprocess_song_info(&source);

        // 1. Tokenize (Lexer)
        let tokens = self.lex(&pre.source)?;

        // 2. Parse (Parser)
        let mut ast = self.parse(tokens)?;

        // Inject metadata and chip assignments from the header block
        for (k, v) in pre.metadata {
            ast.metadata.entry(k).or_insert(v);
        }
        self.apply_chip_assignments(&mut ast, &pre.chip_map);

        // 3. Code Generation
        let part_count = ast.parts.len();
        let output_data = self.generate_code(&ast)?;
        let info = Self::info_from_vgm(&output_data, part_count);

        Ok(CompileResult {
            data: output_data,
            output_path: None,
            warnings: Vec::new(),
            info,
        })
    }

    /// Compile MML source code with pre-decoded PCM samples.
    ///
    /// Works like `compile_from_source` but passes the resolver to the VGM
    /// code generator so `'@ P` instruments can embed their PCM data blocks.
    pub fn compile_from_source_with_resolver(
        &self,
        source: &str,
        resolver: &dyn SampleResolver,
    ) -> MmlResult<CompileResult> {
        let source = self.normalize_source(source);
        let pre = self.preprocess_song_info(&source);
        let tokens = self.lex(&pre.source)?;
        let mut ast = self.parse(tokens)?;
        for (k, v) in pre.metadata {
            ast.metadata.entry(k).or_insert(v);
        }
        self.apply_chip_assignments(&mut ast, &pre.chip_map);
        let part_count = ast.parts.len();
        let output_data = self.generate_code_with_resolver(&ast, resolver)?;
        let info = Self::info_from_vgm(&output_data, part_count);
        Ok(CompileResult {
            data: output_data,
            output_path: None,
            warnings: Vec::new(),
            info,
        })
    }

    /// Generate code using a sample resolver (VGM only; other formats ignore samples).
    fn generate_code_with_resolver(
        &self,
        ast: &MmlAst,
        resolver: &dyn SampleResolver,
    ) -> MmlResult<Vec<u8>> {
        match self.options.format {
            OutputFormat::VGM => {
                let generator = crate::compiler::codegen::vgm::VgmGenerator::from_ast_with_resolver(
                    ast,
                    &self.options,
                    resolver,
                )?;
                generator.generate()
            }
            // XGM, XGM2, ZGM don't yet support embedded PCM samples; fall through.
            _ => self.generate_code(ast),
        }
    }

    /// Build CompileInfo from generated output data (VGM only; other formats return defaults).
    fn info_from_vgm(data: &[u8], part_count: usize) -> crate::CompileInfo {
        let mut info = crate::CompileInfo::default();
        info.part_count = part_count;
        // Extract total_samples from VGM header at offset 0x18
        if data.len() >= 0x1C && &data[0..4] == b"Vgm " {
            let total_samples =
                u32::from_le_bytes([data[0x18], data[0x19], data[0x1A], data[0x1B]]);
            info.duration_samples = total_samples as u64;
            info.duration_seconds = total_samples as f64 / 44100.0;
            // Count register-write commands (opcodes 0x50-0xDF range)
            let mut command_count: usize = 0;
            let data_offset = if data.len() > 0x40 {
                let raw = u32::from_le_bytes([data[0x34], data[0x35], data[0x36], data[0x37]]);
                if raw == 0 { 0x40 } else { (raw + 0x34) as usize }
            } else { 0x40 };
            let mut i = data_offset.min(data.len());
            while i < data.len() {
                let op = data[i];
                match op {
                    0x50 => { command_count += 1; i += 2; }
                    0x51..=0x5F => { command_count += 1; i += 3; }
                    0x61 => { i += 3; }
                    0x62 | 0x63 => { i += 1; }
                    0x66 => break,
                    0x67 => { if i + 6 < data.len() { let len = u32::from_le_bytes([data[i+2],data[i+3],data[i+4],data[i+5]]) as usize; i += 7 + len; } else { break; } }
                    0xA0 | 0xB0..=0xBF => { command_count += 1; i += 3; }
                    0xC0..=0xC4 | 0xD0..=0xD6 => { command_count += 1; i += 4; }
                    0xE0..=0xE1 => { command_count += 1; i += 4; }
                    _ => { i += 1; }
                }
            }
            info.command_count = command_count;
        }
        info
    }

    /// Validate MML source code from a string without generating output
    pub fn validate_from_source(&self, source: &str) -> MmlResult<()> {
        let source = self.normalize_source(source);
        let pre = self.preprocess_song_info(&source);
        let tokens = self.lex(&pre.source)?;
        let _ast = self.parse(tokens)?;
        Ok(())
    }

    /// Determine the output file path
    fn determine_output_path(&self, input_path: &Path) -> std::path::PathBuf {
        let output_ext = self.options.format.extension();

        let mut output_path = input_path.to_path_buf();
        output_path.set_extension(output_ext);

        output_path
    }
}

impl Default for MmlCompiler {
    fn default() -> Self {
        Self::new(CompileOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compiler_creation() {
        let compiler = MmlCompiler::new(CompileOptions::default());
        assert_eq!(compiler.options.format, OutputFormat::VGM);
    }

    #[test]
    fn test_compiler_defaults() {
        let compiler = MmlCompiler::default();
        assert_eq!(compiler.options.format, OutputFormat::VGM);
    }

    #[test]
    fn test_compiler_simple_mml() -> MmlResult<()> {
        // Create a simple MML file
        let mut file = NamedTempFile::new().map_err(|e| {
            MmlError::UnsupportedCommand(format!("Failed to create temp file: {}", e))
        })?;

        let mml_content = "{ Title=Test }\n'F o4 c4 d4 e4\n";
        file.write_all(mml_content.as_bytes()).map_err(|e| {
            MmlError::UnsupportedCommand(format!("Failed to write temp file: {}", e))
        })?;

        let compiler = MmlCompiler::new(CompileOptions::default());
        let result = compiler.compile(file.path())?;

        // Should produce some output
        assert!(!result.data.is_empty());

        Ok(())
    }

    #[test]
    fn test_compiler_validation() -> MmlResult<()> {
        // Create a valid MML file
        let mut file = NamedTempFile::new().map_err(|e| {
            MmlError::UnsupportedCommand(format!("Failed to create temp file: {}", e))
        })?;

        let mml_content = "{ Title=Test }\n'F o4 c4 d4\n";
        file.write_all(mml_content.as_bytes()).map_err(|e| {
            MmlError::UnsupportedCommand(format!("Failed to write temp file: {}", e))
        })?;

        let compiler = MmlCompiler::new(CompileOptions::default());
        assert!(compiler.validate(file.path()).is_ok());

        Ok(())
    }

    #[test]
    fn test_compiler_invalid_file() {
        let compiler = MmlCompiler::new(CompileOptions::default());
        let result = compiler.compile(Path::new("/nonexistent/file.gwi"));
        assert!(result.is_err());
    }

    #[test]
    fn test_compiler_output_path_uses_xgm2_extension() {
        let compiler = MmlCompiler::new(CompileOptions::new().with_output_format(OutputFormat::XGM2));
        let output_path = compiler.determine_output_path(Path::new("song.gwi"));
        assert_eq!(output_path, Path::new("song.xgm2"));
    }

    #[test]
    fn test_compiler_validation_semicolon_comments() -> MmlResult<()> {
        let compiler = MmlCompiler::new(CompileOptions::default());
        compiler.validate_from_source("; comment\n'Y01 v100 l4 o4\n'Y01 c d e f\n")?;
        Ok(())
    }
}
