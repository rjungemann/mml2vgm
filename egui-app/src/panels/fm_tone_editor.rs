use crate::document::DocumentStore;
use egui::{Color32, Context, RichText, Ui, Window};
use mml2vgm::instrument_serializer::{
    get_carrier_ops, parse_fm_instruments, replace_definition_block, serialize_fm_instrument,
    FmInstrumentDef, FM_PARAM_MAX, FM_PARAM_NAMES,
};

pub struct FmToneEditorState {
    pub open: bool,
    pub selected_instrument_idx: Option<usize>,
    pub working_instrument: Option<FmInstrumentDef>,
    pub instruments: Vec<FmInstrumentDef>,
}

impl Default for FmToneEditorState {
    fn default() -> Self {
        Self {
            open: false,
            selected_instrument_idx: None,
            working_instrument: None,
            instruments: Vec::new(),
        }
    }
}

pub fn show(
    ctx: &Context,
    state: &mut FmToneEditorState,
    docs: &mut DocumentStore,
) {
    if !state.open {
        return;
    }

    let mut should_close = false;

    Window::new("FM Tone Editor")
        .open(&mut state.open)
        .resizable(true)
        .default_width(700.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            // Refresh instruments from active document
            if let Some(doc) = docs.active() {
                state.instruments = parse_fm_instruments(&doc.content);
            }

            // Instrument selector
            ui.horizontal(|ui| {
                ui.label("Instrument:");
                let mut selected_name = state
                    .selected_instrument_idx
                    .and_then(|idx| state.instruments.get(idx))
                    .map(|inst| format!("'@ {} {}", inst.ty, inst.number))
                    .unwrap_or_else(|| "(none)".to_string());

                egui::ComboBox::from_id_salt("fm_instrument_selector")
                    .selected_text(&selected_name)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (idx, inst) in state.instruments.iter().enumerate() {
                            let label = if inst.name.is_empty() {
                                format!("'@ {} {}", inst.ty, inst.number)
                            } else {
                                format!("'@ {} {} ({})", inst.ty, inst.number, inst.name)
                            };

                            let is_selected = state.selected_instrument_idx == Some(idx);
                            if ui.selectable_label(is_selected, label).clicked() {
                                state.selected_instrument_idx = Some(idx);
                                if let Some(inst) = state.instruments.get(idx) {
                                    state.working_instrument = Some(inst.clone());
                                }
                            }
                        }
                    });
            });

            ui.separator();

            if state.working_instrument.is_some() {
                // Algorithm and Feedback sliders
                ui.horizontal(|ui| {
                    ui.label("ALG:");
                    if let Some(working) = &mut state.working_instrument {
                        ui.add(egui::Slider::new(&mut working.alg, 0..=7).step_by(1.0));
                    }
                    ui.label("FB:");
                    if let Some(working) = &mut state.working_instrument {
                        ui.add(egui::Slider::new(&mut working.fb, 0..=7).step_by(1.0));
                    }
                });

                // Algorithm diagram (simplified text visualization)
                if let Some(working) = &state.working_instrument {
                    show_algorithm_diagram(ui, working.alg);
                }

                ui.separator();

                // Operator grid
                ui.label(RichText::new("Operators").strong());

                // Header row
                ui.horizontal(|ui| {
                    ui.label("Op");
                    for param_name in FM_PARAM_NAMES {
                        ui.label(*param_name);
                    }
                });

                // Data rows for 4 operators
                if let Some(working) = &state.working_instrument {
                    let carrier_ops = get_carrier_ops(working.alg);
                    let ops_copy = working.ops.clone();

                    for (op_idx, _op) in ops_copy.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let is_carrier = carrier_ops.contains(&op_idx);
                            let carrier_str = if is_carrier { "●" } else { "○" };
                            ui.label(RichText::new(format!("OP{} {}", op_idx + 1, carrier_str))
                                .color(if is_carrier { Color32::YELLOW } else { Color32::GRAY }));

                            if let Some(working) = &mut state.working_instrument {
                                for (param_idx, _param_name) in FM_PARAM_NAMES.iter().enumerate() {
                                    let max_val = FM_PARAM_MAX[param_idx];
                                    ui.add(
                                        egui::Slider::new(&mut working.ops[op_idx][param_idx], 0..=max_val)
                                            .step_by(1.0)
                                            .show_value(true),
                                    );
                                }
                            }
                        });
                    }
                }

                ui.separator();

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        if let Some(doc) = docs.active_mut() {
                            if let Some(working) = &state.working_instrument {
                                let serialized = serialize_fm_instrument(working);
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
                        if let Some(idx) = state.selected_instrument_idx {
                            if let Some(inst) = state.instruments.get(idx) {
                                state.working_instrument = Some(inst.clone());
                            }
                        }
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            } else {
                ui.label(RichText::new("Select an instrument to edit").color(Color32::GRAY));
            }
        });

    if should_close {
        state.open = false;
    }
}

fn show_algorithm_diagram(ui: &mut Ui, alg: u32) {
    // Simple text-based algorithm diagram
    let diagram = match alg {
        0 => "OP1 → OP2 → OP3 → OP4 (serial)",
        1 => "OP1,2 → OP3 → OP4 (OP1,2 parallel→OP3→OP4)",
        2 => "OP1,2 → OP3, OP4 (OP1,2→OP3, parallel OP4)",
        3 => "OP1 → OP2,3,4 (OP1→OP2, OP1→OP3, OP1→OP4)",
        4 => "OP1,2,3 → OP4 (OP1→OP2→OP3→OP4, OP2→OP4, OP3→OP4)",
        5 => "OP1→OP2, OP1→OP3, OP1→OP4 (parallel carriers OP2,3,4)",
        6 => "OP1→OP2→OP3, OP1→OP4 (OP2,3,4 carriers)",
        7 => "OP1,2,3,4 all parallel carriers",
        _ => "Unknown",
    };
    ui.label(RichText::new(format!("ALG {}: {}", alg, diagram)).weak());
}
