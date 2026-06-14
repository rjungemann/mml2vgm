//! Comprehensive lexer tests
//!
//! Run with: cargo test compiler::tests::lexer_tests

use std::time::Instant;

use crate::compiler::lexer::{tokenize, Token};

const TIMEOUT_SECS: u64 = 1;

fn timed_tokenize(source: &str) -> Vec<Token> {
    let start = Instant::now();
    let tokens = tokenize(source).expect("tokenize failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "lexer exceeded {}s timeout",
        TIMEOUT_SECS
    );
    tokens.into_iter().map(|(t, _)| t).collect()
}

// ── Empty / trivial ──────────────────────────────────────────────────────────

#[test]
fn lex_empty_input() {
    let tokens = timed_tokenize("");
    assert!(
        tokens.is_empty(),
        "empty input should yield no tokens (eof filtered)"
    );
}

#[test]
fn lex_whitespace_only() {
    let tokens = timed_tokenize("   \t  ");
    assert!(tokens.is_empty());
}

// ── Notes ────────────────────────────────────────────────────────────────────

#[test]
fn lex_single_note_lowercase() {
    let tokens = timed_tokenize("c");
    assert_eq!(tokens[0], Token::Note('C'));
}

#[test]
fn lex_single_note_uppercase() {
    let tokens = timed_tokenize("C");
    assert_eq!(tokens[0], Token::Note('C'));
}

#[test]
fn lex_all_note_letters() {
    let tokens = timed_tokenize("c d e f g a b");
    let notes: Vec<char> = tokens
        .iter()
        .filter_map(|t| {
            if let Token::Note(c) = t {
                Some(*c)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(notes, vec!['C', 'D', 'E', 'F', 'G', 'A', 'B']);
}

// ── Rest ─────────────────────────────────────────────────────────────────────

#[test]
fn lex_rest_lowercase() {
    let tokens = timed_tokenize("r");
    assert_eq!(tokens[0], Token::Rest);
}

#[test]
fn lex_rest_uppercase() {
    let tokens = timed_tokenize("R");
    assert_eq!(tokens[0], Token::Rest);
}

#[test]
fn lex_rest_with_duration() {
    let tokens = timed_tokenize("r4");
    assert_eq!(tokens[0], Token::Rest);
    assert_eq!(tokens[1], Token::Number(4));
}

// ── Octave ───────────────────────────────────────────────────────────────────

#[test]
fn lex_octave_command_lowercase() {
    let tokens = timed_tokenize("o4");
    assert_eq!(tokens[0], Token::OctaveCommand);
    assert_eq!(tokens[1], Token::Number(4));
}

#[test]
fn lex_octave_up() {
    let tokens = timed_tokenize(">");
    assert_eq!(tokens[0], Token::GreaterThan);
}

#[test]
fn lex_octave_down() {
    let tokens = timed_tokenize("<");
    assert_eq!(tokens[0], Token::LessThan);
}

// ── Volume / Tempo / Length ──────────────────────────────────────────────────

#[test]
fn lex_volume_command() {
    let tokens = timed_tokenize("v13");
    assert_eq!(tokens[0], Token::VolumeCommand);
    assert_eq!(tokens[1], Token::Number(13));
}

#[test]
fn lex_tempo_command_lowercase() {
    let tokens = timed_tokenize("t120");
    assert_eq!(tokens[0], Token::TempoCommand);
    assert_eq!(tokens[1], Token::Number(120));
}

#[test]
fn lex_tempo_command_uppercase() {
    let tokens = timed_tokenize("T120");
    assert_eq!(tokens[0], Token::TempoCommand);
    assert_eq!(tokens[1], Token::Number(120));
}

#[test]
fn lex_length_command() {
    let tokens = timed_tokenize("l8");
    assert_eq!(tokens[0], Token::LengthCommand);
    assert_eq!(tokens[1], Token::Number(8));
}

// ── Modifiers ────────────────────────────────────────────────────────────────

#[test]
fn lex_dotted_note() {
    let tokens = timed_tokenize("c4.");
    assert_eq!(tokens[0], Token::Note('C'));
    assert_eq!(tokens[1], Token::Number(4));
    assert_eq!(tokens[2], Token::Dot);
}

#[test]
fn lex_tied_note() {
    let tokens = timed_tokenize("c4_");
    assert_eq!(tokens[0], Token::Note('C'));
    assert_eq!(tokens[1], Token::Number(4));
    assert_eq!(tokens[2], Token::Underscore);
}

// ── Instrument / At-sign ─────────────────────────────────────────────────────

#[test]
fn lex_at_sign() {
    let tokens = timed_tokenize("@");
    assert_eq!(tokens[0], Token::AtSign);
}

#[test]
fn lex_at_sign_with_number() {
    let tokens = timed_tokenize("@0");
    assert_eq!(tokens[0], Token::AtSign);
    assert_eq!(tokens[1], Token::Number(0));
}

// ── Loops ────────────────────────────────────────────────────────────────────

#[test]
fn lex_infinite_loop_brackets() {
    let tokens = timed_tokenize("[cde]");
    assert_eq!(tokens[0], Token::LeftBracket);
    assert_eq!(tokens[4], Token::RightBracket);
}

#[test]
fn lex_finite_loop_parens() {
    let tokens = timed_tokenize("(cde)3");
    assert_eq!(tokens[0], Token::LeftParen);
    assert_eq!(tokens[4], Token::RightParen);
    assert_eq!(tokens[5], Token::Number(3));
}

// ── Structure tokens ─────────────────────────────────────────────────────────

#[test]
fn lex_bar_line() {
    let tokens = timed_tokenize("|");
    assert_eq!(tokens[0], Token::Bar);
}

#[test]
fn lex_apostrophe() {
    let tokens = timed_tokenize("'");
    assert_eq!(tokens[0], Token::Apostrophe);
}

#[test]
fn lex_left_right_brace() {
    let tokens = timed_tokenize("{}");
    assert_eq!(tokens[0], Token::LeftBrace);
    assert_eq!(tokens[1], Token::RightBrace);
}

#[test]
fn lex_equals_comma() {
    let tokens = timed_tokenize("=,");
    assert_eq!(tokens[0], Token::Equals);
    assert_eq!(tokens[1], Token::Comma);
}

// ── String literals ──────────────────────────────────────────────────────────

#[test]
fn lex_string_literal() {
    let tokens = timed_tokenize("\"Hello World\"");
    assert_eq!(tokens[0], Token::StringLiteral("Hello World".to_string()));
}

#[test]
fn lex_empty_string_literal() {
    let tokens = timed_tokenize("\"\"");
    assert_eq!(tokens[0], Token::StringLiteral(String::new()));
}

// ── Identifiers ──────────────────────────────────────────────────────────────

#[test]
fn lex_identifier() {
    let tokens = timed_tokenize("TitleName");
    assert_eq!(tokens[0], Token::Identifier("TitleName".to_string()));
}

// ── Numeric literals ─────────────────────────────────────────────────────────

#[test]
fn lex_zero() {
    let tokens = timed_tokenize("0");
    assert_eq!(tokens[0], Token::Number(0));
}

#[test]
fn lex_large_number() {
    let tokens = timed_tokenize("99999");
    assert_eq!(tokens[0], Token::Number(99999));
}

// ── Definition line ──────────────────────────────────────────────────────────

#[test]
fn lex_apostrophe_definition_prefix() {
    let tokens = timed_tokenize("'@ E 0");
    assert_eq!(tokens[0], Token::Apostrophe);
    assert_eq!(tokens[1], Token::AtSign);
}

// ── Multiline ────────────────────────────────────────────────────────────────

#[test]
fn lex_multiline_song() {
    let source = "'{
  TitleName = Test
}
'A1 t120 o4 l4
'A1 c d e f
";
    // Must not panic
    let _tokens = timed_tokenize(source);
}

// ── Performance / stress ─────────────────────────────────────────────────────

#[test]
fn lex_very_long_note_sequence() {
    let source: String = "c d e f g a b ".repeat(1_429); // ~10_003 notes
    let start = Instant::now();
    let tokens = tokenize(&source).expect("tokenize failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "lex_very_long_note_sequence exceeded timeout"
    );
    assert!(!tokens.is_empty());
}

#[test]
fn lex_deeply_nested_loops() {
    // 500 opening brackets followed by notes and 500 closing brackets
    let opens: String = "[".repeat(500);
    let closes: String = "]".repeat(500);
    let source = format!("{}c{}", opens, closes);
    let start = Instant::now();
    let tokens = tokenize(&source).expect("tokenize failed");
    assert!(
        start.elapsed().as_secs() < TIMEOUT_SECS,
        "lex_deeply_nested_loops exceeded timeout"
    );
    assert!(!tokens.is_empty());
}

// ── MIDI note number command ─────────────────────────────────────────────────

#[test]
fn lex_note_number_command_lowercase() {
    let tokens = timed_tokenize("n37");
    assert_eq!(tokens[0], Token::NoteNumberCommand);
    assert_eq!(tokens[1], Token::Number(37));
}

#[test]
fn lex_note_number_command_uppercase() {
    let tokens = timed_tokenize("N60");
    assert_eq!(tokens[0], Token::NoteNumberCommand);
    assert_eq!(tokens[1], Token::Number(60));
}

#[test]
fn lex_note_number_in_sequence() {
    // n37 sandwiched between regular notes: a n37 a
    let tokens = timed_tokenize("a n37 a");
    let kinds: Vec<_> = tokens.iter().map(std::mem::discriminant).collect();
    // Note, NoteNumberCommand, Number, Note
    assert!(tokens.contains(&Token::NoteNumberCommand));
    assert_eq!(tokens[0], Token::Note('A'));
    assert_eq!(tokens[1], Token::NoteNumberCommand);
    assert_eq!(tokens[2], Token::Number(37));
    assert_eq!(tokens[3], Token::Note('A'));
    let _ = kinds;
}
