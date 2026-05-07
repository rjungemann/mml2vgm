mod app;
mod document;
mod editor;
mod panels;
mod settings;

use app::MmlApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mml2vgm")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "mml2vgm",
        native_options,
        Box::new(|cc| Ok(Box::new(MmlApp::new(cc)))),
    )
}
