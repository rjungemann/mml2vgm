use egui::{Color32, Stroke, Ui, Vec2, pos2};

/// Draw an oscilloscope view of raw PCM samples (values in [-1.0, 1.0]).
pub fn show(ui: &mut Ui, samples: &[f32]) {
    let desired = Vec2::new(ui.available_width(), ui.available_height().max(60.0));
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());

    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_gray(18));

    if samples.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No audio",
            egui::FontId::proportional(12.0),
            Color32::DARK_GRAY,
        );
        return;
    }

    let w = rect.width();
    let h = rect.height();
    let mid_y = rect.center().y;
    let half_h = h * 0.45;

    // Grid lines at ±0.5 and ±1.0
    let grid_dim  = Color32::from_gray(30);
    let grid_mid  = Color32::from_gray(50);
    for &level in &[1.0f32, 0.5, -0.5, -1.0] {
        let y = mid_y - level * half_h;
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(if level == 0.0 { 1.0 } else { 0.5 }, grid_dim),
        );
    }
    // Zero line (slightly brighter)
    painter.line_segment(
        [pos2(rect.left(), mid_y), pos2(rect.right(), mid_y)],
        Stroke::new(0.5, grid_mid),
    );

    // Downsample to screen pixel columns for performance
    let n = samples.len();
    let pixel_cols = (w as usize).max(1);
    let wave_color = Color32::from_rgb(80, 210, 120);

    let points: Vec<egui::Pos2> = (0..pixel_cols)
        .map(|col| {
            // Map pixel column to sample range
            let s_start = (col * n / pixel_cols).min(n - 1);
            let s_end   = ((col + 1) * n / pixel_cols).min(n);
            // Use the mean of the samples in this bucket
            let range = &samples[s_start..s_end];
            let val = if range.is_empty() {
                0.0f32
            } else {
                range.iter().sum::<f32>() / range.len() as f32
            };
            let x = rect.left() + col as f32 * w / pixel_cols as f32;
            let y = mid_y - val.clamp(-1.0, 1.0) * half_h;
            pos2(x, y)
        })
        .collect();

    for pair in points.windows(2) {
        painter.line_segment([pair[0], pair[1]], Stroke::new(1.5, wave_color));
    }
}
