use egui::{Color32, Rect, Stroke, Ui, Vec2};

/// Draw a waveform from a slice of peak amplitude values (0.0–1.0).
pub fn show(ui: &mut Ui, waveform: &[f32]) {
    let desired = Vec2::new(ui.available_width(), ui.available_height().max(60.0));
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());

    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_gray(18));

    if waveform.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No audio",
            egui::FontId::proportional(12.0),
            Color32::DARK_GRAY,
        );
        return;
    }

    let n = waveform.len();
    let w = rect.width();
    let h = rect.height();
    let mid_y = rect.center().y;
    let bar_w = (w / n as f32).max(1.0);
    let wave_color = Color32::from_rgb(80, 160, 255);

    for (i, &amp) in waveform.iter().enumerate() {
        let x = rect.left() + i as f32 * (w / n as f32);
        let half_h = amp.clamp(0.0, 1.0) * h * 0.45;
        let top = mid_y - half_h;
        let bot = mid_y + half_h;
        let bar_rect = Rect::from_min_max(
            egui::pos2(x, top),
            egui::pos2(x + bar_w.max(1.0) - 0.5, bot.max(top + 1.0)),
        );
        painter.rect_filled(bar_rect, 0.0, wave_color);
    }

    // Centre line
    painter.line_segment(
        [egui::pos2(rect.left(), mid_y), egui::pos2(rect.right(), mid_y)],
        Stroke::new(0.5, Color32::from_gray(60)),
    );
}
