use crate::document::{CompileError, CompileStatus, DocumentStore};
use crate::panels::compile_options::{self, CompileOptions};
use crate::settings::Settings;
use egui::{Color32, Context, RichText, TopBottomPanel, CentralPanel, SidePanel};
use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::{CompileOptions as MmlCompileOptions, OutputFormat};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};

enum CompileResult {
    Ok {
        doc_id: usize,
        bytes: Vec<u8>,
        warnings: usize,
    },
    Err {
        doc_id: usize,
        errors: Vec<CompileError>,
    },
}

pub struct MmlApp {
    docs: DocumentStore,
    settings: Settings,
    compile_opts: CompileOptions,

    // Background compile channel
    compile_tx: Sender<CompileResult>,
    compile_rx: Receiver<CompileResult>,

    // Bottom panel tab
    bottom_tab: BottomTab,

    // Pending file-open future (rfd)
    pending_open: Option<std::sync::Arc<std::sync::Mutex<Option<Option<(PathBuf, String)>>>>>,
    // Pending save-as future
    pending_save_as: Option<std::sync::Arc<std::sync::Mutex<Option<Option<PathBuf>>>>>,
}

#[derive(PartialEq)]
enum BottomTab {
    Errors,
    Output,
}

impl MmlApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let compile_opts = CompileOptions::from_settings(&settings);
        let (compile_tx, compile_rx) = mpsc::channel();

        Self {
            docs: DocumentStore::new(),
            settings,
            compile_opts,
            compile_tx,
            compile_rx,
            bottom_tab: BottomTab::Errors,
            pending_open: None,
            pending_save_as: None,
        }
    }

    // ── compilation ──────────────────────────────────────────────────────────

    fn trigger_compile(&mut self, doc_id: usize) {
        let doc = match self.docs.get_mut(doc_id) {
            Some(d) => d,
            None => return,
        };
        let path = match &doc.path {
            Some(p) => p.clone(),
            None => return, // untitled: save first
        };
        doc.compile_status = CompileStatus::Compiling;

        let tx = self.compile_tx.clone();
        let format_str = self.compile_opts.format.clone();

        std::thread::spawn(move || {
            let fmt = OutputFormat::from_str(&format_str).unwrap_or(OutputFormat::VGM);
            let opts = MmlCompileOptions::default().with_output_format(fmt);
            let compiler = MmlCompiler::new(opts);
            match compiler.compile(&path) {
                Ok(result) => {
                    let _ = tx.send(CompileResult::Ok {
                        doc_id,
                        bytes: result.data,
                        warnings: result.warnings.len(),
                    });
                }
                Err(e) => {
                    let errors = vec![CompileError {
                        line: None,
                        col: None,
                        message: e.to_string(),
                    }];
                    let _ = tx.send(CompileResult::Err { doc_id, errors });
                }
            }
        });
    }

    fn poll_compile_results(&mut self) {
        while let Ok(result) = self.compile_rx.try_recv() {
            match result {
                CompileResult::Ok { doc_id, bytes, warnings } => {
                    if let Some(doc) = self.docs.get_mut(doc_id) {
                        doc.compile_status = CompileStatus::Ok { warnings };
                        doc.compiled_bytes = Some(bytes);
                    }
                }
                CompileResult::Err { doc_id, errors } => {
                    if let Some(doc) = self.docs.get_mut(doc_id) {
                        doc.compile_status = CompileStatus::Errors(errors);
                    }
                }
            }
        }
    }

    // ── auto-compile debounce ─────────────────────────────────────────────────

    fn check_auto_compile(&mut self, now: f64) {
        if !self.compile_opts.auto_compile {
            return;
        }
        let delay = self.settings.auto_compile_delay_ms as f64 / 1000.0;
        let ids: Vec<usize> = self
            .docs
            .docs()
            .iter()
            .filter(|d| {
                d.path.is_some()
                    && matches!(d.compile_status, CompileStatus::Idle | CompileStatus::Ok { .. } | CompileStatus::Errors(_))
                    && d.last_edit_time.map(|t| now - t >= delay).unwrap_or(false)
            })
            .map(|d| d.id)
            .collect();
        for id in ids {
            if let Some(doc) = self.docs.get_mut(id) {
                doc.last_edit_time = None;
            }
            self.trigger_compile(id);
        }
    }

    // ── file I/O ──────────────────────────────────────────────────────────────

    fn open_file_dialog(&mut self) {
        let slot: std::sync::Arc<std::sync::Mutex<Option<Option<(PathBuf, String)>>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let slot_clone = slot.clone();
        self.pending_open = Some(slot);

        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("MML files", &["gwi", "mml", "muc", "txt"])
                .pick_file()
                .and_then(|path| {
                    std::fs::read_to_string(&path)
                        .ok()
                        .map(|content| (path, content))
                });
            *slot_clone.lock().unwrap() = Some(result);
        });
    }

    fn save_file(&mut self, doc_id: usize) {
        let doc = match self.docs.get_mut(doc_id) {
            Some(d) => d,
            None => return,
        };
        if let Some(path) = &doc.path {
            let _ = std::fs::write(path, &doc.content);
            doc.modified = false;
        } else {
            drop(doc);
            self.save_as_dialog(doc_id);
        }
    }

    fn save_as_dialog(&mut self, doc_id: usize) {
        let slot: std::sync::Arc<std::sync::Mutex<Option<Option<PathBuf>>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let slot_clone = slot.clone();
        self.pending_save_as = Some(slot);

        let content = self
            .docs
            .get_mut(doc_id)
            .map(|d| d.content.clone())
            .unwrap_or_default();

        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("MML files", &["gwi", "mml"])
                .save_file();
            if let Some(path) = &result {
                let _ = std::fs::write(path, &content);
            }
            *slot_clone.lock().unwrap() = Some(result);
        });
    }

    fn poll_file_dialogs(&mut self) {
        // Open dialog
        if let Some(slot) = &self.pending_open {
            let done = slot.lock().unwrap().is_some();
            if done {
                let result = slot.lock().unwrap().take().flatten();
                self.pending_open = None;
                if let Some((path, content)) = result {
                    self.settings.add_recent_file(path.clone());
                    self.settings.save();
                    self.docs.open_file(path, content);
                }
            }
        }
        // Save-as dialog
        if let Some(slot) = &self.pending_save_as {
            let done = slot.lock().unwrap().is_some();
            if done {
                let result = slot.lock().unwrap().take().flatten();
                self.pending_save_as = None;
                if let Some(path) = result {
                    if let Some(doc) = self.docs.active_mut() {
                        doc.path = Some(path.clone());
                        doc.modified = false;
                        self.settings.add_recent_file(path);
                        self.settings.save();
                    }
                }
            }
        }
    }

    // ── drag-and-drop ─────────────────────────────────────────────────────────

    fn handle_dropped_files(&mut self, ctx: &Context) {
        ctx.input(|i| {
            for dropped in &i.raw.dropped_files {
                if let Some(path) = &dropped.path {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        self.settings.add_recent_file(path.clone());
                        self.docs.open_file(path.clone(), content);
                    }
                }
            }
        });
    }

    // ── keyboard shortcuts ────────────────────────────────────────────────────

    fn handle_shortcuts(&mut self, ctx: &Context) {
        ctx.input_mut(|i| {
            // Ctrl+O / Cmd+O — open file
            if i.consume_key(egui::Modifiers::COMMAND, egui::Key::O) {
                // Trigger from outside this closure; set a flag instead.
                // We'll handle via a secondary flag approach:
                i.events.push(egui::Event::Key {
                    key: egui::Key::F1, // sentinel
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                });
            }
        });

        // Simpler: check directly in update() — done below.
    }
}

impl eframe::App for MmlApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);

        // Poll background work
        self.poll_compile_results();
        self.poll_file_dialogs();
        self.handle_dropped_files(ctx);
        self.check_auto_compile(now);

        // Keep repainting while a compile or dialog is in flight
        let busy = self.pending_open.is_some()
            || self.pending_save_as.is_some()
            || self.docs.docs().iter().any(|d| d.compile_status == CompileStatus::Compiling);
        if busy {
            ctx.request_repaint();
        }

        // ── keyboard shortcuts ───────────────────────────────────────────────
        let open_requested = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O));
        let save_requested = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S));
        let new_requested = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N));
        let compile_requested = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::B));

        if open_requested && self.pending_open.is_none() {
            self.open_file_dialog();
        }
        if save_requested {
            if let Some(id) = self.docs.active_id() {
                self.save_file(id);
            }
        }
        if new_requested {
            self.docs.open_untitled();
        }
        if compile_requested {
            if let Some(id) = self.docs.active_id() {
                self.trigger_compile(id);
            }
        }

        // ── menu bar ─────────────────────────────────────────────────────────
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New         Ctrl+N").clicked() {
                        self.docs.open_untitled();
                        ui.close_menu();
                    }
                    if ui.button("Open…       Ctrl+O").clicked() && self.pending_open.is_none() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save        Ctrl+S").clicked() {
                        if let Some(id) = self.docs.active_id() {
                            self.save_file(id);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        if let Some(id) = self.docs.active_id() {
                            self.save_as_dialog(id);
                        }
                        ui.close_menu();
                    }
                    if !self.settings.recent_files.is_empty() {
                        ui.separator();
                        ui.label("Recent");
                        let recents: Vec<PathBuf> = self.settings.recent_files.clone();
                        for path in recents {
                            let label = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| path.to_string_lossy().to_string());
                            if ui.button(&label).clicked() {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    self.docs.open_file(path, content);
                                }
                                ui.close_menu();
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Build", |ui| {
                    if ui.button("Compile     Ctrl+B").clicked() {
                        if let Some(id) = self.docs.active_id() {
                            self.trigger_compile(id);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export…").clicked() {
                        if let Some(doc) = self.docs.active() {
                            if let Some(bytes) = &doc.compiled_bytes {
                                let fmt = self.compile_opts.format.clone();
                                let bytes = bytes.clone();
                                std::thread::spawn(move || {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Output", &[fmt.as_str()])
                                        .save_file()
                                    {
                                        let _ = std::fs::write(&path, &bytes);
                                    }
                                });
                            }
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Errors").clicked() {
                        self.bottom_tab = BottomTab::Errors;
                        ui.close_menu();
                    }
                    if ui.button("Output").clicked() {
                        self.bottom_tab = BottomTab::Output;
                        ui.close_menu();
                    }
                });
            });
        });

        // ── tab bar ───────────────────────────────────────────────────────────
        TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let ids: Vec<usize> = self.docs.docs().iter().map(|d| d.id).collect();
                let active_id = self.docs.active_id();
                let mut to_close: Option<usize> = None;
                let mut to_activate: Option<usize> = None;

                for id in &ids {
                    let id = *id;
                    let label = self
                        .docs
                        .docs()
                        .iter()
                        .find(|d| d.id == id)
                        .map(|d| d.tab_label())
                        .unwrap_or_default();

                    let selected = active_id == Some(id);
                    if ui
                        .selectable_label(selected, &label)
                        .on_hover_text("Click to activate")
                        .clicked()
                    {
                        to_activate = Some(id);
                    }
                    if ui.small_button("×").clicked() {
                        to_close = Some(id);
                    }
                    ui.separator();
                }

                if let Some(id) = to_activate {
                    self.docs.set_active(id);
                }
                if let Some(id) = to_close {
                    self.docs.close(id);
                }

                if ui.small_button("+").on_hover_text("New file (Ctrl+N)").clicked() {
                    self.docs.open_untitled();
                }
            });
        });

        // ── compile options toolbar ───────────────────────────────────────────
        TopBottomPanel::top("compile_toolbar").show(ctx, |ui| {
            let clicked = compile_options::show(ui, &mut self.compile_opts);
            if clicked {
                if let Some(id) = self.docs.active_id() {
                    self.trigger_compile(id);
                }
            }
        });

        // ── status bar ────────────────────────────────────────────────────────
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(doc) = self.docs.active() {
                    let file = doc
                        .path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "untitled".to_string());
                    ui.label(&file);
                    ui.separator();
                    match &doc.compile_status {
                        CompileStatus::Idle => {
                            ui.label(RichText::new("idle").color(Color32::GRAY));
                        }
                        CompileStatus::Compiling => {
                            ui.label(RichText::new("⟳ compiling…").color(Color32::YELLOW));
                        }
                        CompileStatus::Ok { warnings } => {
                            let msg = if *warnings > 0 {
                                format!("✓ ok ({warnings} warnings)")
                            } else {
                                "✓ ok".to_string()
                            };
                            ui.label(RichText::new(msg).color(Color32::GREEN));
                        }
                        CompileStatus::Errors(errs) => {
                            ui.label(
                                RichText::new(format!("✗ {} error(s)", errs.len()))
                                    .color(Color32::LIGHT_RED),
                            );
                        }
                    }
                } else {
                    ui.label(RichText::new("No file open").color(Color32::GRAY));
                }
            });
        });

        // ── bottom panel (errors / output) ────────────────────────────────────
        TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(160.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Errors, "Errors");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Output, "Output");
                });
                ui.separator();

                match self.bottom_tab {
                    BottomTab::Errors => {
                        let errors: Vec<_> = self
                            .docs
                            .active()
                            .and_then(|d| {
                                if let CompileStatus::Errors(ref e) = d.compile_status {
                                    Some(e.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        crate::panels::error_list::show(ui, &errors);
                    }
                    BottomTab::Output => {
                        if let Some(doc) = self.docs.active() {
                            if let Some(bytes) = &doc.compiled_bytes {
                                let preview: Vec<String> = bytes
                                    .iter()
                                    .take(256)
                                    .enumerate()
                                    .map(|(i, b)| {
                                        if i % 16 == 0 && i > 0 {
                                            format!("\n{b:02X}")
                                        } else {
                                            format!("{b:02X}")
                                        }
                                    })
                                    .collect();
                                egui::ScrollArea::vertical()
                                    .id_source("output_scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(
                                                &mut preview.join(" "),
                                            )
                                            .font(egui::TextStyle::Monospace)
                                            .interactive(false)
                                            .desired_width(f32::INFINITY),
                                        );
                                    });
                            } else {
                                ui.label(RichText::new("No compiled output yet.").color(Color32::GRAY));
                            }
                        } else {
                            ui.label(RichText::new("No file open.").color(Color32::GRAY));
                        }
                    }
                }
            });

        // ── central panel (editor) ────────────────────────────────────────────
        CentralPanel::default().show(ctx, |ui| {
            if self.docs.has_any() {
                let active_id = self.docs.active_id();
                if let Some(id) = active_id {
                    let doc = self.docs.get_mut(id).unwrap();
                    let changed = crate::editor::show(ui, &mut doc.content);
                    if changed {
                        doc.mark_edited(now);
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Open a file (Ctrl+O) or create a new one (Ctrl+N)")
                            .color(Color32::GRAY),
                    );
                });
            }
        });
    }
}
