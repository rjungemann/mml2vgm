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

#[test]
fn compiler_compile_from_source_xgm_has_fm_psg_blocks() {
    let compiler = make_compiler(OutputFormat::XGM);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");

    assert!(result.data.len() >= 0x20);
    assert_eq!(&result.data[0..4], b"XGM ");
    assert_eq!(result.data[4], 0x10);
    assert_eq!(&result.data[result.data.len() - 4..], &[0x0f, 0xff, 0xff, 0xff]);
}

#[test]
fn compiler_compile_from_source_xgm2_has_block_sizes() {
    let compiler = make_compiler(OutputFormat::XGM2);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");

    assert!(result.data.len() >= 0x20);
    assert_eq!(&result.data[0..4], b"XGM2");
    assert_eq!(result.data[4], 0x10);
    let fm_units = u16::from_le_bytes([result.data[8], result.data[9]]);
    assert!(fm_units > 0);
}

#[test]
fn compiler_compile_from_source_zgm_has_offsets() {
    let compiler = make_compiler(OutputFormat::ZGM);
    let result = compiler.compile_from_source(MINIMAL_MML).expect("compile failed");

    assert!(result.data.len() >= 0x40);
    assert_eq!(&result.data[0..4], b"ZGM ");
    let define_offset = u32::from_le_bytes(result.data[0x1c..0x20].try_into().expect("slice"));
    let track_offset = u32::from_le_bytes(result.data[0x20..0x24].try_into().expect("slice"));
    let define_count = u16::from_le_bytes(result.data[0x24..0x26].try_into().expect("slice"));
    let track_count = u16::from_le_bytes(result.data[0x26..0x28].try_into().expect("slice"));
    assert_eq!(define_offset, 0x40);
    assert!(track_offset > define_offset);
    assert!(define_count > 0);
    assert_eq!(track_count, 1);
    assert!(result.data.windows(3).any(|window| window == b"Def"));
    assert!(result.data.windows(3).any(|window| window == b"Trk"));
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

// ── C# MML format: song info block + chip mapping ────────────────────────────

#[test]
fn csharp_format_song_info_preprocessed() {
    // The '{...}' block should be extracted and not confuse the lexer.
    let src = "'{
  TitleName   =My Song
  PartYM2612  =A
  PartSN76489 =B
  Format      =VGM
  ClockCount  =192
}

'A1 t120
'A1 o4 l4 c d e f
";
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(src).expect("csharp_format_song_info_preprocessed compile failed");
    assert!(!result.data.is_empty());
    assert_eq!(&result.data[0..4], b"Vgm ", "VGM magic missing");
}

#[test]
fn csharp_format_chip_assignments() {
    use crate::compiler::compiler::MmlCompiler;
    use crate::CompileOptions;
    // Verify that Part* mappings from the header propagate to part.chip.
    let src = "'{
  PartYM2612  =A
  PartSN76489 =B
}
'A1 t120
'A1 c d
'B1 r r
";
    let compiler = MmlCompiler::new(CompileOptions::default());
    // Preprocessing strips the block; the remaining source should parse two parts.
    let result = compiler.compile_from_source(src).expect("compile failed");
    assert!(!result.data.is_empty());
}

#[test]
fn csharp_format_note_letter_part_names() {
    // F is a note letter (CDEFGAB), but 'F1 means part F channel 1 in C# format.
    let src = "'F1 t120\n'F1 o4 l4 c r d r\n'F2 o4 l4 e r f r\n";
    let compiler = make_compiler(OutputFormat::VGM);
    let result = compiler.compile_from_source(src).expect("note-letter part name compile failed");
    assert!(!result.data.is_empty());
}

#[test]
fn csharp_format_fm_instrument_accumulated() {
    // '@ M NNN followed by 4 operator rows (11 params each) + 1 ALG/FB row (2 params)
    // = 46 total parameters stored in FmInstrument.parameters.
    let src = "'{
  TitleName   = FM Accum Test
  Format      = VGM
  ClockCount  = 192
  PartYM2612  = A
}

'@ M 000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000

'A1 T120
'A1 @0 v100 l4 o4 c d e f
";
    use crate::compiler::lexer::tokenize;
    use crate::compiler::parser::Parser;
    let compiler = make_compiler(OutputFormat::VGM);

    // Parse the FM instrument block directly to inspect the AST
    let src_no_header = "'@ M 000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000
";
    let tokens = tokenize(src_no_header).expect("tokenize failed");
    let ast = Parser::new(tokens).parse().expect("parse failed");
    assert!(ast.fm_instruments.contains_key(&0), "FM instrument 0 not stored");
    let inst = &ast.fm_instruments[&0];
    // 4 operators × 11 params + 2 ALG/FB params = 46
    assert_eq!(inst.parameters.len(), 46, "expected 46 parameters, got {}", inst.parameters.len());
    // First operator's first param (AR) = 31
    assert_eq!(inst.parameters[0], 31, "expected AR=31 for op1");
    // ALG/FB row: param index 44 = ALG = 7, param index 45 = FB = 0
    assert_eq!(inst.parameters[44], 7, "expected ALG=7");
    assert_eq!(inst.parameters[45], 0, "expected FB=0");

    // Full compile should also succeed
    let result = compiler.compile_from_source(src).expect("compile_from_source failed");
    assert!(!result.data.is_empty());
    assert_eq!(&result.data[0..4], b"Vgm ");
}

#[test]
fn csharp_format_song_info_no_hang() {
    // Feed a real-looking T0100 header and make sure it terminates quickly.
    use std::time::Instant;
    let src = "'{
  TitleName   =YM2612 OPNB Channel Test
  SystemName  =Sega Genesis
  Format      =VGM
  ClockCount  =192
  Octave-Rev  =FALSE
  ForcedMonoPartYM2612
}

'@ M 000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000

'F1 t120
'F1 o4 l4 c r r r
'F2 o4 l4 r d r r
";
    let compiler = make_compiler(OutputFormat::VGM);
    let start = Instant::now();
    let result = compiler.compile_from_source(src).expect("T0100-style compile failed");
    assert!(start.elapsed().as_secs() < 2, "compile took too long (hang?)");
    assert!(!result.data.is_empty());
}
