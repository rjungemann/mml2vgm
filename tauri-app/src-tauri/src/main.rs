// Main Tauri application entry point
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // On macOS, set up a basic menu bar
            #[cfg(target_os = "macos")]
            {
                use tauri::Menu;
                let menu = Menu::new();
                app.set_menu(menu).expect("Failed to set menu");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
