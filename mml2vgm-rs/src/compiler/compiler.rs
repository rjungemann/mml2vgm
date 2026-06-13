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

        // 4. Parse (Parser) — pass the header metadata so flags that affect
        //    token interpretation (e.g. `Octave-Rev`) take effect.
        let mut ast = self.parse_with_metadata(tokens, &pre.metadata)?;

        // 5. Inject metadata and chip assignments from the header block
        for (k, v) in pre.metadata {
            ast.metadata.entry(k).or_insert(v);
        }
        self.apply_chip_assignments(&mut ast, &pre.chip_map);

        // 6. Code Generation
        let part_count = ast.parts.len();
        let (output_data, source_map) = self.generate_code_with_sourcemap(&ast)?;
        let info = Self::info_from_vgm(&output_data, part_count);

        // 7. Create result
        let output_path = self.determine_output_path(input_path);

        Ok(CompileResult {
            data: output_data,
            output_path: Some(output_path.to_string_lossy().to_string()),
            warnings: Vec::new(),
            info,
            source_map,
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

        // Lines consumed by the song-info block are replaced with this empty
        // sentinel rather than dropped from `out_lines`. That keeps the
        // 1-based line number of every surviving line identical to the
        // original source, so source-map note events point at the right
        // editor line (instead of being off by the header-block height).
        const STRIPPED: &str = "";

        for line in source.lines() {
            let trimmed = line.trim();

            if !in_block {
                let block_start = if trimmed.starts_with("'{") {
                    Some(trimmed[2..].trim())
                } else if trimmed == "{" {
                    Some("")
                } else {
                    None
                };

                if let Some(content) = block_start {
                    if content.ends_with('}') {
                        // Single-line block: '{ key = val }
                        let inner = content.trim_end_matches('}').trim();
                        Self::process_song_info_line(inner, &mut metadata, &mut chip_map);
                    } else {
                        in_block = true;
                        if !content.is_empty() {
                            Self::process_song_info_line(content, &mut metadata, &mut chip_map);
                        }
                    }
                    out_lines.push(STRIPPED);
                    continue;
                }
            }

            if in_block {
                if trimmed == "}" {
                    in_block = false;
                } else {
                    Self::process_song_info_line(trimmed, &mut metadata, &mut chip_map);
                }
                out_lines.push(STRIPPED);
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

    /// Parse with the song-info metadata available, so flags like
    /// `Octave-Rev` that affect token-stream interpretation can be applied
    /// during AST construction (notes bake their octave at parse time, so
    /// the flip has to happen here — codegen is too late).
    fn parse_with_metadata(
        &self,
        tokens: Vec<(crate::compiler::lexer::Token, crate::Position)>,
        metadata: &HashMap<String, String>,
    ) -> MmlResult<MmlAst> {
        let mut parser = Parser::new(tokens);
        let octave_reversed = metadata
            .get("Octave-Rev")
            .map(|v| v.trim().eq_ignore_ascii_case("TRUE"))
            .unwrap_or(false);
        parser.set_octave_reversed(octave_reversed);
        parser.parse()
    }

    /// Generate code for the specified output format
    fn generate_code(&self, ast: &MmlAst) -> MmlResult<Vec<u8>> {
        let (data, _source_map) = self.generate_code_with_sourcemap(ast)?;
        Ok(data)
    }

    /// Generate code and extract both the binary output and source map
    fn generate_code_with_sourcemap(&self, ast: &MmlAst) -> MmlResult<(Vec<u8>, crate::compiler::codegen::SourceMap)> {
        match self.options.format {
            OutputFormat::VGM => {
                let generator = crate::compiler::codegen::vgm::VgmGenerator::from_ast(ast, &self.options)?;
                let data = generator.generate()?;
                let source_map = generator.source_map().clone();
                Ok((data, source_map))
            }
            OutputFormat::XGM => {
                let generator = crate::compiler::codegen::xgm::XgmGenerator::from_ast(ast, &self.options)?;
                let data = generator.generate()?;
                Ok((data, crate::compiler::codegen::SourceMap::default()))
            }
            OutputFormat::XGM2 => {
                let generator = crate::compiler::codegen::xgm::Xgm2Generator::from_ast(ast, &self.options)?;
                let data = generator.generate()?;
                Ok((data, crate::compiler::codegen::SourceMap::default()))
            }
            OutputFormat::ZGM => {
                let generator = crate::compiler::codegen::zgm::ZgmGenerator::from_ast(ast, &self.options)?;
                let data = generator.generate()?;
                Ok((data, crate::compiler::codegen::SourceMap::default()))
            }
            OutputFormat::MID => {
                let generator = crate::compiler::codegen::midi::MidiGenerator::from_ast(ast, &self.options)?;
                let data = generator.generate()?;
                let source_map = generator.source_map().clone();
                Ok((data, source_map))
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

        // 2. Parse (Parser) — see compile() for why metadata threads here.
        let mut ast = self.parse_with_metadata(tokens, &pre.metadata)?;

        // Inject metadata and chip assignments from the header block
        for (k, v) in pre.metadata {
            ast.metadata.entry(k).or_insert(v);
        }
        self.apply_chip_assignments(&mut ast, &pre.chip_map);

        // 3. Code Generation
        let part_count = ast.parts.len();
        let (output_data, source_map) = self.generate_code_with_sourcemap(&ast)?;
        let info = Self::info_from_vgm(&output_data, part_count);

        Ok(CompileResult {
            data: output_data,
            output_path: None,
            warnings: Vec::new(),
            info,
            source_map,
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
        let mut ast = self.parse_with_metadata(tokens, &pre.metadata)?;
        for (k, v) in pre.metadata {
            ast.metadata.entry(k).or_insert(v);
        }
        self.apply_chip_assignments(&mut ast, &pre.chip_map);
        let part_count = ast.parts.len();
        let (output_data, source_map) = self.generate_code_with_resolver_and_sourcemap(&ast, resolver)?;
        let info = Self::info_from_vgm(&output_data, part_count);
        Ok(CompileResult {
            data: output_data,
            output_path: None,
            warnings: Vec::new(),
            info,
            source_map,
        })
    }

    /// Generate code using a sample resolver (VGM only; other formats ignore samples).
    fn generate_code_with_resolver(
        &self,
        ast: &MmlAst,
        resolver: &dyn SampleResolver,
    ) -> MmlResult<Vec<u8>> {
        let (data, _source_map) = self.generate_code_with_resolver_and_sourcemap(ast, resolver)?;
        Ok(data)
    }

    /// Generate code with resolver and extract both output and source map.
    fn generate_code_with_resolver_and_sourcemap(
        &self,
        ast: &MmlAst,
        resolver: &dyn SampleResolver,
    ) -> MmlResult<(Vec<u8>, crate::compiler::codegen::SourceMap)> {
        match self.options.format {
            OutputFormat::VGM => {
                let generator = crate::compiler::codegen::vgm::VgmGenerator::from_ast_with_resolver(
                    ast,
                    &self.options,
                    resolver,
                )?;
                let data = generator.generate()?;
                let source_map = generator.source_map().clone();
                Ok((data, source_map))
            }
            // XGM, XGM2, ZGM don't yet support embedded PCM samples; fall through.
            _ => self.generate_code_with_sourcemap(ast),
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
            info.chips_used = Self::chips_from_vgm_header(data);
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

    /// Inspect a VGM byte stream's header and return the chips it declares.
    /// Each chip occupies one little-endian u32 clock field at a fixed
    /// offset; a nonzero value (top bit masked, since VGM uses it as the
    /// dual-chip flag) marks the chip as present. The browser side runs the
    /// same scan in `detectChipsFromVgmHeader`; the table here is gated by
    /// the header's declared version so we don't misread reserved bytes on
    /// older VGM streams.
    fn chips_from_vgm_header(data: &[u8]) -> Vec<crate::SoundChip> {
        use crate::SoundChip;
        if data.len() < 0x40 { return Vec::new(); }
        let read_u32 = |off: usize| -> u32 {
            if off + 4 > data.len() { return 0; }
            u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
        };
        let version = read_u32(0x08);

        // (offset, min_version, chip)
        const ENTRIES: &[(usize, u32, SoundChip)] = &[
            (0x0C, 0x100, SoundChip::SN76489),
            (0x10, 0x100, SoundChip::YM2413),
            (0x2C, 0x110, SoundChip::YM2612),
            (0x30, 0x110, SoundChip::YM2151),
            (0x38, 0x151, SoundChip::SegaPCM),
            (0x40, 0x151, SoundChip::RF5C164),
            (0x44, 0x151, SoundChip::YM2203),
            (0x48, 0x151, SoundChip::YM2608),
            (0x4C, 0x151, SoundChip::YM2610B),
            (0x50, 0x151, SoundChip::YM3812),
            (0x54, 0x151, SoundChip::YM3526),
            (0x58, 0x151, SoundChip::Y8950),
            (0x5C, 0x151, SoundChip::YMF262),
            (0x6C, 0x171, SoundChip::YMF271),
            (0x74, 0x161, SoundChip::AY8910),
            (0x80, 0x161, SoundChip::DMG),
            (0x84, 0x161, SoundChip::NES),
            (0x9C, 0x161, SoundChip::K054539),
            (0xA0, 0x161, SoundChip::HuC6280),
            (0xA4, 0x161, SoundChip::C140),
            (0xAC, 0x161, SoundChip::K053260),
            (0xB0, 0x161, SoundChip::POKEY),
            (0xB4, 0x161, SoundChip::QSound),
            (0xC4, 0x171, SoundChip::C352),
            (0xE0, 0x171, SoundChip::VRC6),
        ];

        let mut found = Vec::new();
        for &(offset, min_version, chip) in ENTRIES {
            if version < min_version { continue; }
            let clock = read_u32(offset) & 0x7fff_ffff; // mask dual-chip flag
            if clock != 0 { found.push(chip); }
        }
        found
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
