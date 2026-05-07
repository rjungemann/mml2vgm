use crate::document::CompileError;
use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::{CompileOptions, MmlError, OutputFormat};
use std::path::Path;
use std::str::FromStr;

pub struct CompileOutput {
    pub bytes: Vec<u8>,
    pub warnings: usize,
}

/// Compile `path` with the given format string. Returns structured errors on failure.
pub fn compile(path: &Path, format: &str) -> Result<CompileOutput, Vec<CompileError>> {
    let fmt = OutputFormat::from_str(format).unwrap_or(OutputFormat::VGM);
    let opts = CompileOptions::default().with_output_format(fmt);
    let compiler = MmlCompiler::new(opts);

    compiler.compile(path).map(|r| CompileOutput {
        bytes: r.data,
        warnings: r.warnings.len(),
    }).map_err(extract_errors)
}

/// Compile MML source string (no file required). Used by the socket interface.
pub fn compile_content(content: &str, format: &str) -> Result<CompileOutput, Vec<CompileError>> {
    let fmt = OutputFormat::from_str(format).unwrap_or(OutputFormat::VGM);
    let opts = CompileOptions::default().with_output_format(fmt);
    let compiler = MmlCompiler::new(opts);

    compiler.compile_from_source(content).map(|r| CompileOutput {
        bytes: r.data,
        warnings: r.warnings.len(),
    }).map_err(extract_errors)
}

fn extract_errors(e: MmlError) -> Vec<CompileError> {
    match e {
        MmlError::Parse { line, column, message } => vec![CompileError {
            line: Some(line),
            col: Some(column),
            message,
        }],
        other => vec![CompileError {
            line: None,
            col: None,
            message: other.to_string(),
        }],
    }
}
