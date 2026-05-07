use crate::document::CompileError;
use egui::{Color32, RichText, ScrollArea, Ui};

/// Show the error list panel. Returns `Some(line)` if the user clicked an error to jump to.
pub fn show(ui: &mut Ui, errors: &[CompileError]) -> Option<usize> {
    let mut jump_to = None;

    if errors.is_empty() {
        ui.label(RichText::new("No errors.").color(Color32::GRAY));
        return None;
    }

    ScrollArea::vertical()
        .id_source("error_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for err in errors {
                let loc = match (err.line, err.col) {
                    (Some(l), Some(c)) => format!("[{l}:{c}] "),
                    (Some(l), None) => format!("[{l}] "),
                    _ => String::new(),
                };
                let label = format!("{loc}{}", err.message);
                let response = ui.add(
                    egui::Label::new(RichText::new(&label).color(Color32::LIGHT_RED).monospace())
                        .sense(egui::Sense::click()),
                );
                if response.clicked() {
                    if let Some(line) = err.line {
                        jump_to = Some(line);
                    }
                }
                response.on_hover_text("Click to jump to line");
            }
        });

    jump_to
}
