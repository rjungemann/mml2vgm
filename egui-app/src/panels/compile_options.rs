use crate::settings::Settings;
use egui::Ui;

pub struct CompileOptions {
    pub format: String,
    pub chip: String,
    pub auto_compile: bool,
}

impl CompileOptions {
    pub fn from_settings(s: &Settings) -> Self {
        Self {
            format: s.default_format.clone(),
            chip: s.default_chip.clone(),
            auto_compile: s.auto_compile,
        }
    }
}

const FORMATS: &[&str] = &["vgm", "xgm", "xgm2", "zgm"];
const CHIPS: &[&str] = &[
    "auto",
    "YM2612", "SN76489", "YM2608", "YM2151", "YM2203",
    "YM3812", "YM3526", "YMF262", "Y8950", "YM2413",
    "AY8910", "HuC6280", "RF5C164", "SegaPCM",
];

/// Show the compile options bar. Returns `true` if the user clicked Compile.
pub fn show(ui: &mut Ui, opts: &mut CompileOptions) -> bool {
    let mut compile_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Format:");
        egui::ComboBox::from_id_salt("format_selector")
            .selected_text(&opts.format)
            .show_ui(ui, |ui| {
                for &fmt in FORMATS {
                    ui.selectable_value(&mut opts.format, fmt.to_string(), fmt);
                }
            });

        ui.separator();

        ui.label("Chip:");
        egui::ComboBox::from_id_salt("chip_selector")
            .selected_text(&opts.chip)
            .width(110.0)
            .show_ui(ui, |ui| {
                for &chip in CHIPS {
                    ui.selectable_value(&mut opts.chip, chip.to_string(), chip);
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
