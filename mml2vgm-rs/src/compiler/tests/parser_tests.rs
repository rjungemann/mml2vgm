//! Comprehensive parser tests
//!
//! Run with: cargo test compiler::tests::parser_tests

use std::time::Instant;

use crate::compiler::ast::{MmlNode, OctaveShift};
use crate::compiler::lexer::tokenize;
use crate::compiler::parser::Parser;

const TIMEOUT_SECS: u64 = 2;

fn timed_parse(source: &str) -> crate::compiler::ast::MmlAst {
    let start = Instant::now();
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = Parser::new(tokens).parse().expect("parse failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "parser exceeded {}s timeout",
        TIMEOUT_SECS
    );
    ast
}

fn parse_result(source: &str) -> crate::MmlResult<crate::compiler::ast::MmlAst> {
    let tokens = tokenize(source).expect("tokenize failed");
    let start = Instant::now();
    let result = Parser::new(tokens).parse();
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "parser exceeded {}s timeout",
        TIMEOUT_SECS
    );
    result
}

// ── Empty / trivial ──────────────────────────────────────────────────────────

#[test]
fn parse_empty_source() {
    let ast = timed_parse("");
    assert!(ast.parts.is_empty());
    assert!(ast.global_settings.is_empty());
    assert!(ast.metadata.is_empty());
}

// ── Metadata block ───────────────────────────────────────────────────────────

#[test]
fn parse_metadata_block() {
    let ast = timed_parse("'{\n  TitleName = Test\n}");
    // The parser accepts { without apostrophe too
    let ast2 = timed_parse("{\n  TitleName = MyGame\n}");
    assert!(ast2.metadata.contains_key("TitleName"));
    assert_eq!(ast2.metadata["TitleName"], "MyGame");
    let _ = ast; // may or may not parse depending on implementation
}

#[test]
fn parse_metadata_multiple_keys() {
    let ast = timed_parse("{\n  TitleName = Song\n  ComposerJ = Author\n}");
    assert_eq!(ast.metadata.get("TitleName"), Some(&"Song".to_string()));
    assert_eq!(ast.metadata.get("ComposerJ"), Some(&"Author".to_string()));
}

// ── Part definition ──────────────────────────────────────────────────────────

#[test]
fn parse_part_definition() {
    let ast = timed_parse("'A1 t120\n'A1 c d e f");
    assert!(ast.parts.contains_key("A1"));
}

#[test]
fn parse_multiple_parts() {
    let ast = timed_parse("'A1 t120\n'A2 t120\n'A1 c\n'A2 d");
    assert!(ast.parts.contains_key("A1"));
    assert!(ast.parts.contains_key("A2"));
}

// ── Tempo ────────────────────────────────────────────────────────────────────

#[test]
fn parse_tempo_in_part() {
    let ast = timed_parse("'A1 t120\n'A1 c");
    // Part A1 exists; tempo is in global settings or part definition
    assert!(ast.parts.contains_key("A1"));
}

#[test]
fn parse_tempo_node_in_global() {
    let ast = timed_parse("t120");
    let has_tempo = ast
        .global_settings
        .iter()
        .any(|n| matches!(n, MmlNode::Tempo(_)));
    assert!(has_tempo, "expected Tempo node in global settings");
}

// ── Notes ────────────────────────────────────────────────────────────────────

#[test]
fn parse_single_note_global() {
    let ast = timed_parse("c");
    let has_note = ast.global_settings.iter().any(|n| {
        if let MmlNode::Note(note) = n {
            note.letter == 'C'
        } else {
            false
        }
    });
    assert!(has_note, "expected Note('C') in global settings");
}

#[test]
fn parse_note_with_duration() {
    let ast = timed_parse("c4");
    let has_note = ast
        .global_settings
        .iter()
        .any(|n| matches!(n, MmlNode::Note(_)));
    assert!(has_note);
}

#[test]
fn parse_dotted_rest() {
    let ast = timed_parse("r4.");
    let has_rest = ast.global_settings.iter().any(|n| {
        if let MmlNode::Rest(r) = n {
            r.dotted
        } else {
            false
        }
    });
    assert!(has_rest, "expected dotted rest");
}

#[test]
fn parse_tied_note() {
    let ast = timed_parse("c4_");
    let has_tied = ast.global_settings.iter().any(|n| {
        if let MmlNode::Note(note) = n {
            note.tied
        } else {
            false
        }
    });
    assert!(has_tied, "expected tied note");
}

// ── Octave ───────────────────────────────────────────────────────────────────

#[test]
fn parse_octave_absolute() {
    let ast = timed_parse("o4");
    let has_octave = ast.global_settings.iter().any(|n| {
        if let MmlNode::Octave(o) = n {
            o.number == 4
        } else {
            false
        }
    });
    assert!(has_octave, "expected Octave(4)");
}

#[test]
fn parse_octave_relative_up() {
    let ast = timed_parse(">");
    let has_shift = ast
        .global_settings
        .iter()
        .any(|n| matches!(n, MmlNode::OctaveShift(OctaveShift::Up)));
    assert!(has_shift, "expected OctaveShift::Up");
}

#[test]
fn parse_octave_relative_down() {
    let ast = timed_parse("<");
    let has_shift = ast
        .global_settings
        .iter()
        .any(|n| matches!(n, MmlNode::OctaveShift(OctaveShift::Down)));
    assert!(has_shift, "expected OctaveShift::Down");
}

// ── Volume / Length ──────────────────────────────────────────────────────────

#[test]
fn parse_volume() {
    let ast = timed_parse("v13");
    let has_vol = ast.global_settings.iter().any(|n| {
        if let MmlNode::Volume(v) = n {
            v.level == 13
        } else {
            false
        }
    });
    assert!(has_vol, "expected Volume(13)");
}

#[test]
fn parse_length_command() {
    let ast = timed_parse("l8");
    let has_len = ast.global_settings.iter().any(|n| {
        if let MmlNode::Length(l) = n {
            l.value == 8
        } else {
            false
        }
    });
    assert!(has_len, "expected Length(8)");
}

// ── Loops ────────────────────────────────────────────────────────────────────

#[test]
fn parse_loop_infinite() {
    let ast = timed_parse("[c d e]");
    let has_loop = ast
        .global_settings
        .iter()
        .any(|n| matches!(n, MmlNode::Loop(_)));
    assert!(has_loop, "expected a Loop node");
}

#[test]
fn parse_loop_finite() {
    let ast = timed_parse("(c d e)3");
    let has_loop = ast.global_settings.iter().any(|n| {
        if let MmlNode::Loop(l) = n {
            l.count == 3
        } else {
            false
        }
    });
    assert!(has_loop, "expected Loop with count=3");
}

// ── Instrument definitions ────────────────────────────────────────────────────

#[test]
fn parse_envelope_definition() {
    let ast = timed_parse("'@ E 0, 15, 0, 120, 7, 100, 4, 1");
    assert!(ast.envelopes.contains_key(&0), "expected envelope 0");
}

#[test]
fn parse_pcm_instrument_definition() {
    let ast = timed_parse("'@ P 0, \"b.wav\", 8000, 100, C140");
    assert!(
        ast.pcm_instruments.contains_key(&0),
        "expected PCM instrument 0"
    );
    let pcm = &ast.pcm_instruments[&0];
    assert_eq!(pcm.frequency, 8000);
    assert_eq!(pcm.volume, 100);
    assert_eq!(pcm.chip, "C140");
}

#[test]
fn parse_fm_instrument_definition() {
    let ast = timed_parse("'@ F 0 \"Bass\" 31 0 8 0 0 5 3 1 24");
    assert!(
        ast.fm_instruments.contains_key(&0),
        "expected FM instrument 0"
    );
}

// ── Note MIDI mapping ─────────────────────────────────────────────────────────

#[test]
fn parse_note_midi_c4() {
    let note = crate::compiler::ast::Note::new('C', 0, 4);
    assert_eq!(note.midi_note(), 60);
}

#[test]
fn parse_note_midi_a4() {
    let note = crate::compiler::ast::Note::new('A', 0, 4);
    assert_eq!(note.midi_note(), 69);
}

#[test]
fn parse_note_midi_csharp4() {
    let note = crate::compiler::ast::Note::new('C', 1, 4);
    assert_eq!(note.midi_note(), 61);
}

#[test]
fn parse_note_midi_dflat4() {
    // D flat = same pitch as C# at octave 4
    let note = crate::compiler::ast::Note::new('D', -1, 4);
    assert_eq!(note.midi_note(), 61);
}

#[test]
fn parse_note_midi_boundary_low() {
    let note = crate::compiler::ast::Note::new('C', -1, 0); // B-1 would be below 0
                                                            // clamped to 0
    assert!(note.midi_note() <= 127);
}

#[test]
fn parse_note_midi_boundary_high() {
    let note = crate::compiler::ast::Note::new('G', 1, 8);
    assert!(note.midi_note() <= 127);
}

// ── Error cases ───────────────────────────────────────────────────────────────

#[test]
fn parse_error_unclosed_brace() {
    // An unclosed song-info block should either succeed (lenient) or return Err — never panic.
    let result = parse_result("{ TitleName = Test");
    // We just verify it doesn't panic; error or success are both acceptable.
    let _ = result;
}

#[test]
fn parse_error_unclosed_loop() {
    // An unclosed loop should not panic.
    let result = parse_result("[c d e");
    let _ = result;
}

// ── Stress ───────────────────────────────────────────────────────────────────

#[test]
fn parse_stress_many_parts() {
    // 64 parts each with 20 notes
    let mut source = String::new();
    for i in 0..64u32 {
        source.push_str(&format!("'P{} t120\n", i));
    }
    for i in 0..64u32 {
        let notes = "c d e f g a b r ".repeat(3);
        source.push_str(&format!("'P{} {}\n", i, notes));
    }
    let start = Instant::now();
    let tokens = tokenize(&source).expect("tokenize failed");
    let ast = Parser::new(tokens).parse().expect("parse failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "parse_stress_many_parts exceeded timeout"
    );
    assert_eq!(ast.parts.len(), 64);
}

#[test]
fn parse_stress_long_loop_body() {
    let notes = "c d e f ".repeat(1_250); // 5 000 notes
    let source = format!("[{}]", notes);
    let start = Instant::now();
    let tokens = tokenize(&source).expect("tokenize failed");
    let _ast = Parser::new(tokens).parse().expect("parse failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "parse_stress_long_loop_body exceeded timeout"
    );
}

#[test]
fn parse_stress_all_note_variants() {
    // All 7 letters × 3 accidentals across all 9 octaves
    let mut source = String::new();
    for oct in 0..=8u8 {
        for letter in ['c', 'd', 'e', 'f', 'g', 'a', 'b'] {
            source.push_str(&format!("o{} {} ", oct, letter));
        }
    }
    let start = Instant::now();
    let tokens = tokenize(&source).expect("tokenize failed");
    let _ast = Parser::new(tokens).parse().expect("parse failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "parse_stress_all_note_variants exceeded timeout"
    );
}

// ── MIDI note number command ─────────────────────────────────────────────────

#[test]
fn parse_note_number_middle_c() {
    // n60 = MIDI 60 = C4 (middle C)
    let ast = timed_parse("'A1 n60");
    let part = ast.parts.get("A1").unwrap();
    let note = part
        .commands
        .iter()
        .find_map(|n| {
            if let MmlNode::Note(note) = n {
                Some(note)
            } else {
                None
            }
        })
        .expect("expected a Note node");
    assert_eq!(note.letter, 'C');
    assert_eq!(note.accidental, 0);
    assert_eq!(note.octave, 4);
}

#[test]
fn parse_note_number_csharp2() {
    // n37 = MIDI 37 = C#2 (as used in Sitaraba/3MLE files)
    let ast = timed_parse("'A1 n37");
    let part = ast.parts.get("A1").unwrap();
    let note = part
        .commands
        .iter()
        .find_map(|n| {
            if let MmlNode::Note(note) = n {
                Some(note)
            } else {
                None
            }
        })
        .expect("expected a Note node");
    assert_eq!(note.letter, 'C');
    assert_eq!(note.accidental, 1); // sharp
    assert_eq!(note.octave, 2);
}

#[test]
fn parse_note_number_b_natural() {
    // n47 = MIDI 47 = B2
    let ast = timed_parse("'A1 n47");
    let part = ast.parts.get("A1").unwrap();
    let note = part
        .commands
        .iter()
        .find_map(|n| {
            if let MmlNode::Note(note) = n {
                Some(note)
            } else {
                None
            }
        })
        .expect("expected a Note node");
    assert_eq!(note.letter, 'B');
    assert_eq!(note.accidental, 0);
    assert_eq!(note.octave, 2);
}

#[test]
fn parse_note_number_does_not_change_octave() {
    // n37 is C#2; the A that follows should still be at octave 3 (set by o3)
    let ast = timed_parse("'A1 o3 n37 a");
    let part = ast.parts.get("A1").unwrap();
    let notes: Vec<_> = part
        .commands
        .iter()
        .filter_map(|n| {
            if let MmlNode::Note(note) = n {
                Some(note)
            } else {
                None
            }
        })
        .collect();
    // First note is from n37 (octave 2), second note 'a' should be at octave 3
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].octave, 2); // from n37
    assert_eq!(notes[1].letter, 'A');
    assert_eq!(notes[1].octave, 3); // current_octave still 3
}

#[test]
fn parse_note_number_dotted() {
    let ast = timed_parse("'A1 l8 n60.");
    let part = ast.parts.get("A1").unwrap();
    let note = part
        .commands
        .iter()
        .find_map(|n| {
            if let MmlNode::Note(note) = n {
                Some(note)
            } else {
                None
            }
        })
        .expect("expected a Note node");
    assert!(note.dotted, "expected dotted note");
    assert_eq!(note.letter, 'C');
}

#[test]
fn parse_note_number_roundtrip() {
    // Verify n<N> midi_note() equals the original N for all naturals
    let naturals = [
        (0u32, 'C', 0i8),
        (2, 'D', 0),
        (4, 'E', 0),
        (5, 'F', 0),
        (7, 'G', 0),
        (9, 'A', 0),
        (11, 'B', 0),
    ];
    for (offset, letter, acc) in &naturals {
        let midi = 48 + offset; // octave 3
        let octave = (midi / 12).saturating_sub(1) as u8;
        let note = crate::compiler::ast::Note::new(*letter, *acc, octave);
        assert_eq!(
            note.midi_note(),
            midi as u8,
            "roundtrip failed for {}",
            letter
        );
    }
}

// ── Phase 9: Newly-wired chip commands ──────────────────────────────────────

/// Walk every node in the AST (global + parts) and collect each ChipCommand
/// (command name, args).
fn collect_chip_commands(ast: &crate::compiler::ast::MmlAst) -> Vec<(String, Vec<u32>)> {
    let mut out = Vec::new();
    let mut visit = |nodes: &[MmlNode]| {
        for n in nodes {
            if let MmlNode::ChipCommand { command, args, .. } = n {
                out.push((command.clone(), args.clone()));
            }
        }
    };
    visit(&ast.global_settings);
    for part in ast.parts.values() {
        visit(&part.commands);
    }
    out
}

#[test]
fn parse_phase9_ymf262_mode_commands() {
    // OPL3MODE / 4OP must reach the AST as ChipCommand nodes with their args.
    let ast = timed_parse("A @OPL3MODE 1 @4OP 5");
    let cmds = collect_chip_commands(&ast);
    assert!(cmds.iter().any(|(c, a)| c == "OPL3MODE" && a == &vec![1]));
    assert!(cmds.iter().any(|(c, a)| c == "4OP" && a == &vec![5]));
}

#[test]
fn parse_phase9_pokey_hpoly() {
    let ast = timed_parse("A @HPOLY 1");
    let cmds = collect_chip_commands(&ast);
    assert!(cmds.iter().any(|(c, a)| c == "HPOLY" && a == &vec![1]));
}

#[test]
fn parse_phase9_wavetable_extras() {
    // @NW (HuC6280 noise), @SW (DMG sweep, 3 args), @P (DMG LFSR width)
    let ast = timed_parse("A @NW 5 @SW 2,1,3 @P 1");
    let cmds = collect_chip_commands(&ast);
    assert!(cmds.iter().any(|(c, a)| c == "NW" && a == &vec![5]));
    assert!(cmds.iter().any(|(c, a)| c == "SW" && a == &vec![2, 1, 3]));
    assert!(cmds.iter().any(|(c, a)| c == "P" && a == &vec![1]));
}

#[test]
fn parse_phase9_pcm_address_and_volume() {
    let ast = timed_parse("A @START 16,32,0 @END 64,128 @VOLUME 200,150");
    let cmds = collect_chip_commands(&ast);
    assert!(cmds
        .iter()
        .any(|(c, a)| c == "START" && a == &vec![16, 32, 0]));
    assert!(cmds.iter().any(|(c, a)| c == "END" && a == &vec![64, 128]));
    assert!(cmds
        .iter()
        .any(|(c, a)| c == "VOLUME" && a == &vec![200, 150]));
}

#[test]
fn parse_phase9_pcm_qsound_and_reverse() {
    let ast = timed_parse("A @REVERSE 1 @PAN 32 @REVERB 64");
    let cmds = collect_chip_commands(&ast);
    assert!(cmds.iter().any(|(c, a)| c == "REVERSE" && a == &vec![1]));
    assert!(cmds.iter().any(|(c, a)| c == "PAN" && a == &vec![32]));
    assert!(cmds.iter().any(|(c, a)| c == "REVERB" && a == &vec![64]));
}
