//! Parser stress tests
//!
//! These tests exercise worst-case paths for the lexer and parser.
//! They are marked #[ignore] and excluded from the default test run.
//!
//! Run with: cargo test --test parser_stress -- --ignored --nocapture
//! Or all at once: cargo test --test parser_stress -- --include-ignored --nocapture

use std::time::Instant;
use mml2vgm::compiler::lexer::tokenize;
use mml2vgm::compiler::parser::Parser;
use mml2vgm::{CompileOptions, OutputFormat};
use mml2vgm::compiler::compiler::MmlCompiler;

/// Global timeout for every stress test in seconds.
const STRESS_TIMEOUT_SECS: u64 = 30;

macro_rules! within_timeout {
    ($label:expr, $block:block) => {{
        let _start = Instant::now();
        $block
        let elapsed = _start.elapsed();
        assert!(
            elapsed.as_secs() < STRESS_TIMEOUT_SECS,
            "{} exceeded {}s stress timeout (took {:.2}s)",
            $label,
            STRESS_TIMEOUT_SECS,
            elapsed.as_secs_f64()
        );
    }};
}

// ── 50 000 notes ─────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_50k_notes() {
    within_timeout!("stress_50k_notes", {
        let notes = "c d e f g a b r ".repeat(6_250); // 50 000 notes
        let tokens = tokenize(&notes).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}

// ── 1 000 parts ──────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_1k_parts() {
    within_timeout!("stress_1k_parts", {
        let mut source = String::with_capacity(1_000 * 64);
        for i in 0..1_000u32 {
            source.push_str(&format!("'P{} t120\n", i));
        }
        for i in 0..1_000u32 {
            source.push_str(&format!("'P{} c d e f g a b r\n", i));
        }
        let tokens = tokenize(&source).expect("tokenize failed");
        let ast = Parser::new(tokens).parse().expect("parse failed");
        assert_eq!(ast.parts.len(), 1_000);
    });
}

// ── 500 levels of infinite-loop nesting ──────────────────────────────────────

#[test]
#[ignore]
fn stress_deeply_nested_infinite_loops_500() {
    within_timeout!("stress_deeply_nested_infinite_loops_500", {
        let opens: String = "[".repeat(500);
        let closes: String = "]".repeat(500);
        let source = format!("{}c{}", opens, closes);
        let tokens = tokenize(&source).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}

// ── 200 levels of finite-loop nesting ────────────────────────────────────────

#[test]
#[ignore]
fn stress_deeply_nested_finite_loops_200() {
    within_timeout!("stress_deeply_nested_finite_loops_200", {
        let opens: String = "(c ".repeat(200);
        let closes: String = ")2 ".repeat(200);
        let source = format!("{}{}", opens, closes);
        let tokens = tokenize(&source).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}

// ── 64 KB string literal in metadata ─────────────────────────────────────────

#[test]
#[ignore]
fn stress_very_long_string_literal() {
    within_timeout!("stress_very_long_string_literal", {
        let long_value: String = "a".repeat(64 * 1_024);
        let source = format!("{{\n  Notes = \"{}\"\n}}", long_value);
        let tokens = tokenize(&source).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}

// ── All token types interleaved 10 000 iterations ────────────────────────────

#[test]
#[ignore]
fn stress_all_valid_tokens_interleaved() {
    within_timeout!("stress_all_valid_tokens_interleaved", {
        // A cycle that touches: note, rest, octave-up, octave-down, bar, volume, tempo, length
        let cycle = "c r > < | v8 t120 l4 ";
        let source = cycle.repeat(10_000);
        let tokens = tokenize(&source).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}

// ── Max tempo ─────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_max_tempo() {
    within_timeout!("stress_max_tempo", {
        let notes = "c d e f ".repeat(2_500); // 10 000 notes
        let source = format!("t255 {}", notes);
        let compiler = MmlCompiler::new(CompileOptions {
            format: OutputFormat::VGM,
            ..Default::default()
        });
        let result = compiler.compile_from_source(&source);
        assert!(result.is_ok(), "max tempo compile failed: {:?}", result.err());
    });
}

// ── Min tempo ─────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_min_tempo() {
    within_timeout!("stress_min_tempo", {
        let source = "t1 c d e f";
        let compiler = MmlCompiler::new(CompileOptions {
            format: OutputFormat::VGM,
            ..Default::default()
        });
        let result = compiler.compile_from_source(source);
        // Must not divide-by-zero; error is acceptable
        let _ = result;
    });
}

// ── Octave boundary cycling ────────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_octave_boundary_cycling() {
    within_timeout!("stress_octave_boundary_cycling", {
        // Rapidly alternate > and < 10 000 times
        let octave_cycle = "> < ".repeat(10_000);
        let source = format!("o4 {}", octave_cycle);
        let tokens = tokenize(&source).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}

// ── 256 PCM instrument definitions ───────────────────────────────────────────

#[test]
#[ignore]
fn stress_large_pcm_instrument_table() {
    within_timeout!("stress_large_pcm_instrument_table", {
        let mut source = String::new();
        for i in 0..256u32 {
            source.push_str(&format!(
                "'@ P {}, \"sample{}.wav\", 8000, 100, C140\n",
                i, i
            ));
        }
        source.push_str("'A1 t120\n'A1 c d e f\n");
        let compiler = MmlCompiler::new(CompileOptions {
            format: OutputFormat::VGM,
            ..Default::default()
        });
        let result = compiler.compile_from_source(&source);
        let _ = result; // may error due to missing wavs; must not panic
    });
}

// ── Multi-format sequential ───────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_multiformat_sequential() {
    let notes = "c d e f g a b r ".repeat(3_000); // large-ish source
    let source = format!("t120 o4 l8 {}", notes);

    let overall_start = Instant::now();
    let formats = [OutputFormat::VGM, OutputFormat::XGM, OutputFormat::XGM2, OutputFormat::ZGM];

    for format in &formats {
        let compiler = MmlCompiler::new(CompileOptions {
            format: *format,
            ..Default::default()
        });
        let _ = compiler.compile_from_source(&source);
    }

    let elapsed = overall_start.elapsed();
    assert!(
        elapsed.as_secs() < 60,
        "stress_multiformat_sequential exceeded 60s (took {:.2}s)",
        elapsed.as_secs_f64()
    );
}

// ── Unicode metadata ──────────────────────────────────────────────────────────

#[test]
#[ignore]
fn stress_unicode_metadata() {
    within_timeout!("stress_unicode_metadata", {
        let source = "{\n  TitleName = \"サウンドテスト音楽\"\n  ComposerJ = \"作曲家の名前\"\n  Notes = \"これはテストです。ゲームミュージック\"\n}\n'A1 t120\n'A1 c d e f\n";
        let tokens = tokenize(source).expect("tokenize failed");
        let _ast = Parser::new(tokens).parse().expect("parse failed");
    });
}
