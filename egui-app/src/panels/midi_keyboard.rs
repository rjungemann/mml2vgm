use egui::{Color32, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};

/// Notes that are black keys (within an octave 0–11).
const BLACK_KEY_OFFSETS: [u8; 5] = [1, 3, 6, 8, 10]; // C#, D#, F#, G#, A#

/// Returns true if `note_in_octave` (0=C … 11=B) is a black key.
fn is_black(note_in_octave: u8) -> bool {
    BLACK_KEY_OFFSETS.contains(&note_in_octave)
}

/// Returns the white-key index (0-based) within an octave for a note.
/// Black keys return the same index as the white key to their left.
fn white_key_index(note_in_octave: u8) -> u8 {
    // C D E F G A B → 0 1 2 3 4 5 6
    match note_in_octave {
        0 => 0, 1 => 0,  // C, C#
        2 => 1, 3 => 1,  // D, D#
        4 => 2,          // E
        5 => 3, 6 => 3,  // F, F#
        7 => 4, 8 => 4,  // G, G#
        9 => 5, 10 => 5, // A, A#
        11 => 6,         // B
        _ => 0,
    }
}

/// Show a 3-octave piano keyboard (C3–B5, MIDI notes 48–83).
///
/// `active` — 128-element bool array: which notes are currently pressed.
///
/// Returns a note that was clicked (NoteOn), and a note that was released
/// (NoteOff).  Both may be `None`.
pub fn show(
    ui: &mut Ui,
    active: &[bool; 128],
    mouse_held_note: &mut Option<u8>,
) -> (Option<u8>, Option<u8>) {
    const FIRST_NOTE: u8 = 48; // C3
    const OCTAVES: u8 = 3;
    const WHITE_KEYS: usize = OCTAVES as usize * 7; // 21

    let white_w = 22.0f32;
    let white_h = 90.0f32;
    let black_w = 14.0f32;
    let black_h = 56.0f32;

    let total_w = WHITE_KEYS as f32 * white_w;
    let desired = Vec2::new(total_w, white_h + 4.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

    if !ui.is_rect_visible(rect) {
        return (None, None);
    }

    let painter = ui.painter_at(rect);
    let origin = rect.left_top();

    // ── draw white keys ───────────────────────────────────────────────────────
    let mut white_idx = 0usize;
    for octave in 0..OCTAVES {
        for note_in_oct in 0u8..12 {
            if is_black(note_in_oct) { continue; }
            let midi = FIRST_NOTE + octave * 12 + note_in_oct;
            let x = origin.x + white_idx as f32 * white_w;
            let key_rect = Rect::from_min_size(
                Pos2::new(x, origin.y),
                Vec2::new(white_w - 1.0, white_h),
            );
            let pressed = midi < 128 && active[midi as usize];
            let bg = if pressed { Color32::from_rgb(120, 180, 255) } else { Color32::WHITE };
            painter.rect_filled(key_rect, 0.0, bg);
            painter.rect_stroke(key_rect, 0.0, Stroke::new(1.0, Color32::DARK_GRAY));
            white_idx += 1;
        }
    }

    // ── draw black keys (on top) ───────────────────────────────────────────────
    let mut white_idx2 = 0usize;
    for octave in 0..OCTAVES {
        let oct_x = origin.x + white_idx2 as f32 * white_w;
        for note_in_oct in 0u8..12 {
            if !is_black(note_in_oct) {
                white_idx2 += 1;
                continue;
            }
            let midi = FIRST_NOTE + octave * 12 + note_in_oct;
            let wk = white_key_index(note_in_oct);
            // Position black key between the white key to the left and right.
            let x = oct_x + wk as f32 * white_w + white_w - black_w * 0.5;
            let key_rect = Rect::from_min_size(
                Pos2::new(x, origin.y),
                Vec2::new(black_w, black_h),
            );
            let pressed = midi < 128 && active[midi as usize];
            let bg = if pressed { Color32::from_rgb(60, 120, 220) } else { Color32::from_gray(30) };
            painter.rect_filled(key_rect, 1.0, bg);
        }
    }

    // ── mouse interaction ──────────────────────────────────────────────────────
    let mut note_on: Option<u8> = None;
    let mut note_off: Option<u8> = None;

    if let Some(pos) = response.interact_pointer_pos() {
        let note = hit_test(pos, origin, FIRST_NOTE, OCTAVES, white_w, white_h, black_w, black_h);
        if response.is_pointer_button_down_on() {
            if *mouse_held_note != note {
                // Release old note
                if let Some(prev) = mouse_held_note.take() {
                    note_off = Some(prev);
                }
                // Press new note
                if let Some(n) = note {
                    *mouse_held_note = Some(n);
                    note_on = Some(n);
                }
            }
        }
    }

    // Release when mouse lifted
    if !response.is_pointer_button_down_on() {
        if let Some(held) = mouse_held_note.take() {
            note_off = Some(held);
        }
    }

    (note_on, note_off)
}

/// Find which MIDI note (if any) is under `pos`.
/// Black keys take priority over white keys (they are drawn on top).
fn hit_test(
    pos: Pos2,
    origin: Pos2,
    first_note: u8,
    octaves: u8,
    white_w: f32,
    white_h: f32,
    black_w: f32,
    black_h: f32,
) -> Option<u8> {
    let lx = pos.x - origin.x;
    let ly = pos.y - origin.y;
    if lx < 0.0 || ly < 0.0 || ly > white_h { return None; }

    // Check black keys first (they sit on top).
    if ly <= black_h {
        let mut wi = 0usize;
        for octave in 0..octaves {
            let oct_x = wi as f32 * white_w;
            for note_in_oct in 0u8..12 {
                if !is_black(note_in_oct) {
                    wi += 1;
                    continue;
                }
                let wk = white_key_index(note_in_oct);
                let bx = oct_x + wk as f32 * white_w + white_w - black_w * 0.5;
                if lx >= bx && lx < bx + black_w {
                    let midi = first_note + octave * 12 + note_in_oct;
                    return Some(midi);
                }
            }
        }
    }

    // White key fallback.
    let wi = (lx / white_w) as usize;
    if wi >= (octaves as usize * 7) { return None; }
    let octave = (wi / 7) as u8;
    let key_in_oct = (wi % 7) as u8;
    // Map white-key index to note: C=0,D=2,E=4,F=5,G=7,A=9,B=11
    let note_in_oct: u8 = match key_in_oct {
        0 => 0, 1 => 2, 2 => 4, 3 => 5, 4 => 7, 5 => 9, 6 => 11, _ => 0,
    };
    Some(first_note + octave * 12 + note_in_oct)
}

/// Draw a compact note label (e.g. "C4") for display.
#[allow(dead_code)]
pub fn note_name(midi: u8) -> String {
    const NAMES: [&str; 12] = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    let octave = (midi / 12) as i8 - 1;
    format!("{}{}", NAMES[(midi % 12) as usize], octave)
}
