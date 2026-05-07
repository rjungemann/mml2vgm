use crate::audio::AudioEngine;
use egui::{Color32, RichText, Slider, Ui};

/// Show the playback control panel.
/// Returns `(play_clicked, pause_clicked, stop_clicked)`.
pub fn show(ui: &mut Ui, audio: &mut AudioEngine) {
    ui.horizontal(|ui| {
        let has = audio.has_buffer();

        // Play / Pause toggle
        if audio.is_playing() {
            if ui
                .add_enabled(has, egui::Button::new("⏸"))
                .on_hover_text("Pause")
                .clicked()
            {
                audio.pause();
            }
        } else {
            if ui
                .add_enabled(has, egui::Button::new("▶"))
                .on_hover_text("Play")
                .clicked()
            {
                audio.play();
            }
        }

        // Stop
        if ui
            .add_enabled(has, egui::Button::new("⏹"))
            .on_hover_text("Stop")
            .clicked()
        {
            audio.stop();
        }

        ui.separator();

        // Loop toggle
        let mut looping = audio.looping();
        if ui.toggle_value(&mut looping, "🔁").on_hover_text("Loop").changed() {
            audio.set_loop(looping);
        }

        ui.separator();

        // Seek bar (read-only position indicator)
        let pos = audio.position_secs();
        let dur = audio.duration_secs();
        if dur > 0.0 {
            let frac = (pos / dur).clamp(0.0, 1.0) as f32;
            let pos_label = fmt_time(pos);
            let dur_label = fmt_time(dur);
            ui.label(
                RichText::new(format!("{pos_label} / {dur_label}"))
                    .color(Color32::LIGHT_GRAY)
                    .monospace(),
            );
            // Draw a thin progress bar inline
            let desired = egui::vec2(ui.available_width().min(220.0), 8.0);
            let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                let bg = Color32::from_gray(50);
                let fg = Color32::from_rgb(80, 160, 255);
                ui.painter().rect_filled(rect, 2.0, bg);
                let mut filled = rect;
                filled.set_right(rect.left() + rect.width() * frac);
                ui.painter().rect_filled(filled, 2.0, fg);
            }
        } else {
            ui.label(RichText::new("--:-- / --:--").color(Color32::DARK_GRAY).monospace());
        }

        ui.separator();

        // Volume slider
        ui.label("🔊");
        let mut vol = audio.volume();
        if ui
            .add(Slider::new(&mut vol, 0.0..=1.5).fixed_decimals(2).show_value(false))
            .changed()
        {
            audio.set_volume(vol);
        }
    });
}

fn fmt_time(secs: f64) -> String {
    let s = secs as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}
