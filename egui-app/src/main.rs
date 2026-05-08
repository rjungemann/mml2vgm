mod app;
mod audio;
mod compiler;
mod document;
mod editor;
mod highlight;
mod midi;
mod panels;
mod settings;
mod socket;

use app::MmlApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let enable_socket = args.contains(&"--socket".to_string());
    let headless = args.contains(&"--headless".to_string());
    let port: u16 = parse_socket_port(&args).unwrap_or(7878);

    // Headless mode: run socket server without a GUI window, then exit.
    if headless {
        socket::run_headless(port);
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mml2vgm")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    // Build the socket channel before starting eframe so the server is ready.
    let socket_rx = if enable_socket {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<socket::SocketCmd>();
        socket::run(port, cmd_tx);
        Some(cmd_rx)
    } else {
        None
    };

    eframe::run_native(
        "mml2vgm",
        native_options,
        Box::new(move |cc| Ok(Box::new(MmlApp::new(cc, socket_rx)))),
    )
}

/// Parse `--socket-port <N>` from the argument list.
fn parse_socket_port(args: &[String]) -> Option<u16> {
    args.windows(2).find_map(|w| {
        if w[0] == "--socket-port" {
            w[1].parse::<u16>().ok()
        } else {
            None
        }
    })
}
