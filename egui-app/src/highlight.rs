use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId,
};

// ── colour palette (dark-theme) ───────────────────────────────────────────────

const C_DEFAULT:  Color32 = Color32::from_rgb(210, 210, 210);
const C_COMMENT:  Color32 = Color32::from_rgb(110, 120, 110);
/// `'{` / `}` block delimiters
const C_HEADER:   Color32 = Color32::from_rgb(200, 180, 60);
/// Key names inside `'{...}` blocks
const C_HDR_KEY:  Color32 = Color32::from_rgb(140, 200, 255);
/// Values inside `'{...}` blocks
const C_HDR_VAL:  Color32 = Color32::from_rgb(255, 200, 120);
/// `'@` instrument definition lines
const C_INSTR:    Color32 = Color32::from_rgb(255, 155, 50);
/// `'A1` / `'B2` channel identifiers
const C_CHANNEL:  Color32 = Color32::from_rgb(80, 220, 220);
/// Musical note letters (c d e f g a b r) + accidentals + duration
const C_NOTE:     Color32 = Color32::from_rgb(100, 215, 100);
/// Numeric literals
const C_NUMBER:   Color32 = Color32::from_rgb(200, 150, 255);
/// MML command letters and `@N` instrument-select
const C_COMMAND:  Color32 = Color32::from_rgb(255, 230, 80);
/// `>` / `<` octave-shift operators
const C_OPERATOR: Color32 = Color32::from_rgb(255, 120, 120);

// ── public entry point ────────────────────────────────────────────────────────

/// Build a `LayoutJob` with MML syntax colouring for the full document `text`.
pub fn highlight(text: &str, font: FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut in_header = false;
    let mut pos = 0;

    while pos <= text.len() {
        let line_end = text[pos..].find('\n').map(|p| pos + p).unwrap_or(text.len());
        let line = &text[pos..line_end];
        let has_nl = line_end < text.len();

        highlight_line(&mut job, line, &mut in_header, &font);
        if has_nl {
            job.append("\n", 0.0, fmt(C_DEFAULT, &font));
        }

        pos = line_end + if has_nl { 1 } else { 0 };
        if !has_nl {
            break;
        }
    }

    job
}

// ── per-line dispatcher ───────────────────────────────────────────────────────

fn highlight_line(job: &mut LayoutJob, line: &str, in_header: &mut bool, font: &FontId) {
    // Preserve leading whitespace in default colour.
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    if indent_len > 0 {
        job.append(&line[..indent_len], 0.0, fmt(C_DEFAULT, font));
    }

    if trimmed.is_empty() {
        return;
    }

    // ── inside a header block ─────────────────────────────────────────────────
    if *in_header {
        if trimmed.starts_with('}') {
            // Closing brace: may have preceding spaces (already emitted as indent).
            job.append("}", 0.0, fmt(C_HEADER, font));
            let after = &trimmed[1..];
            if !after.is_empty() {
                job.append(after, 0.0, fmt(C_DEFAULT, font));
            }
            *in_header = false;
        } else {
            highlight_header_kv(job, trimmed, font);
        }
        return;
    }

    // ── comment line ──────────────────────────────────────────────────────────
    if trimmed.starts_with(';') {
        job.append(trimmed, 0.0, fmt(C_COMMENT, font));
        return;
    }

    // ── lines starting with the ' sigil ──────────────────────────────────────
    if trimmed.starts_with('\'') {
        // Header open: '{
        if trimmed.starts_with("'{") {
            job.append("'{", 0.0, fmt(C_HEADER, font));
            let rest = &trimmed[2..];
            if !rest.is_empty() {
                job.append(rest, 0.0, fmt(C_DEFAULT, font));
            }
            *in_header = true;
            return;
        }

        // Instrument definition: '@
        if trimmed.starts_with("'@") {
            job.append("'@", 0.0, fmt(C_INSTR, font));
            highlight_instr_data(job, &trimmed[2..], font);
            return;
        }

        // Channel/part ID: 'A1, 'B2, …
        let b = trimmed.as_bytes();
        if b.len() >= 3 && b[1].is_ascii_uppercase() && b[2].is_ascii_digit() {
            job.append(&trimmed[..3], 0.0, fmt(C_CHANNEL, font));
            highlight_mml_content(job, &trimmed[3..], font);
            return;
        }

        // Fallback ' line
        job.append(trimmed, 0.0, fmt(C_DEFAULT, font));
        return;
    }

    // ── plain text (e.g. column headers inside instrument blocks) ────────────
    job.append(trimmed, 0.0, fmt(C_DEFAULT, font));
}

// ── header key = value ────────────────────────────────────────────────────────

fn highlight_header_kv(job: &mut LayoutJob, line: &str, font: &FontId) {
    if let Some(eq) = line.find('=') {
        job.append(&line[..eq], 0.0, fmt(C_HDR_KEY, font));
        job.append("=", 0.0, fmt(C_DEFAULT, font));
        job.append(&line[eq + 1..], 0.0, fmt(C_HDR_VAL, font));
    } else {
        job.append(line, 0.0, fmt(C_DEFAULT, font));
    }
}

// ── instrument data line (after '@ ) ─────────────────────────────────────────

fn highlight_instr_data(job: &mut LayoutJob, s: &str, font: &FontId) {
    let mut pos = 0;
    while pos < s.len() {
        let b = s.as_bytes()[pos];
        if b.is_ascii_digit() {
            let start = pos;
            while pos < s.len() && s.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            job.append(&s[start..pos], 0.0, fmt(C_NUMBER, font));
        } else if b.is_ascii_alphabetic() {
            let start = pos;
            while pos < s.len() && s.as_bytes()[pos].is_ascii_alphabetic() {
                pos += 1;
            }
            job.append(&s[start..pos], 0.0, fmt(C_COMMAND, font));
        } else {
            job.append(&s[pos..pos + 1], 0.0, fmt(C_DEFAULT, font));
            pos += 1;
        }
    }
}

// ── MML content (after channel ID) ───────────────────────────────────────────

fn highlight_mml_content(job: &mut LayoutJob, s: &str, font: &FontId) {
    let mut pos = 0;
    while pos < s.len() {
        let b = s.as_bytes()[pos];

        // Inline comment: rest of line
        if b == b';' {
            job.append(&s[pos..], 0.0, fmt(C_COMMENT, font));
            return;
        }

        // Note letters: c d e f g a b  +  rest r
        if matches!(b, b'c' | b'd' | b'e' | b'f' | b'g' | b'a' | b'b' | b'r') {
            let start = pos;
            pos += 1;
            // Optional accidental
            if pos < s.len() && matches!(s.as_bytes()[pos], b'+' | b'-') {
                pos += 1;
            }
            // Optional duration digits
            while pos < s.len() && s.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            // Optional dot
            if pos < s.len() && s.as_bytes()[pos] == b'.' {
                pos += 1;
            }
            job.append(&s[start..pos], 0.0, fmt(C_NOTE, font));
            continue;
        }

        // Numeric literal
        if b.is_ascii_digit() {
            let start = pos;
            while pos < s.len() && s.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            job.append(&s[start..pos], 0.0, fmt(C_NUMBER, font));
            continue;
        }

        // @ instrument select (e.g. @0)
        if b == b'@' {
            let start = pos;
            pos += 1;
            while pos < s.len() && s.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            job.append(&s[start..pos], 0.0, fmt(C_COMMAND, font));
            continue;
        }

        // Octave operators
        if matches!(b, b'>' | b'<') {
            job.append(&s[pos..pos + 1], 0.0, fmt(C_OPERATOR, font));
            pos += 1;
            continue;
        }

        // Other command letters (T, v, l, o, q, n, w, L, …)
        if b.is_ascii_alphabetic() {
            let start = pos;
            pos += 1;
            while pos < s.len() && s.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            job.append(&s[start..pos], 0.0, fmt(C_COMMAND, font));
            continue;
        }

        // Everything else (spaces, commas, brackets, …)
        job.append(&s[pos..pos + 1], 0.0, fmt(C_DEFAULT, font));
        pos += 1;
    }
}

// ── helper ────────────────────────────────────────────────────────────────────

#[inline]
fn fmt(color: Color32, font: &FontId) -> TextFormat {
    TextFormat { color, font_id: font.clone(), ..Default::default() }
}
