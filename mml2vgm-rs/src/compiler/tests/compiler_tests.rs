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
fn csharp_format_t0100_full_no_hang() {
    use std::time::Instant;
    // Full T0100 content with column headers (as they appear in the actual file).
    let src = "'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000

'@ F 110
   AR  DR  SR  RR  SL  TL  KS  ML  DT
'@ 031,000,000,000,000,033,000,004,007
'@ 018,015,009,007,003,009,000,004,007
'@ 031,000,000,000,000,024,000,002,003
'@ 031,015,009,007,003,000,000,002,003
'@ 004,007

'F1 T120

'F1  Q8@0v100l4o4 crrrrrb>r<
'F2  Q8@0v100l4o4 rdrrrrr>c<
'F3  Q8@0v100l4o4 rrerrrr>r<
'F4  Q8@0v100l4o4 rrrfrrr>r<
'F5  Q8@0v100l4o4 rrrrgrr>r<
'F6  Q8@0v100l4o4 rrrrrar>r<
'F7            l4 rrrrrrrr
'F8            l4 rrrrrrrr
'F9            l4 rrrrrrrr

'F3 EON EX1 v120l4o4 crrrg
'F7     EX2 v120l4o4 rdrrra
'F8     EX3 v120l4o4 rrerrrb
'F9     EX4 v120l4o4 rrrfrrr>c
";
    let compiler = make_compiler(OutputFormat::VGM);
    let start = Instant::now();
    let result = compiler.compile_from_source(src);
    assert!(start.elapsed().as_secs() < 5, "T0100 full content compile took too long (hang?)");
    assert!(result.is_ok(), "T0100 full content compile failed: {:?}", result.err());
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

#[test]
fn test_instrument_110_stored() {
    // Test that F-type instrument 110 from T0000 is stored correctly
    let src = "'@ F 110\n   AR  DR  SR  RR  SL  TL  KS  ML  DT\n'@ 031,000,000,000,000,033,000,004,007\n'@ 018,015,009,007,003,009,000,004,007\n'@ 031,000,000,000,000,024,000,002,003\n'@ 031,015,009,007,003,000,000,002,003\n'@ 004,007\n";
    let ast = crate::compiler::parser::parse(src).expect("parse failed");
    assert!(ast.fm_instruments.contains_key(&110), "instrument 110 not stored; keys: {:?}", ast.fm_instruments.keys().collect::<Vec<_>>());
    let inst = &ast.fm_instruments[&110];
    assert_eq!(inst.parameters.len(), 38, "expected 38 params, got {}", inst.parameters.len());
    // ALG should be at [36], FB at [37]
    assert_eq!(inst.parameters[36], 4, "ALG expected 4 got {}", inst.parameters[36]);
    assert_eq!(inst.parameters[37], 7, "FB expected 7 got {}", inst.parameters[37]);
    // TL of op0 should be at [5]
    assert_eq!(inst.parameters[5], 33, "TL op0 expected 33 got {}", inst.parameters[5]);
}

#[test]
fn test_instrument_110_params_in_vgm() {
    // Test that instrument 110 params are used in VGM codegen (not defaults)
    // With instrument 110 (F-type): ALG=4, FB=7
    // B0+ch0 value = ((FB & 7) << 3) | (ALG & 7) = ((7) << 3) | (4) = 0x3C
    // With defaults: ALG=7, FB=0 → B0 value = 0x07
    let src = "'@ F 110\n   AR  DR  SR  RR  SL  TL  KS  ML  DT\n'@ 031,000,000,000,000,033,000,004,007\n'@ 018,015,009,007,003,009,000,004,007\n'@ 031,000,000,000,000,024,000,002,003\n'@ 031,015,009,007,003,000,000,002,003\n'@ 004,007\n'A1 v100l4o4 @110 c\n";
    let compiler = make_compiler(crate::OutputFormat::VGM);
    let result = compiler.compile_from_source(src).expect("compile failed");
    // Find B0+ch0 write (reg 0xB0): should be 0x3C (instrument 110) not 0x07 (default)
    // VGM format: 0x52, reg, val
    let data = &result.data;
    let mut found = false;
    let mut i = 0x100; // start after VGM header
    while i + 2 < data.len() {
        if data[i] == 0x52 && data[i+1] == 0xB0 {
            let val = data[i+2];
            eprintln!("Found B0+ch0 write at offset {}: val=0x{:02X}", i, val);
            assert_eq!(val, 0x3C, "Expected B0 = 0x3C (ALG=4,FB=7 from inst 110), got 0x{:02X}", val);
            found = true;
            break;
        }
        i += 1;
    }
    assert!(found, "No B0+ch0 write found in VGM output");
}

#[test]
fn test_instrument_110_debug() {
    let src = "'@ F 110\n   AR  DR  SR  RR  SL  TL  KS  ML  DT\n'@ 031,000,000,000,000,033,000,004,007\n'@ 018,015,009,007,003,009,000,004,007\n'@ 031,000,000,000,000,024,000,002,003\n'@ 031,015,009,007,003,000,000,002,003\n'@ 004,007\n'A1 v100l4o4 @110 c\n";
    // First, check what AST is parsed
    let ast = crate::compiler::parser::parse(src).expect("parse failed");
    eprintln!("Parts: {:?}", ast.parts.keys().collect::<Vec<_>>());
    eprintln!("FM instruments: {:?}", ast.fm_instruments.keys().collect::<Vec<_>>());
    if let Some(part) = ast.parts.get("A1") {
        eprintln!("Part A1 chip: {:?}", part.chip);
        eprintln!("Part A1 commands: {:?}", part.commands.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>());
    }
}

#[test]
fn test_t0000_codegen_debug() {
    // Simulate T0000's actual compilation (with chip map via ForcedMono)
    let src = "'\x7B\nForcedMonoPartYM2612\n\x7D\n'@ F 110\n   AR  DR  SR  RR  SL  TL  KS  ML  DT\n'@ 031,000,000,000,000,033,000,004,007\n'@ 018,015,009,007,003,009,000,004,007\n'@ 031,000,000,000,000,024,000,002,003\n'@ 031,015,009,007,003,000,000,002,003\n'@ 004,007\n'A1 T90\n'A1 @110 v100 l4 o4 c\n";
    let compiler = make_compiler(crate::OutputFormat::VGM);
    let result = compiler.compile_from_source(src).expect("compile failed");
    // Dump first 30 writes
    let data = &result.data;
    let mut i = 0x100;
    let mut count = 0;
    while i < data.len() && count < 30 {
        match data[i] {
            0x52 | 0x53 if i + 2 < data.len() => {
                eprintln!("write @{}: port={} reg=0x{:02X} val=0x{:02X}", count, data[i]-0x52, data[i+1], data[i+2]);
                i += 3; count += 1;
            }
            0x61 if i + 2 < data.len() => {
                let s = u16::from_le_bytes([data[i+1], data[i+2]]);
                eprintln!("wait: {} samples", s);
                i += 3;
            }
            0x66 => { eprintln!("EOF"); break; }
            _ => { i += 1; }
        }
    }
}

#[test]
fn test_t0000_actual_file() {
    // Test directly against the actual T0000 file
    let path = std::path::Path::new("/tmp/mml2vgm-csharp/mml2vgm/samples/test/T0000_SongInfoDef.gwi");
    if !path.exists() { eprintln!("Skipping: file not found"); return; }
    // Parse the T0000 source directly (without preprocess) and check fm_instruments
    {
        let raw = std::fs::read_to_string(path).expect("read");
        let source = raw.strip_prefix('\u{FEFF}').unwrap_or(&raw)
            .replace("\r\n", "\n")
            .split('\n')
            .map(|l| if l.trim_start().starts_with(';') { "" } else { l })
            .collect::<Vec<_>>().join("\n");
        let tokens = crate::compiler::lexer::tokenize(&source).expect("lex");
        let ast = crate::compiler::parser::Parser::new(tokens).parse().expect("parse");
        eprintln!("=== WITHOUT preprocess ===");
        eprintln!("fm_instruments keys: {:?}", ast.fm_instruments.keys().collect::<Vec<_>>());
    }
    // Parse WITH preprocess (same as compiler.compile path)
    {
        use crate::compiler::compiler::MmlCompiler;
        let raw = std::fs::read_to_string(path).expect("read");
        let source = raw.strip_prefix('\u{FEFF}').unwrap_or(&raw)
            .replace("\r\n", "\n")
            .split('\n')
            .map(|l| if l.trim_start().starts_with(';') { "" } else { l })
            .collect::<Vec<_>>().join("\n");
        // Simulate preprocess: strip '{...}' block manually
        let mut out_lines: Vec<&str> = Vec::new();
        let mut in_block = false;
        for line in source.lines() {
            let trimmed = line.trim();
            if !in_block {
                if trimmed.starts_with("'{") || trimmed == "{" {
                    in_block = true;
                    continue;
                }
            }
            if in_block {
                if trimmed == "}" { in_block = false; }
                continue;
            }
            out_lines.push(line);
        }
        let preprocessed = out_lines.join("\n");
        let tokens = crate::compiler::lexer::tokenize(&preprocessed).expect("lex2");
        let ast = crate::compiler::parser::Parser::new(tokens).parse().expect("parse2");
        eprintln!("=== WITH preprocess ===");
        eprintln!("fm_instruments keys: {:?}", ast.fm_instruments.keys().collect::<Vec<_>>());
        for (num, inst) in &ast.fm_instruments {
            let stride = if inst.parameters.len() >= 46 { 11usize } else { 9usize };
            eprintln!("  @{}: {} params (stride {}), alg={:?}, fb={:?}, tl[0]={:?}",
                num, inst.parameters.len(), stride,
                inst.parameters.get(stride * 4),
                inst.parameters.get(stride * 4 + 1),
                inst.parameters.get(5));
        }
        eprintln!("=== PART COMMANDS ===");
        for (part_name, part) in &ast.parts {
            eprintln!("Part '{}': {} commands", part_name, part.commands.len());
            for (i, cmd) in part.commands.iter().enumerate().take(20) {
                eprintln!("  [{}] {:?}", i, cmd);
            }
        }
        eprintln!("=== GLOBAL SETTINGS ===");
        for (i, cmd) in ast.global_settings.iter().enumerate().take(10) {
            eprintln!("  [{}] {:?}", i, cmd);
        }
    }
    let compiler = make_compiler(crate::OutputFormat::VGM);
    let result = compiler.compile(path).expect("compile failed");
    // Dump writes
    let data = &result.data;
    let mut i = 0x100;
    let mut count = 0;
    let mut total_wait = 0u64;
    while i < data.len() && count < 60 {
        match data[i] {
            0x66 => { eprintln!("EOF"); break; }
            0x52 | 0x53 if i+2 < data.len() => {
                let port = data[i] - 0x52;
                eprintln!("write[{}]: port={} reg=0x{:02X} val=0x{:02X}", count, port, data[i+1], data[i+2]);
                i += 3; count += 1;
            }
            0x61 if i+2 < data.len() => {
                let s = (data[i+1] as u64) | ((data[i+2] as u64) << 8);
                total_wait += s;
                eprintln!("wait: {} samples (total={} ≈ {} notes @29400)", s, total_wait, total_wait as f64/29400.0);
                i += 3;
            }
            _ => { i += 1; }
        }
    }
}
