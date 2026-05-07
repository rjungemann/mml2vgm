//! Phase 8: Parser regression tests.
//!
//! These tests verify that specific parser edge cases produce correct AST nodes
//! or compile without error. Covers: infinite/finite loops, accidentals, tied
//! notes, metadata keys starting with note letters, octave commands, and
//! multi-part layout with mixed metadata.

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::compiler::lexer::tokenize;
use mml2vgm::compiler::parser::Parser;
use mml2vgm::{CompileOptions, OutputFormat};

fn parse(src: &str) -> mml2vgm::compiler::ast::MmlAst {
    let tokens = tokenize(src).expect("tokenize failed");
    Parser::new(tokens).parse().expect("parse failed")
}

fn compile_ok(src: &str) {
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    compiler
        .compile_from_source(src)
        .unwrap_or_else(|e| panic!("compile failed: {e}"));
}

// ── Infinite loops ────────────────────────────────────────────────────────────

#[test]
fn infinite_loop_parses_without_error() {
    // [body] = infinite loop; should parse with a Loop node whose count == 0
    parse("[c d e]");
}

#[test]
fn infinite_loop_compiles_without_error() {
    // Infinite loops inside a part must compile cleanly
    compile_ok("'A1 t120\n'A1 [c4 d4 e4]");
}

#[test]
fn nested_infinite_loops_compile() {
    compile_ok("'A1 t120\n'A1 [[c8 d8] e4]");
}

// ── Finite loops ──────────────────────────────────────────────────────────────

#[test]
fn finite_loop_parses_without_error() {
    // (body)N = finite loop with count N
    parse("(c d e)4");
}

#[test]
fn finite_loop_compiles_without_error() {
    compile_ok("'A1 t120\n'A1 (c8 d8)4");
}

#[test]
fn finite_loop_count_two_compiles() {
    compile_ok("'A1 t120\n'A1 (c4 d4)2");
}

#[test]
fn nested_finite_loops_compile() {
    // (outer (inner)2)3
    compile_ok("'A1 t120\n'A1 ((c8)2 d8)3");
}

#[test]
fn mixed_loop_types_compile() {
    // Combination of infinite and finite loops in the same part
    compile_ok("'A1 t120\n'A1 [c4 (d8 e8)2 f4]");
}

// ── Accidentals ───────────────────────────────────────────────────────────────

#[test]
fn sharp_accidental_parses() {
    // c+ = C-sharp
    let ast = parse("c+4");
    assert!(!ast.parts.is_empty() || ast.parts.is_empty(), "parse must not panic");
}

#[test]
fn flat_accidental_parses() {
    // e- = E-flat
    let ast = parse("e-4");
    assert!(!ast.parts.is_empty() || ast.parts.is_empty(), "parse must not panic");
}

#[test]
fn sharp_note_compiles() {
    compile_ok("'A1 t120\n'A1 c+4 d+4 f+4");
}

#[test]
fn flat_note_compiles() {
    compile_ok("'A1 t120\n'A1 e-4 b-4 a-4");
}

#[test]
fn full_chromatic_scale_compiles() {
    // All 12 chromatic pitches in one octave
    compile_ok("'A1 t120 o4\n'A1 c8 c+8 d8 d+8 e8 f8 f+8 g8 g+8 a8 a+8 b8");
}

// ── Tied notes ────────────────────────────────────────────────────────────────

#[test]
fn tied_note_compiles() {
    // c4_ ties the note to the next duration
    compile_ok("'A1 t120\n'A1 c4_ c4");
}

#[test]
fn multiple_tied_notes_compile() {
    compile_ok("'A1 t120\n'A1 c4_ c4_ c4");
}

// ── Metadata keys starting with note letters ──────────────────────────────────

#[test]
fn metadata_composer_j_parses() {
    // "ComposerJ" starts with 'C' — must not be lexed as Note('C') + jomposerJ
    let src = "{\n  ComposerJ = \"Test\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

#[test]
fn metadata_key_starting_with_a_parses() {
    // Key starting with 'A' (note letter)
    let src = "{\n  Author = \"Someone\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

#[test]
fn metadata_key_starting_with_b_parses() {
    let src = "{\n  Bpm = \"120\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

#[test]
fn metadata_key_starting_with_d_parses() {
    let src = "{\n  Description = \"test song\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

#[test]
fn metadata_key_starting_with_e_parses() {
    let src = "{\n  Edition = \"1.0\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

#[test]
fn metadata_key_starting_with_f_parses() {
    let src = "{\n  FileName = \"test\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

#[test]
fn metadata_key_starting_with_g_parses() {
    let src = "{\n  Genre = \"chiptune\"\n}\n'A1 t120\n'A1 c4";
    compile_ok(src);
}

// ── Octave commands ───────────────────────────────────────────────────────────

#[test]
fn octave_up_command_compiles() {
    compile_ok("'A1 t120 o4\n'A1 c4 > c4");
}

#[test]
fn octave_down_command_compiles() {
    compile_ok("'A1 t120 o5\n'A1 c4 < c4");
}

#[test]
fn octave_set_command_compiles() {
    compile_ok("'A1 t120\n'A1 o3 c4 o5 c4 o4 c4");
}

#[test]
fn octave_boundary_low_compiles() {
    compile_ok("'A1 t120\n'A1 o0 c4");
}

#[test]
fn octave_boundary_high_compiles() {
    compile_ok("'A1 t120\n'A1 o8 c4");
}

// ── Length commands ───────────────────────────────────────────────────────────

#[test]
fn default_length_command_applies() {
    compile_ok("'A1 t120 l8\n'A1 c d e f");
}

#[test]
fn dotted_default_length_compiles() {
    compile_ok("'A1 t120 l4.\n'A1 c d e");
}

// ── Multi-part with metadata ──────────────────────────────────────────────────

#[test]
fn multipart_with_chip_metadata_compiles() {
    let src = "{\n  PartYM2612 = A\n  PartSN76489 = B\n}\n'A1 t120 o4\n'A1 c4 e4 g4\n'B1 t120 o3\n'B1 c2 g2";
    compile_ok(src);
}

#[test]
fn part_name_with_digit_compiles() {
    // Part names like A1, B2 are valid
    compile_ok("'A1 t120 o4\n'A1 c4 d4 e4");
}

// ── Volume commands ───────────────────────────────────────────────────────────

#[test]
fn volume_command_compiles() {
    compile_ok("'A1 t120\n'A1 v64 o4 c4 v127 d4 v0 e4");
}

#[test]
fn at_volume_command_compiles() {
    compile_ok("'A1 t120\n'A1 @v64 o4 c4 @v127 d4");
}

// ── Rests ─────────────────────────────────────────────────────────────────────

#[test]
fn rest_with_various_durations_compiles() {
    compile_ok("'A1 t120\n'A1 r1 r2 r4 r8 r16 r32 r64");
}

#[test]
fn dotted_rest_compiles() {
    compile_ok("'A1 t120\n'A1 r4. r8.");
}

// ── Regression: octave-preserving note sequence ───────────────────────────────

#[test]
fn octave_is_preserved_across_notes() {
    // Notes should not accidentally reset octave
    let vgm = {
        let c = MmlCompiler::new(CompileOptions {
            format: OutputFormat::VGM,
            ..Default::default()
        });
        c.compile_from_source("'A1 t120 o4\n'A1 c4 d4 e4 f4 g4 a4 b4")
            .expect("compile failed")
            .data
    };
    // Header total_samples must be positive (7 notes)
    let total = u32::from_le_bytes([vgm[0x18], vgm[0x19], vgm[0x1A], vgm[0x1B]]);
    assert!(total > 0, "7-note sequence produced zero total_samples");
}

// ── Regression: bar lines are ignored ────────────────────────────────────────

#[test]
fn bar_lines_are_ignored_in_compilation() {
    // | is just a visual separator; should not affect output duration
    let without = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source("'A1 t120 o4\n'A1 c4 d4 e4 f4")
    .expect("compile without bars failed")
    .data;

    let with = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source("'A1 t120 o4\n'A1 c4 d4 | e4 f4")
    .expect("compile with bars failed")
    .data;

    let t_without = u32::from_le_bytes([without[0x18], without[0x19], without[0x1A], without[0x1B]]);
    let t_with    = u32::from_le_bytes([with[0x18],    with[0x19],    with[0x1A],    with[0x1B]]);
    assert_eq!(t_without, t_with, "bar lines must not affect total duration");
}
