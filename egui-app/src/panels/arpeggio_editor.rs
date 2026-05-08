use crate::document::DocumentStore;
use egui::{Color32, Context, RichText, Ui, Window};
use mml2vgm::instrument_serializer::{
    parse_arpeggios, replace_definition_block, serialize_arpeggio, ArpeggioDef,
};

pub struct ArpeggioEditorState {
    pub open: bool,
    pub selected_arpeggio_idx: Option<usize>,
    pub working_arpeggio: Option<ArpeggioDef>,
    pub arpeggios: Vec<ArpeggioDef>,
}

impl Default for ArpeggioEditorState {
    fn default() -> Self {
        Self {
            open: false,
            selected_arpeggio_idx: None,
            working_arpeggio: None,
            arpeggios: Vec::new(),
        }
    }
}

pub fn show(
    ctx: &Context,
    state: &mut ArpeggioEditorState,
    docs: &mut DocumentStore,
) {
    if !state.open {
        return;
    }

    let mut should_close = false;

    Window::new("Arpeggio Editor")
        .open(&mut state.open)
        .resizable(true)
        .default_width(600.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            // Refresh arpeggios from active document
            if let Some(doc) = docs.active() {
                state.arpeggios = parse_arpeggios(&doc.content);
            }

            // Arpeggio selector
            ui.horizontal(|ui| {
                ui.label("Arpeggio:");
                let mut selected_name = state
                    .selected_arpeggio_idx
                    .and_then(|idx| state.arpeggios.get(idx))
                    .map(|a| format!("'@ A {}", a.number))
                    .unwrap_or_else(|| "(none)".to_string());

                egui::ComboBox::from_id_salt("arpeggio_selector")
                    .selected_text(&selected_name)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (idx, arp) in state.arpeggios.iter().enumerate() {
                            let label = format!("'@ A {}", arp.number);
                            let is_selected = state.selected_arpeggio_idx == Some(idx);
                            if ui.selectable_label(is_selected, label).clicked() {
                                state.selected_arpeggio_idx = Some(idx);
                                if let Some(a) = state.arpeggios.get(idx) {
                                    state.working_arpeggio = Some(a.clone());
                                }
                            }
                        }
                    });
            });

            ui.separator();

            if state.working_arpeggio.is_some() {
                // Notes display
                ui.label(RichText::new("Notes").strong());

                if let Some(working) = &state.working_arpeggio {
                    let notes_str = working.notes.join(" ");
                    ui.label(RichText::new(&notes_str).monospace());
                }

                ui.separator();

                // Note editor
                ui.label("Edit Notes:");

                if let Some(working) = &mut state.working_arpeggio {
                    let mut note_to_remove = None;

                    egui::ScrollArea::vertical().id_salt("arp_notes_scroll").show(ui, |ui| {
                        for (idx, note) in working.notes.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Note {}:", idx + 1));
                                ui.text_edit_singleline(note);
                                if ui.button("−").clicked() {
                                    note_to_remove = Some(idx);
                                }
                            });
                        }
                    });

                    if let Some(idx) = note_to_remove {
                        if working.notes.len() > 1 {
                            working.notes.remove(idx);
                        }
                    }

                    ui.separator();

                    // Add note button
                    if ui.button("+ Add Note").clicked() {
                        working.notes.push("C4".to_string());
                    }

                    // Note hints
                    ui.label(
                        RichText::new("Format: C4, D#4, E5, etc. (C=0, C#=1, D=2, D#=3, E=4, F=5, F#=6, G=7, G#=8, A=9, A#=10, B=11)")
                            .weak()
                            .small(),
                    );
                }

                ui.separator();

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        if let Some(doc) = docs.active_mut() {
                            if let Some(working) = &state.working_arpeggio {
                                let serialized = serialize_arpeggio(working);
                                let new_content = replace_definition_block(
                                    &doc.content,
                                    working.start_line,
                                    working.end_line,
                                    &serialized,
                                );
                                doc.content = new_content;
                                doc.modified = true;
                                doc.mark_edited(ctx.input(|i| i.time));
                            }
                        }
                    }

                    if ui.button("Revert").clicked() {
                        if let Some(idx) = state.selected_arpeggio_idx {
                            if let Some(a) = state.arpeggios.get(idx) {
                                state.working_arpeggio = Some(a.clone());
                            }
                        }
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            } else {
                ui.label(RichText::new("Select an arpeggio to edit").color(Color32::GRAY));
            }
        });

    if should_close {
        state.open = false;
    }
}
