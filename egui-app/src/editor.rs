use egui::{Response, ScrollArea, TextEdit, Ui};

/// Show the code editor for `content`. Returns `true` if the content was changed.
///
/// If `jump_to_line` is `Some(n)` (1-based), the cursor is moved to that line
/// before rendering; the caller should clear the value after this call.
pub fn show(ui: &mut Ui, content: &mut String, jump_to_line: Option<usize>) -> bool {
    let te_id = ui.make_persistent_id("code_editor");

    if let Some(line) = jump_to_line {
        let char_offset = line_start_char(content, line);
        let ccursor = egui::text::CCursor { index: char_offset, prefer_next_row: false };
        let cursor = egui::text::CCursorRange::one(ccursor);
        let mut state = egui::text_edit::TextEditState::load(ui.ctx(), te_id)
            .unwrap_or_default();
        state.cursor.set_char_range(Some(cursor));
        state.store(ui.ctx(), te_id);
    }

    let body_font = egui::TextStyle::Monospace.resolve(ui.style());
    let mut layouter = |_ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut job = crate::highlight::highlight(string, body_font.clone());
        job.wrap.max_width = wrap_width;
        _ui.fonts(|f| f.layout_job(job))
    };

    let mut changed = false;
    ScrollArea::both()
        .id_salt("editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let response: Response = ui.add(
                TextEdit::multiline(content)
                    .id(te_id)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40)
                    .lock_focus(true)
                    .layouter(&mut layouter),
            );
            if response.changed() {
                changed = true;
            }
        });
    changed
}

/// Returns the character offset of the first character on `line` (1-based).
fn line_start_char(content: &str, line: usize) -> usize {
    if line <= 1 { return 0; }
    let mut cur_line = 1;
    for (char_idx, ch) in content.chars().enumerate() {
        if ch == '\n' {
            cur_line += 1;
            if cur_line >= line {
                return char_idx + 1;
            }
        }
    }
    content.chars().count()
}
