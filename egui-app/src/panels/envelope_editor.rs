use crate::document::DocumentStore;
use egui::{Color32, Context, RichText, Ui, Window};
use mml2vgm::instrument_serializer::{
    parse_envelopes, replace_definition_block, serialize_envelope, EnvelopeDef,
};

pub struct EnvelopeEditorState {
    pub open: bool,
    pub selected_envelope_idx: Option<usize>,
    pub working_envelope: Option<EnvelopeDef>,
    pub envelopes: Vec<EnvelopeDef>,
}

impl Default for EnvelopeEditorState {
    fn default() -> Self {
        Self {
            open: false,
            selected_envelope_idx: None,
            working_envelope: None,
            envelopes: Vec::new(),
        }
    }
}

pub fn show(
    ctx: &Context,
    state: &mut EnvelopeEditorState,
    docs: &mut DocumentStore,
) {
    if !state.open {
        return;
    }

    let mut should_close = false;

    Window::new("Envelope Editor")
        .open(&mut state.open)
        .resizable(true)
        .default_width(600.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            // Refresh envelopes from active document
            if let Some(doc) = docs.active() {
                state.envelopes = parse_envelopes(&doc.content);
            }

            // Envelope selector
            ui.horizontal(|ui| {
                ui.label("Envelope:");
                let mut selected_name = state
                    .selected_envelope_idx
                    .and_then(|idx| state.envelopes.get(idx))
                    .map(|e| format!("'@ E {}", e.number))
                    .unwrap_or_else(|| "(none)".to_string());

                egui::ComboBox::from_id_salt("envelope_selector")
                    .selected_text(&selected_name)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (idx, env) in state.envelopes.iter().enumerate() {
                            let label = format!("'@ E {}", env.number);
                            let is_selected = state.selected_envelope_idx == Some(idx);
                            if ui.selectable_label(is_selected, label).clicked() {
                                state.selected_envelope_idx = Some(idx);
                                if let Some(e) = state.envelopes.get(idx) {
                                    state.working_envelope = Some(e.clone());
                                }
                            }
                        }
                    });
            });

            ui.separator();

            if state.working_envelope.is_some() {
                // Steps display and editing
                ui.label(RichText::new("Steps (0-127)").strong());

                if let Some(working) = &state.working_envelope {
                    // Bar chart preview
                    ui.horizontal(|ui| {
                        let max_step = *working.steps.iter().max().unwrap_or(&128).max(&128) as f32;
                        let height = 80.0;

                        for (idx, step) in working.steps.iter().enumerate() {
                            let bar_height = (*step as f32 / max_step) * height;
                            let (rect, _response) = ui.allocate_exact_size(egui::vec2(15.0, height), egui::Sense::hover());

                            let painter = ui.painter();
                            let bar_rect = egui::Rect::from_min_size(
                                egui::Pos2::new(rect.left(), rect.bottom() - bar_height),
                                egui::vec2(14.0, bar_height),
                            );
                            painter.rect_filled(bar_rect, 0.0, Color32::from_rgb(100, 150, 255));
                            painter.rect_stroke(bar_rect, 0.0, egui::Stroke::new(1.0, Color32::DARK_GRAY));
                        }
                    });
                }

                ui.separator();

                // Step editor grid
                ui.label("Edit Steps:");

                if let Some(working) = &mut state.working_envelope {
                    let mut steps_to_add = 0;
                    let mut steps_to_remove = 0;

                    // Show steps in a scrollable grid
                    egui::ScrollArea::vertical().id_salt("env_steps_scroll").show(ui, |ui| {
                        for (idx, step) in working.steps.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Step {}:", idx + 1));
                                ui.add(egui::Slider::new(step, 0..=127).step_by(1.0).show_value(true));
                                if ui.button("−").clicked() {
                                    steps_to_remove = idx + 1;
                                }
                            });
                        }
                    });

                    if steps_to_remove > 0 && working.steps.len() > 1 {
                        working.steps.remove(steps_to_remove - 1);
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("+ Add Step").clicked() {
                            steps_to_add = 1;
                        }
                    });

                    if steps_to_add > 0 {
                        working.steps.push(0);
                    }
                }

                ui.separator();

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        if let Some(doc) = docs.active_mut() {
                            if let Some(working) = &state.working_envelope {
                                let serialized = serialize_envelope(working);
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
                        if let Some(idx) = state.selected_envelope_idx {
                            if let Some(e) = state.envelopes.get(idx) {
                                state.working_envelope = Some(e.clone());
                            }
                        }
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            } else {
                ui.label(RichText::new("Select an envelope to edit").color(Color32::GRAY));
            }
        });

    if should_close {
        state.open = false;
    }
}
