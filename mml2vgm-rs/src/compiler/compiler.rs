//! MML Compiler
//!
//! Orchestrates the full compilation pipeline from MML source to VGM/XGM/ZGM output.
//! Handles tokenization, parsing, semantic analysis, and code generation.

use crate::compiler::ast::MmlAst;
use crate::compiler::codegen::CodeGenerator;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::sema::Sema;
use crate::{CompileOptions, CompileResult, MmlError, MmlResult, OutputFormat};
use std::fs;
use std::path::Path;

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
        // 1. Read source file
        let source = self.read_file(input_path)?;

        // 2. Tokenize (Lexer)
        let tokens = self.lex(&source)?;

        // 3. Parse (Parser)
        let ast = self.parse(tokens)?;

        // 4. Semantic Analysis (currently unimplemented, so skip for now)
        // let mut sema = Sema::new();
        // sema.analyze(&mut ast)?;

        // 5. Code Generation
        let output_data = self.generate_code(&ast)?;

        // 6. Create result
        let output_path = self.determine_output_path(input_path);

        Ok(CompileResult {
            data: output_data,
            output_path: Some(output_path.to_string_lossy().to_string()),
            warnings: Vec::new(),
            info: crate::CompileInfo::default(),
        })
    }

    /// Validate an MML file without generating output
    pub fn validate(&self, input_path: &Path) -> MmlResult<()> {
        // 1. Read source file
        let source = self.read_file(input_path)?;

        // 2. Tokenize
        let tokens = self.lex(&source)?;

        // 3. Parse
        let _ast = self.parse(tokens)?;

        Ok(())
    }

    /// Read input file
    fn read_file(&self, path: &Path) -> MmlResult<String> {
        fs::read_to_string(path).map_err(|e| {
            MmlError::UnsupportedCommand(format!("Failed to read file {}: {}", path.display(), e))
        })
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
        // 1. Tokenize (Lexer)
        let tokens = self.lex(source)?;

        // 2. Parse (Parser)
        let ast = self.parse(tokens)?;

        // 3. Semantic Analysis (currently unimplemented, so skip for now)
        // let mut sema = Sema::new();
        // sema.analyze(&mut ast)?;

        // 4. Code Generation
        let output_data = self.generate_code(&ast)?;

        Ok(CompileResult {
            data: output_data,
            output_path: None,
            warnings: Vec::new(),
            info: crate::CompileInfo::default(),
        })
    }

    /// Validate MML source code from a string without generating output
    pub fn validate_from_source(&self, source: &str) -> MmlResult<()> {
        // 1. Tokenize
        let tokens = self.lex(source)?;

        // 2. Parse
        let _ast = self.parse(tokens)?;

        Ok(())
    }

    /// Determine the output file path
    fn determine_output_path(&self, input_path: &Path) -> std::path::PathBuf {
        let output_ext = match self.options.format {
            OutputFormat::VGM => "vgm",
            OutputFormat::XGM => "xgm",
            OutputFormat::XGM2 => "xgm",
            OutputFormat::ZGM => "zgm",
        };

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
}
