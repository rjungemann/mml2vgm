use crate::settings::Settings;
use egui::Ui;

pub struct CompileOptions {
    pub format: String,
    pub auto_compile: bool,
}

impl CompileOptions {
    pub fn from_settings(s: &Settings) -> Self {
        Self {
            format: s.default_format.clone(),
            auto_compile: s.auto_compile,
        }
    }
}

/// Show the compile options bar. Returns `true` if the user clicked Compile.
pub fn show(ui: &mut Ui, opts: &mut CompileOptions) -> bool {
    let mut compile_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Format:");
        egui::ComboBox::from_id_source("format_selector")
            .selected_text(&opts.format)
            .show_ui(ui, |ui| {
                for fmt in ["vgm", "xgm", "xgm2", "zgm"] {
                    ui.selectable_value(&mut opts.format, fmt.to_string(), fmt);
                }
            });

        ui.separator();

        ui.checkbox(&mut opts.auto_compile, "Auto-compile");

        ui.separator();

        if ui.button("▶ Compile").clicked() {
            compile_clicked = true;
        }
    });

    compile_clicked
}
