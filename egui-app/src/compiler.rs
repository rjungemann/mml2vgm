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

// ── keyboard preview helpers ──────────────────────────────────────────────────

/// Convert a MIDI note number to an MML (octave, note_name) pair.
/// MIDI 60 = C4 → `(4, "c")`.
pub fn midi_to_mml_note(midi: u8) -> (i32, &'static str) {
    const NAMES: [&str; 12] = ["c", "c+", "d", "d+", "e", "f", "f+", "g", "g+", "a", "a+", "b"];
    let octave = (midi as i32) / 12 - 1;
    let note = NAMES[(midi % 12) as usize];
    (octave, note)
}

/// Collect setup tokens from a channel command string, stopping at the first
/// note/rest/octave-shift character (c d e f g a b r > <).
fn extract_setup_tokens(content: &str) -> String {
    let mut result: Vec<&str> = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b' ' || ch == b'\t' { i += 1; continue; }
        if ch == b';' { break; }
        if matches!(ch, b'>' | b'<') { break; }
        if matches!(ch, b'c' | b'd' | b'e' | b'f' | b'g' | b'a' | b'b' | b'r') { break; }
        let start = i;
        while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'\t' { i += 1; }
        if start < i {
            result.push(&content[start..i]);
        }
    }
    result.join(" ")
}

/// Return the unique channel identifiers found in `source` (e.g. `["A1", "A2", "B1"]`),
/// in order of first appearance.
pub fn detect_channels(source: &str) -> Vec<String> {
    let mut channels: Vec<String> = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        let b = t.as_bytes();
        if b.len() >= 3 && b[0] == b'\'' && b[1].is_ascii_uppercase() && b[2].is_ascii_digit() {
            let ch = format!("{}{}", b[1] as char, b[2] as char);
            if !channels.contains(&ch) {
                channels.push(ch);
            }
        }
    }
    channels
}

/// Build a minimal MML snippet that plays `midi_note` as a half note on `channel`,
/// reusing the instrument/header definitions from `source`.
/// Returns `None` if `channel` is too short or empty.
pub fn build_note_preview(source: &str, midi_note: u8, channel: &str) -> Option<String> {
    if channel.len() < 2 { return None; }
    let ch_bytes = channel.as_bytes();
    let (ch_letter, ch_digit) = (ch_bytes[0], ch_bytes[1]);

    let mut header_lines: Vec<String> = Vec::new();
    let mut instr_lines: Vec<String> = Vec::new();
    let mut setup_cmds: Vec<String> = Vec::new();
    let mut in_header = false;

    for line in source.lines() {
        let t = line.trim();
        if t.is_empty() { continue; }
        if !in_header && (t.starts_with("'{") || t == "{") {
            in_header = true;
            header_lines.push(t.to_string());
            continue;
        }
        if in_header {
            header_lines.push(t.to_string());
            if t == "}" { in_header = false; }
            continue;
        }
        if t.starts_with("'@") {
            instr_lines.push(t.to_string());
            continue;
        }
        let b = t.as_bytes();
        if b.len() > 3 && b[0] == b'\'' && b[1] == ch_letter && b[2] == ch_digit {
            let content = t[3..].trim_start();
            let setup = extract_setup_tokens(content);
            if !setup.is_empty() {
                setup_cmds.push(setup);
            }
        }
    }

    let (octave, note_name) = midi_to_mml_note(midi_note);
    let ch_sigil = format!("'{}", channel);

    let mut parts: Vec<String> = header_lines;
    parts.extend(instr_lines);
    // Deduplicate setup commands that set the same parameter prefix (keep last).
    for cmd in &setup_cmds {
        parts.push(format!("{} {}", ch_sigil, cmd));
    }
    // Half note then a half rest so the note has time to ring before the VGM ends.
    parts.push(format!("{} o{} {}2 r2", ch_sigil, octave, note_name));
    Some(parts.join("\n"))
}
