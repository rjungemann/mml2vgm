use egui::{Response, ScrollArea, TextEdit, Ui};

/// Show the code editor for `content`. Returns `true` if the content was changed.
pub fn show(ui: &mut Ui, content: &mut String) -> bool {
    let mut changed = false;
    ScrollArea::both()
        .id_source("editor_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let response: Response = ui.add(
                TextEdit::multiline(content)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40)
                    .lock_focus(true)
                    .code_editor(),
            );
            if response.changed() {
                changed = true;
            }
        });
    changed
}
