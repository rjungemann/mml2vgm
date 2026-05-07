use crate::settings::{Settings, Theme};
use egui::{Context, Ui, Window};

/// Show the settings window. `open` is toggled by the caller.
pub fn show(ctx: &Context, open: &mut bool, settings: &mut Settings, midi_inputs: &[String], midi_outputs: &[String]) {
    let mut save_clicked = false;
    Window::new("Settings")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.heading("Appearance");
            theme_row(ui, settings);
            font_size_row(ui, settings);

            ui.separator();
            ui.heading("Compilation");
            auto_compile_row(ui, settings);

            ui.separator();
            ui.heading("MIDI");
            midi_input_row(ui, settings, midi_inputs);
            midi_output_row(ui, settings, midi_outputs);

            ui.separator();
            if ui.button("Save").clicked() {
                settings.save();
                save_clicked = true;
            }
        });
    if save_clicked {
        *open = false;
    }
}

fn theme_row(ui: &mut Ui, settings: &mut Settings) {
    ui.horizontal(|ui| {
        ui.label("Theme:");
        ui.selectable_value(&mut settings.theme, Theme::Dark, "Dark");
        ui.selectable_value(&mut settings.theme, Theme::Light, "Light");
    });
}

fn font_size_row(ui: &mut Ui, settings: &mut Settings) {
    ui.horizontal(|ui| {
        ui.label("Font size:");
        ui.add(egui::Slider::new(&mut settings.font_size, 10.0..=24.0).step_by(1.0));
    });
}

fn auto_compile_row(ui: &mut Ui, settings: &mut Settings) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut settings.auto_compile, "Auto-compile on change");
    });
    ui.horizontal(|ui| {
        ui.label("Delay (ms):");
        ui.add(egui::Slider::new(&mut settings.auto_compile_delay_ms, 100..=2000).step_by(100.0));
    });
}

fn midi_input_row(ui: &mut Ui, settings: &mut Settings, ports: &[String]) {
    ui.horizontal(|ui| {
        ui.label("MIDI Input:");
        let current = settings.preferred_midi_input.clone().unwrap_or_else(|| "(none)".to_string());
        egui::ComboBox::from_id_salt("midi_in_sel")
            .selected_text(&current)
            .width(220.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(settings.preferred_midi_input.is_none(), "(none)").clicked() {
                    settings.preferred_midi_input = None;
                }
                for name in ports {
                    let sel = settings.preferred_midi_input.as_deref() == Some(name.as_str());
                    if ui.selectable_label(sel, name).clicked() {
                        settings.preferred_midi_input = Some(name.clone());
                    }
                }
            });
    });
}

fn midi_output_row(ui: &mut Ui, settings: &mut Settings, ports: &[String]) {
    ui.horizontal(|ui| {
        ui.label("MIDI Output:");
        let current = settings.preferred_midi_output.clone().unwrap_or_else(|| "(none)".to_string());
        egui::ComboBox::from_id_salt("midi_out_sel")
            .selected_text(&current)
            .width(220.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(settings.preferred_midi_output.is_none(), "(none)").clicked() {
                    settings.preferred_midi_output = None;
                }
                for name in ports {
                    let sel = settings.preferred_midi_output.as_deref() == Some(name.as_str());
                    if ui.selectable_label(sel, name).clicked() {
                        settings.preferred_midi_output = Some(name.clone());
                    }
                }
            });
    });
}
