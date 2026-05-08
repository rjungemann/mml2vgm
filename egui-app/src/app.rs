use crate::audio::{AudioBuffer, AudioEngine};
use crate::compiler;
use crate::document::{CompileError, CompileStatus, DocumentStore};
use crate::live_audio::LiveAudioEngine;
use crate::midi::{MidiEvent, MidiManager};
use crate::panels::compile_options::{self, CompileOptions};
use crate::settings::{Settings, Theme};
use crate::socket::{SocketCmd, SocketRequest};
use egui::{CentralPanel, Color32, Context, RichText, TopBottomPanel};
use mml2vgm::player::VgmPlayer;
use serde_json::json;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

// ── background messages ───────────────────────────────────────────────────────

enum WorkerMsg {
    CompileOk { doc_id: usize, bytes: Vec<u8>, pcm: Vec<f32>, warnings: usize },
    CompileErr { doc_id: usize, errors: Vec<CompileError> },
}

// ── app ───────────────────────────────────────────────────────────────────────

pub struct MmlApp {
    docs: DocumentStore,
    settings: Settings,
    compile_opts: CompileOptions,

    worker_tx: Sender<WorkerMsg>,
    worker_rx: Receiver<WorkerMsg>,

    audio: Option<AudioEngine>,
    midi: MidiManager,

    // 128-element active-note array updated from MIDI input.
    active_notes: [bool; 128],
    // Note currently held by mouse click on keyboard.
    keyboard_held: Option<u8>,

    bottom_tab: BottomTab,
    settings_open: bool,
    fm_editor: crate::panels::fm_tone_editor::FmToneEditorState,
    envelope_editor: crate::panels::envelope_editor::EnvelopeEditorState,
    arpeggio_editor: crate::panels::arpeggio_editor::ArpeggioEditorState,
    sample_editor: crate::panels::sample_editor::SampleEditorState,

    pending_open: PendingSlot<Option<(PathBuf, String)>>,
    pending_save_as: PendingSlot<Option<PathBuf>>,

    /// Optional socket command receiver (present when `--socket` flag is set).
    socket_rx: Option<Receiver<SocketCmd>>,

    /// Channel selected for on-screen keyboard note preview (e.g. "A1").
    keyboard_preview_channel: String,
    /// Receives (note, pcm) pairs from background preview-compile threads.
    preview_rx: Receiver<(u8, Vec<f32>)>,
    preview_tx: Sender<(u8, Vec<f32>)>,

    /// Real-time chip-register audio engine for the on-screen keyboard.
    live_audio: Option<LiveAudioEngine>,
}

type PendingSlot<T> = Option<std::sync::Arc<std::sync::Mutex<Option<T>>>>;

#[derive(PartialEq)]
enum BottomTab {
    Errors,
    Output,
    Waveform,
    Midi,
}

impl MmlApp {
    pub fn new(cc: &eframe::CreationContext<'_>, socket_rx: Option<Receiver<SocketCmd>>) -> Self {
        let settings = Settings::load();
        let compile_opts = CompileOptions::from_settings(&settings);
        let (worker_tx, worker_rx) = mpsc::channel();
        let (preview_tx, preview_rx) = mpsc::channel();
        let audio = AudioEngine::new();

        apply_theme(&cc.egui_ctx, &settings);
        apply_font_size(&cc.egui_ctx, settings.font_size);

        let mut midi = MidiManager::new();
        // Auto-connect preferred ports from settings.
        if let Some(ref name) = settings.preferred_midi_input.clone() {
            if let Some(idx) = midi.input_port_names.iter().position(|n| n == name) {
                midi.connect_input(idx);
            }
        }
        if let Some(ref name) = settings.preferred_midi_output.clone() {
            if let Some(idx) = midi.output_port_names.iter().position(|n| n == name) {
                midi.connect_output(idx);
            }
        }

        let mut docs = DocumentStore::new();
        docs.open_untitled();

        let live_audio = LiveAudioEngine::new();

        Self {
            docs,
            settings,
            compile_opts,
            worker_tx,
            worker_rx,
            audio,
            midi,
            active_notes: [false; 128],
            keyboard_held: None,
            bottom_tab: BottomTab::Errors,
            settings_open: false,
            fm_editor: Default::default(),
            envelope_editor: Default::default(),
            arpeggio_editor: Default::default(),
            sample_editor: Default::default(),
            pending_open: None,
            pending_save_as: None,
            socket_rx,
            keyboard_preview_channel: String::new(),
            preview_rx,
            preview_tx,
            live_audio,
        }
    }

    // ── compilation ───────────────────────────────────────────────────────────

    fn trigger_compile(&mut self, doc_id: usize) {
        let doc = match self.docs.get_mut(doc_id) {
            Some(d) => d,
            None => return,
        };
        doc.compile_status = CompileStatus::Compiling;
        let content = doc.content.clone();
        let tx = self.worker_tx.clone();
        let format = self.compile_opts.format.clone();
        std::thread::spawn(move || {
            match compiler::compile_content(&content, &format) {
                Ok(out) => {
                    let pcm = render_pcm(&out.bytes);
                    let _ = tx.send(WorkerMsg::CompileOk { doc_id, bytes: out.bytes, pcm, warnings: out.warnings });
                }
                Err(errors) => {
                    let _ = tx.send(WorkerMsg::CompileErr { doc_id, errors });
                }
            }
        });
    }

    fn poll_workers(&mut self) {
        while let Ok(msg) = self.worker_rx.try_recv() {
            match msg {
                WorkerMsg::CompileOk { doc_id, bytes, pcm, warnings } => {
                    // Grab content before mutably borrowing doc
                    let content = self.docs.get(doc_id)
                        .map(|d| d.content.clone())
                        .unwrap_or_default();
                    if let Some(doc) = self.docs.get_mut(doc_id) {
                        doc.compile_status = CompileStatus::Ok { warnings };
                        doc.compiled_bytes = Some(bytes);
                    }
                    // Update the live player with the freshly compiled source
                    if !content.is_empty() {
                        if let Some(la) = &self.live_audio {
                            la.load_source(&content);
                        }
                    }
                    if !pcm.is_empty() {
                        if let Some(engine) = &mut self.audio {
                            engine.load(AudioBuffer { samples: pcm, sample_rate: 44100, channels: 2 });
                        }
                    }
                }
                WorkerMsg::CompileErr { doc_id, errors } => {
                    if let Some(doc) = self.docs.get_mut(doc_id) {
                        doc.compile_status = CompileStatus::Errors(errors);
                    }
                }
            }
        }
    }

    // ── auto-compile debounce ─────────────────────────────────────────────────

    fn check_auto_compile(&mut self, now: f64) {
        if !self.compile_opts.auto_compile { return; }
        let delay = self.settings.auto_compile_delay_ms as f64 / 1000.0;
        let ids: Vec<usize> = self.docs.docs().iter()
            .filter(|d| !matches!(d.compile_status, CompileStatus::Compiling)
                && d.last_edit_time.map(|t| now - t >= delay).unwrap_or(false))
            .map(|d| d.id)
            .collect();
        for id in ids {
            if let Some(doc) = self.docs.get_mut(id) { doc.last_edit_time = None; }
            self.trigger_compile(id);
        }
    }

    // ── MIDI ──────────────────────────────────────────────────────────────────

    fn poll_midi(&mut self) {
        for event in self.midi.poll_events() {
            match event {
                MidiEvent::NoteOn { note, .. } if (note as usize) < 128 => {
                    self.active_notes[note as usize] = true;
                }
                MidiEvent::NoteOff { note } if (note as usize) < 128 => {
                    self.active_notes[note as usize] = false;
                }
                _ => {}
            }
        }
    }

    // ── socket commands ───────────────────────────────────────────────────────

    fn poll_socket(&mut self, ctx: &Context) {
        if self.socket_rx.is_none() {
            return;
        }
        // Drain all pending commands into a local Vec to avoid borrow conflicts
        // between `socket_rx` and the mutable `self` methods below.
        let cmds: Vec<SocketCmd> = {
            let rx = self.socket_rx.as_ref().unwrap();
            let mut cmds = Vec::new();
            while let Ok(cmd) = rx.try_recv() {
                cmds.push(cmd);
            }
            cmds
        };
        for cmd in cmds {
            let resp = self.handle_socket_cmd(&cmd.req, ctx);
            let _ = cmd.resp_tx.send(resp);
        }
    }

    fn handle_socket_cmd(&mut self, req: &SocketRequest, ctx: &Context) -> serde_json::Value {
        match req {
            SocketRequest::Ping => json!({"ok": true}),

            SocketRequest::GetState => {
                let playing = self.audio.as_ref().map(|a| a.is_playing()).unwrap_or(false);
                let pos = self.audio.as_ref().map(|a| a.position_secs()).unwrap_or(0.0);
                let dur = self.audio.as_ref().map(|a| a.duration_secs()).unwrap_or(0.0);
                let errors = self.docs.active()
                    .and_then(|d| if let CompileStatus::Errors(ref e) = d.compile_status { Some(e.len()) } else { None })
                    .unwrap_or(0);
                json!({
                    "ok": true,
                    "is_playing": playing,
                    "position_secs": pos,
                    "duration_secs": dur,
                    "error_count": errors,
                })
            }

            SocketRequest::Play => {
                if let Some(audio) = &mut self.audio {
                    audio.play();
                    json!({"ok": true, "position_secs": audio.position_secs()})
                } else {
                    json!({"ok": false, "error": "no audio device"})
                }
            }

            SocketRequest::Stop => {
                if let Some(audio) = &mut self.audio { audio.stop(); }
                json!({"ok": true})
            }

            SocketRequest::GetErrors => {
                let errors = self.docs.active()
                    .and_then(|d| if let CompileStatus::Errors(ref e) = d.compile_status {
                        Some(e.clone())
                    } else {
                        None
                    })
                    .unwrap_or_default();
                let errs: Vec<serde_json::Value> = errors.iter().map(|e| json!({
                    "message": e.message,
                    "line": e.line,
                    "col": e.col,
                })).collect();
                json!({"ok": true, "errors": errs})
            }

            SocketRequest::GetPlayback => {
                if let Some(audio) = &self.audio {
                    json!({
                        "ok": true,
                        "is_playing": audio.is_playing(),
                        "position_secs": audio.position_secs(),
                        "duration_secs": audio.duration_secs(),
                        "volume": audio.volume(),
                    })
                } else {
                    json!({"ok": true, "is_playing": false, "position_secs": 0.0, "duration_secs": 0.0, "volume": 1.0})
                }
            }

            SocketRequest::OpenFile { path } => {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        self.docs.open_file(std::path::PathBuf::from(path), content);
                        if let Some(id) = self.docs.active_id() {
                            self.trigger_compile(id);
                        }
                        json!({"ok": true})
                    }
                    Err(e) => json!({"ok": false, "error": format!("read error: {e}")}),
                }
            }

            SocketRequest::Compile { .. } => {
                // compile is handled inline in the socket handler thread.
                json!({"ok": false, "error": "internal: compile dispatched unexpectedly"})
            }

            SocketRequest::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                json!({"ok": true})
            }
        }
    }


    // ── file I/O ──────────────────────────────────────────────────────────────

    fn open_file_dialog(&mut self) {
        let slot = make_slot::<Option<(PathBuf, String)>>();
        let clone = slot.clone();
        self.pending_open = Some(slot);
        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .add_filter("MML files", &["gwi", "mml", "muc", "txt"])
                .pick_file()
                .and_then(|p| std::fs::read_to_string(&p).ok().map(|c| (p, c)));
            *clone.lock().unwrap() = Some(result);
        });
    }

    fn save_file(&mut self, doc_id: usize) {
        let has_path = self.docs.get_mut(doc_id).and_then(|d| d.path.clone());
        match has_path {
            Some(path) => {
                if let Some(doc) = self.docs.get_mut(doc_id) {
                    let _ = std::fs::write(&path, &doc.content);
                    doc.modified = false;
                }
            }
            None => self.save_as_dialog(doc_id),
        }
    }

    fn save_as_dialog(&mut self, doc_id: usize) {
        let slot = make_slot::<Option<PathBuf>>();
        let clone = slot.clone();
        self.pending_save_as = Some(slot);
        let content = self.docs.get_mut(doc_id).map(|d| d.content.clone()).unwrap_or_default();
        std::thread::spawn(move || {
            let result = rfd::FileDialog::new().add_filter("MML files", &["gwi", "mml"]).save_file();
            if let Some(p) = &result { let _ = std::fs::write(p, &content); }
            *clone.lock().unwrap() = Some(result);
        });
    }

    fn poll_file_dialogs(&mut self) {
        if let Some(slot) = &self.pending_open {
            if slot.lock().unwrap().is_some() {
                let result = slot.lock().unwrap().take().flatten();
                self.pending_open = None;
                if let Some((path, content)) = result {
                    self.settings.add_recent_file(path.clone());
                    self.settings.save();
                    self.docs.open_file(path, content);
                }
            }
        }
        if let Some(slot) = &self.pending_save_as {
            if slot.lock().unwrap().is_some() {
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
}

impl eframe::App for MmlApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);

        self.poll_workers();
        self.poll_preview();
        self.poll_file_dialogs();
        self.handle_dropped_files(ctx);
        self.check_auto_compile(now);
        self.poll_midi();
        self.poll_socket(ctx);
        if let Some(engine) = &mut self.audio { engine.tick(); }

        // Keep repainting while busy.
        let busy = self.docs.docs().iter().any(|d| d.compile_status == CompileStatus::Compiling)
            || self.audio.as_ref().map(|e| e.is_playing()).unwrap_or(false)
            || self.pending_open.is_some()
            || self.pending_save_as.is_some()
            || self.midi.has_input();
        if busy { ctx.request_repaint(); }

        // ── keyboard shortcuts ────────────────────────────────────────────────
        let open    = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O));
        let save    = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S));
        let new     = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N));
        let compile = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::B));
        let comma   = ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma));
        let space   = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space));

        if open && self.pending_open.is_none() { self.open_file_dialog(); }
        if save { if let Some(id) = self.docs.active_id() { self.save_file(id); } }
        if new  { self.docs.open_untitled(); }
        if compile { if let Some(id) = self.docs.active_id() { self.trigger_compile(id); } }
        if comma { self.settings_open = true; }
        if space {
            if let Some(e) = &mut self.audio { if e.is_playing() { e.pause(); } else { e.play(); } }
        }

        // ── settings window ───────────────────────────────────────────────────
        let prev_theme = self.settings.theme.clone();
        let prev_font_size = self.settings.font_size;
        let midi_inputs  = self.midi.input_port_names.clone();
        let midi_outputs = self.midi.output_port_names.clone();
        crate::panels::settings_panel::show(ctx, &mut self.settings_open, &mut self.settings, &midi_inputs, &midi_outputs);
        if self.settings.theme != prev_theme {
            apply_theme(ctx, &self.settings);
        }
        if (self.settings.font_size - prev_font_size).abs() > f32::EPSILON {
            apply_font_size(ctx, self.settings.font_size);
        }
        // Apply MIDI port changes (reconnect when preferred port changed).
        reconnect_midi_if_needed(&mut self.midi, &self.settings);

        // ── instrument editors ────────────────────────────────────────────────
        crate::panels::fm_tone_editor::show(ctx, &mut self.fm_editor, &mut self.docs);
        crate::panels::envelope_editor::show(ctx, &mut self.envelope_editor, &mut self.docs);
        crate::panels::arpeggio_editor::show(ctx, &mut self.arpeggio_editor, &mut self.docs);
        crate::panels::sample_editor::show(ctx, &mut self.sample_editor, &mut self.docs);

        // ── menu bar ──────────────────────────────────────────────────────────
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New         Ctrl+N").clicked() { self.docs.open_untitled(); ui.close_menu(); }
                    if ui.button("Open…       Ctrl+O").clicked() && self.pending_open.is_none() { self.open_file_dialog(); ui.close_menu(); }
                    ui.separator();
                    if ui.button("Save        Ctrl+S").clicked() { if let Some(id) = self.docs.active_id() { self.save_file(id); } ui.close_menu(); }
                    if ui.button("Save As…").clicked() { if let Some(id) = self.docs.active_id() { self.save_as_dialog(id); } ui.close_menu(); }
                    if !self.settings.recent_files.is_empty() {
                        ui.separator();
                        ui.label("Recent");
                        let recents: Vec<PathBuf> = self.settings.recent_files.clone();
                        for path in recents {
                            let label = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string());
                            if ui.button(&label).clicked() {
                                if let Ok(content) = std::fs::read_to_string(&path) { self.docs.open_file(path, content); }
                                ui.close_menu();
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });

                ui.menu_button("Examples", |ui| {
                    const EXAMPLES: &[(&str, &str)] = &[
                        ("Hello World",        include_str!("../../examples/hello.gwi")),
                        ("FM Chord",           include_str!("../../examples/fm_chord.gwi")),
                        ("Loop Arp",           include_str!("../../examples/loop_arp.gwi")),
                        ("PSG Melody",         include_str!("../../examples/psg_melody.gwi")),
                    ];
                    for (label, content) in EXAMPLES {
                        if ui.button(*label).clicked() {
                            let id = self.docs.open_untitled();
                            if let Some(doc) = self.docs.get_mut(id) {
                                doc.content = content.to_string();
                            }
                            ui.close_menu();
                        }
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Settings…   Ctrl+,").clicked() { self.settings_open = true; ui.close_menu(); }
                });

                ui.menu_button("Instruments", |ui| {
                    if ui.button("FM Tone Editor").clicked() { self.fm_editor.open = true; ui.close_menu(); }
                    if ui.button("Envelope Editor").clicked() { self.envelope_editor.open = true; ui.close_menu(); }
                    if ui.button("Arpeggio Editor").clicked() { self.arpeggio_editor.open = true; ui.close_menu(); }
                    if ui.button("Sample/PCM Editor").clicked() { self.sample_editor.open = true; ui.close_menu(); }
                });

                ui.menu_button("Build", |ui| {
                    if ui.button("Compile     Ctrl+B").clicked() { if let Some(id) = self.docs.active_id() { self.trigger_compile(id); } ui.close_menu(); }
                    ui.separator();
                    if ui.button("Export…").clicked() {
                        if let Some(bytes) = self.docs.active().and_then(|d| d.compiled_bytes.clone()) {
                            let fmt = self.compile_opts.format.clone();
                            std::thread::spawn(move || {
                                if let Some(path) = rfd::FileDialog::new().add_filter("Output", &[fmt.as_str()]).save_file() {
                                    let _ = std::fs::write(&path, &bytes);
                                }
                            });
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("Playback", |ui| {
                    if ui.button("Play / Pause   Space").clicked() {
                        if let Some(e) = &mut self.audio { if e.is_playing() { e.pause(); } else { e.play(); } }
                        ui.close_menu();
                    }
                    if ui.button("Stop").clicked() { if let Some(e) = &mut self.audio { e.stop(); } ui.close_menu(); }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Errors").clicked()   { self.bottom_tab = BottomTab::Errors;   ui.close_menu(); }
                    if ui.button("Output").clicked()   { self.bottom_tab = BottomTab::Output;   ui.close_menu(); }
                    if ui.button("Waveform").clicked() { self.bottom_tab = BottomTab::Waveform; ui.close_menu(); }
                    if ui.button("MIDI").clicked()     { self.bottom_tab = BottomTab::Midi;     ui.close_menu(); }
                });
            });
        });

        // ── tab bar ───────────────────────────────────────────────────────────
        TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let ids: Vec<usize> = self.docs.docs().iter().map(|d| d.id).collect();
                let active_id = self.docs.active_id();
                let mut to_close = None;
                let mut to_activate = None;
                for id in &ids {
                    let id = *id;
                    let label = self.docs.docs().iter().find(|d| d.id == id).map(|d| d.tab_label()).unwrap_or_default();
                    if ui.selectable_label(active_id == Some(id), &label).clicked() { to_activate = Some(id); }
                    if ui.small_button("×").clicked() { to_close = Some(id); }
                    ui.separator();
                }
                if let Some(id) = to_activate { self.docs.set_active(id); }
                if let Some(id) = to_close    { self.docs.close(id); }
                if ui.small_button("+").on_hover_text("New file").clicked() { self.docs.open_untitled(); }
            });
        });

        // ── compile toolbar ───────────────────────────────────────────────────
        TopBottomPanel::top("compile_toolbar").show(ctx, |ui| {
            let clicked = compile_options::show(ui, &mut self.compile_opts);
            if clicked { if let Some(id) = self.docs.active_id() { self.trigger_compile(id); } }
        });

        // ── playback toolbar ──────────────────────────────────────────────────
        TopBottomPanel::top("playback_toolbar").show(ctx, |ui| {
            if let Some(engine) = &mut self.audio {
                crate::panels::playback::show(ui, engine);
            } else {
                ui.label(RichText::new("Audio unavailable").color(Color32::DARK_GRAY));
            }
        });

        // ── status bar ────────────────────────────────────────────────────────
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(doc) = self.docs.active() {
                    let file = doc.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "untitled".to_string());
                    ui.label(&file);
                    ui.separator();
                    match &doc.compile_status {
                        CompileStatus::Idle      => { ui.label(RichText::new("idle").color(Color32::GRAY)); }
                        CompileStatus::Compiling => { ui.label(RichText::new("⟳ compiling…").color(Color32::YELLOW)); }
                        CompileStatus::Ok { warnings } => {
                            let msg = if *warnings > 0 { format!("✓ ok ({warnings} warnings)") } else { "✓ ok".to_string() };
                            ui.label(RichText::new(msg).color(Color32::GREEN));
                        }
                        CompileStatus::Errors(errs) => {
                            ui.label(RichText::new(format!("✗ {} error(s)", errs.len())).color(Color32::LIGHT_RED));
                        }
                    }
                } else {
                    ui.label(RichText::new("No file open — Ctrl+O to open, Ctrl+N for new").color(Color32::GRAY));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.midi.has_input() {
                        ui.label(RichText::new("● MIDI").color(Color32::GREEN));
                    }
                });
            });
        });

        // ── bottom panel ──────────────────────────────────────────────────────
        TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Errors,   "Errors");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Output,   "Output");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Waveform, "Waveform");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Midi,     "MIDI");
                });
                ui.separator();

                match self.bottom_tab {
                    BottomTab::Errors => {
                        let errors = self.docs.active()
                            .and_then(|d| if let CompileStatus::Errors(ref e) = d.compile_status { Some(e.clone()) } else { None })
                            .unwrap_or_default();
                        if let Some(line) = crate::panels::error_list::show(ui, &errors) {
                            if let Some(doc) = self.docs.active_mut() {
                                doc.jump_to_line = Some(line);
                            }
                        }
                    }
                    BottomTab::Output => {
                        let preview: Option<String> = self.docs.active()
                            .and_then(|d| d.compiled_bytes.as_ref())
                            .map(|bytes| bytes.iter().take(256).enumerate()
                                .map(|(i, b)| if i % 16 == 15 { format!("{b:02X}\n") } else { format!("{b:02X} ") })
                                .collect());
                        if let Some(mut text) = preview {
                            egui::ScrollArea::vertical().id_salt("output_scroll").auto_shrink([false, false]).show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut text).font(egui::TextStyle::Monospace).interactive(false).desired_width(f32::INFINITY));
                            });
                        } else {
                            ui.label(RichText::new("No compiled output yet.").color(Color32::GRAY));
                        }
                    }
                    BottomTab::Waveform => {
                        let scope: Vec<f32> = self.audio.as_ref()
                            .map(|e| e.scope_samples(2048))
                            .unwrap_or_default();
                        crate::panels::waveform::show(ui, &scope);
                    }
                    BottomTab::Midi => {
                        self.show_midi_panel(ui);
                    }
                }
            });

        // ── central panel (editor) ────────────────────────────────────────────
        CentralPanel::default().show(ctx, |ui| {
            if self.docs.has_any() {
                if let Some(id) = self.docs.active_id() {
                    let doc = self.docs.get_mut(id).unwrap();
                    let jump = doc.jump_to_line.take();
                    let changed = crate::editor::show(ui, &mut doc.content, jump);
                    if changed { doc.mark_edited(now); }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Open a file (Ctrl+O) or create a new one (Ctrl+N)").color(Color32::GRAY));
                });
            }
        });
    }
}

// ── MIDI panel (inline, to avoid borrow conflicts) ────────────────────────────

impl MmlApp {
    /// Drain preview PCM from background threads and start playback if the note is still held.
    fn poll_preview(&mut self) {
        while let Ok((note, pcm)) = self.preview_rx.try_recv() {
            if self.keyboard_held == Some(note) {
                if let Some(engine) = &mut self.audio {
                    engine.play_preview(pcm);
                }
            }
        }
    }

    fn show_midi_panel(&mut self, ui: &mut egui::Ui) {
        // Port selectors
        ui.horizontal(|ui| {
            ui.label("Input:");
            let in_label = self.midi.active_input
                .and_then(|i| self.midi.input_port_names.get(i).cloned())
                .unwrap_or_else(|| "(none)".to_string());
            egui::ComboBox::from_id_salt("midi_in_panel")
                .selected_text(&in_label)
                .width(180.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.midi.active_input.is_none(), "(none)").clicked() {
                        self.midi.disconnect_input();
                    }
                    let names = self.midi.input_port_names.clone();
                    for (i, name) in names.iter().enumerate() {
                        let sel = self.midi.active_input == Some(i);
                        if ui.selectable_label(sel, name).clicked() {
                            self.midi.connect_input(i);
                            self.settings.preferred_midi_input = Some(name.clone());
                        }
                    }
                });

            ui.separator();

            ui.label("Output:");
            let out_label = self.midi.active_output
                .and_then(|i| self.midi.output_port_names.get(i).cloned())
                .unwrap_or_else(|| "(none)".to_string());
            egui::ComboBox::from_id_salt("midi_out_panel")
                .selected_text(&out_label)
                .width(180.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.midi.active_output.is_none(), "(none)").clicked() {
                        self.midi.disconnect_output();
                    }
                    let names = self.midi.output_port_names.clone();
                    for (i, name) in names.iter().enumerate() {
                        let sel = self.midi.active_output == Some(i);
                        if ui.selectable_label(sel, name).clicked() {
                            self.midi.connect_output(i);
                            self.settings.preferred_midi_output = Some(name.clone());
                        }
                    }
                });

            ui.separator();

            if ui.button("⟳ Refresh").clicked() {
                self.midi.refresh_ports();
            }
        });

        ui.separator();

        // Preview channel selector
        let channels = compiler::detect_channels(
            self.docs.active().map(|d| d.content.as_str()).unwrap_or(""),
        );
        if !channels.is_empty() {
            if self.keyboard_preview_channel.is_empty()
                || !channels.contains(&self.keyboard_preview_channel)
            {
                self.keyboard_preview_channel = channels[0].clone();
            }
            ui.horizontal(|ui| {
                ui.label("Preview channel:");
                egui::ComboBox::from_id_salt("preview_channel_sel")
                    .selected_text(&self.keyboard_preview_channel)
                    .show_ui(ui, |ui| {
                        for ch in &channels {
                            ui.selectable_value(
                                &mut self.keyboard_preview_channel,
                                ch.clone(),
                                ch,
                            );
                        }
                    });
            });
        }

        // On-screen keyboard
        let (note_on, note_off) = crate::panels::midi_keyboard::show(
            ui,
            &self.active_notes,
            &mut self.keyboard_held,
        );
        if let Some(note) = note_on {
            self.midi.send_note_on(0, note, 100);
            if (note as usize) < 128 { self.active_notes[note as usize] = true; }
            // Direct chip-register note-on via LiveAudioEngine (zero compile latency)
            if !self.keyboard_preview_channel.is_empty() {
                if let Some(la) = &self.live_audio {
                    la.note_on(&self.keyboard_preview_channel, note, 100);
                }
            }
        }
        if let Some(note) = note_off {
            self.midi.send_note_off(0, note);
            if (note as usize) < 128 { self.active_notes[note as usize] = false; }
            if let Some(la) = &self.live_audio {
                la.note_off(&self.keyboard_preview_channel);
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_slot<T>() -> std::sync::Arc<std::sync::Mutex<Option<T>>> {
    std::sync::Arc::new(std::sync::Mutex::new(None))
}

fn render_pcm(vgm_bytes: &[u8]) -> Vec<f32> {
    render_pcm_pub(vgm_bytes)
}

/// Public wrapper used by `socket.rs` for headless audio loading.
pub fn render_pcm_pub(vgm_bytes: &[u8]) -> Vec<f32> {
    let mut player = VgmPlayer::new();
    if player.load(vgm_bytes).is_err() { return Vec::new(); }
    player.init_chips_from_header();
    player.render_to_pcm(44100).unwrap_or_default()
}

fn apply_theme(ctx: &Context, settings: &Settings) {
    match settings.theme {
        Theme::Dark  => ctx.set_visuals(egui::Visuals::dark()),
        Theme::Light => ctx.set_visuals(egui::Visuals::light()),
    }
}

fn apply_font_size(ctx: &Context, size: f32) {
    let mut style = (*ctx.style()).clone();
    for text_style in [egui::TextStyle::Body, egui::TextStyle::Button, egui::TextStyle::Monospace] {
        if let Some(font_id) = style.text_styles.get_mut(&text_style) {
            font_id.size = size;
        }
    }
    ctx.set_style(style);
}

fn reconnect_midi_if_needed(midi: &mut MidiManager, settings: &Settings) {
    // If preferred input changed and doesn't match what's connected, reconnect.
    let desired_in = settings.preferred_midi_input.as_deref();
    let current_in = midi.active_input.and_then(|i| midi.input_port_names.get(i).map(|s| s.as_str()));
    if desired_in != current_in {
        if let Some(name) = desired_in {
            let names = midi.input_port_names.clone();
            if let Some(idx) = names.iter().position(|n| n == name) {
                midi.connect_input(idx);
            }
        } else {
            midi.disconnect_input();
        }
    }

    let desired_out = settings.preferred_midi_output.as_deref();
    let current_out = midi.active_output.and_then(|i| midi.output_port_names.get(i).map(|s| s.as_str()));
    if desired_out != current_out {
        if let Some(name) = desired_out {
            let names = midi.output_port_names.clone();
            if let Some(idx) = names.iter().position(|n| n == name) {
                midi.connect_output(idx);
            }
        } else {
            midi.disconnect_output();
        }
    }
}
