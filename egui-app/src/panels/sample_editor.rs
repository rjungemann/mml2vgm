use crate::document::DocumentStore;
use egui::{Color32, Context, RichText, Ui, Window};
use mml2vgm::instrument_serializer::{
    parse_pcm_instruments, replace_definition_block, serialize_pcm_instrument, PcmInstrumentDef,
};

pub struct SampleEditorState {
    pub open: bool,
    pub selected_sample_idx: Option<usize>,
    pub working_sample: Option<PcmInstrumentDef>,
    pub samples: Vec<PcmInstrumentDef>,
}

impl Default for SampleEditorState {
    fn default() -> Self {
        Self {
            open: false,
            selected_sample_idx: None,
            working_sample: None,
            samples: Vec::new(),
        }
    }
}

pub fn show(
    ctx: &Context,
    state: &mut SampleEditorState,
    docs: &mut DocumentStore,
) {
    if !state.open {
        return;
    }

    let mut should_close = false;

    Window::new("Sample/PCM Editor")
        .open(&mut state.open)
        .resizable(true)
        .default_width(600.0)
        .default_height(500.0)
        .show(ctx, |ui| {
            // Refresh samples from active document
            if let Some(doc) = docs.active() {
                state.samples = parse_pcm_instruments(&doc.content);
            }

            // Sample selector
            ui.horizontal(|ui| {
                ui.label("Sample:");
                let selected_name = state
                    .selected_sample_idx
                    .and_then(|idx| state.samples.get(idx))
                    .map(|s| format!("'@ P {}", s.number))
                    .unwrap_or_else(|| "(none)".to_string());

                egui::ComboBox::from_id_salt("sample_selector")
                    .selected_text(&selected_name)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (idx, sample) in state.samples.iter().enumerate() {
                            let label = format!("'@ P {} ({})", sample.number, sample.filename);
                            let is_selected = state.selected_sample_idx == Some(idx);
                            if ui.selectable_label(is_selected, label).clicked() {
                                state.selected_sample_idx = Some(idx);
                                if let Some(s) = state.samples.get(idx) {
                                    state.working_sample = Some(s.clone());
                                }
                            }
                        }
                    });
            });

            ui.separator();

            if state.working_sample.is_some() {
                // Filename
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    if let Some(working) = &mut state.working_sample {
                        ui.text_edit_singleline(&mut working.filename);
                    }
                });

                // Chip selector
                ui.horizontal(|ui| {
                    ui.label("Chip:");
                    if let Some(working) = &mut state.working_sample {
                        const CHIPS: &[&str] = &["SegaPCM", "RF5C164", "HuC6280", "ADPCM"];
                        egui::ComboBox::from_id_salt("sample_chip_selector")
                            .selected_text(&working.chip)
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                for &chip in CHIPS {
                                    let is_selected = working.chip == chip;
                                    if ui.selectable_label(is_selected, chip).clicked() {
                                        working.chip = chip.to_string();
                                    }
                                }
                            });
                    }
                });

                // Frequency
                ui.horizontal(|ui| {
                    ui.label("Frequency (Hz):");
                    if let Some(working) = &mut state.working_sample {
                        ui.add(egui::Slider::new(&mut working.frequency, 100..=96000).step_by(100.0).show_value(true));
                    }
                });

                // Volume
                ui.horizontal(|ui| {
                    ui.label("Volume (0-255):");
                    if let Some(working) = &mut state.working_sample {
                        ui.add(egui::Slider::new(&mut working.volume, 0..=255).step_by(1.0).show_value(true));
                    }
                });

                // Options
                ui.horizontal(|ui| {
                    ui.label("Options:");
                    if let Some(working) = &mut state.working_sample {
                        ui.text_edit_singleline(&mut working.option);
                    }
                });

                ui.separator();

                // Control buttons
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        if let Some(doc) = docs.active_mut() {
                            if let Some(working) = &state.working_sample {
                                let serialized = serialize_pcm_instrument(working);
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
                        if let Some(idx) = state.selected_sample_idx {
                            if let Some(s) = state.samples.get(idx) {
                                state.working_sample = Some(s.clone());
                            }
                        }
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            } else {
                ui.label(RichText::new("Select a sample to edit").color(Color32::GRAY));
            }
        });

    if should_close {
        state.open = false;
    }
}
