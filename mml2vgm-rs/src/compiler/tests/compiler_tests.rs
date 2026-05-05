//! AST, codegen, and compiler integration tests
//!
//! Run with: cargo test compiler::tests::compiler_tests

use std::time::Instant;

use crate::compiler::ast::Note;
use crate::{CompileOptions, OutputFormat};
use crate::compiler::compiler::MmlCompiler;

// ── Note MIDI mapping ─────────────────────────────────────────────────────────

const AST_TIMEOUT_MS: u128 = 500;
const CODEGEN_TIMEOUT_SECS: u64 = 5;
const COMPILER_TIMEOUT_SECS: u64 = 10;

#[test]
fn ast_note_midi_c4() {
    let start = Instant::now();
    let note = Note::new('C', 0, 4);
    assert_eq!(note.midi_note(), 60);
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

#[test]
fn ast_note_midi_a4() {
    let start = Instant::now();
    assert_eq!(Note::new('A', 0, 4).midi_note(), 69);
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

#[test]
fn ast_note_midi_sharp() {
    let start = Instant::now();
    assert_eq!(Note::new('C', 1, 4).midi_note(), 61);
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

#[test]
fn ast_note_midi_flat() {
    let start = Instant::now();
    // D-flat = C#  at the same octave
    assert_eq!(Note::new('D', -1, 4).midi_note(), 61);
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

#[test]
fn ast_note_midi_clamp_low() {
    let start = Instant::now();
    // C at octave 0 with large negative accidental would underflow -> clamped to 0
    let note = Note::new('C', -10, 0);
    assert!(note.midi_note() <= 127);
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

#[test]
fn ast_note_midi_clamp_high() {
    let start = Instant::now();
    let note = Note::new('B', 10, 8);
    assert!(note.midi_note() <= 127);
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

#[test]
fn ast_all_chromatic_notes_octave4() {
    let start = Instant::now();
    // C4=60, C#4=61 … B4=71
    let expected: Vec<(char, i8, u8)> = vec![
        ('C', 0, 4), ('C', 1, 4), ('D', 0, 4), ('D', 1, 4),
        ('E', 0, 4), ('F', 0, 4), ('F', 1, 4), ('G', 0, 4),
        ('G', 1, 4), ('A', 0, 4), ('A', 1, 4), ('B', 0, 4),
    ];
    for (i, (letter, acc, oct)) in expected.iter().enumerate() {
        let note = Note::new(*letter, *acc, *oct);
        assert_eq!(note.midi_note() as usize, 60 + i, "chromatic note {} wrong", i);
    }
    assert!(start.elapsed().as_millis() < AST_TIMEOUT_MS);
}

// ── Compiler helpers ──────────────────────────────────────────────────────────

fn make_compiler(format: OutputFormat) -> MmlCompiler {
    MmlCompiler::new(CompileOptions {
        format,
        ..Default::default()
    })
}

const MINIMAL_MML: &str = "t120 o4 l4 c d e f";

// ── Compiler integration ──────────────────────────────────────────────────────

#[test]
fn compiler_compile_from_source_ok() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(MINIMAL_MML);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

#[test]
fn compiler_output_contains_data() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");
    assert!(!result.data.is_empty(), "expected non-empty output");
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

#[test]
fn compiler_validate_ok() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    assert!(compiler.validate_from_source(MINIMAL_MML).is_ok());
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

#[test]
fn compiler_warnings_empty_by_default() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");
    assert!(result.warnings.is_empty());
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

#[test]
fn compiler_compile_empty_source() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    // An empty source should either succeed (empty VGM) or return a graceful error.
    let result = compiler.compile_from_source("");
    let _ = result; // Just must not panic
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

// ── VGM magic bytes ───────────────────────────────────────────────────────────

#[test]
fn codegen_vgm_header_magic() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");
    assert!(
        result.data.len() >= 4,
        "VGM output too short to have magic bytes"
    );
    assert_eq!(
        &result.data[0..4],
        b"Vgm ",
        "VGM magic bytes mismatch"
    );
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

#[test]
fn codegen_vgm_nonzero_output() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");
    assert!(!result.data.is_empty());
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

// ── All output formats ────────────────────────────────────────────────────────

#[test]
fn codegen_all_formats_same_source() {
    let start = Instant::now();
    for format in [OutputFormat::VGM, OutputFormat::XGM, OutputFormat::XGM2, OutputFormat::ZGM] {
        let compiler = make_compiler(format);
        let result = compiler.compile_from_source(MINIMAL_MML);
        assert!(result.is_ok(), "format {:?} failed: {:?}", format, result.err());
    }
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

// ── Octave range ──────────────────────────────────────────────────────────────

#[test]
fn codegen_octave_range() {
    let start = Instant::now();
    let compiler = make_compiler(OutputFormat::VGM);
    for oct in 0..=8u8 {
        let src = format!("o{} c d e", oct);
        let result = compiler.compile_from_source(&src);
        assert!(result.is_ok(), "octave {} failed: {:?}", oct, result.err());
    }
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}

// ── Multiple parts ────────────────────────────────────────────────────────────

#[test]
fn codegen_multiple_parts() {
    let start = Instant::now();
    let src = "'A1 t120\n'A2 t120\n'A1 c d e f\n'A2 g a b r";
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(src).expect("compile failed");
    assert!(!result.data.is_empty());
    assert!(start.elapsed().as_secs() < COMPILER_TIMEOUT_SECS);
}
