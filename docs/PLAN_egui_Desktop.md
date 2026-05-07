# Plan: Migrate from Tauri to Rust + egui Desktop App

## Overview

Replace the Tauri desktop app (`tauri-app/`) with a fully native Rust + egui desktop
application. The Tauri app is a thin shell around the browser-ide React frontend and runs
the MML compiler through WASM. The egui replacement will run the compiler natively, use
native audio output via `cpal`, and use native MIDI I/O via `midir` — eliminating all the
pain points that come from WASM sandboxing, Web Audio API latency, and unreliable Web MIDI
support in Chromium-based webviews.

---

## Why Replace Tauri

| Pain point | Tauri root cause | egui fix |
|---|---|---|
| MIDI input unreliable | Webview depends on Chrome's Web MIDI API, requires `--enable-web-midi` flag, often denied on macOS | `midir` crate talks directly to CoreMIDI / WinMM / ALSA |
| Audio latency | Web Audio API buffer sizes, WASM↔JS bridge copies | `cpal` ring-buffer callback, no copies, sub-10ms latency typical |
| WASM compilation overhead | Every compile goes through wasm-bindgen serialization | Call `mml2vgm-rs` lib directly, zero serialization |
| SharedArrayBuffer / COOP headers | Tauri must inject headers; fragile across Tauri versions | Not applicable — no WebWorker needed |
| MIDI output impossible | Web MIDI output still behind a flag on most platforms | `midir` output port works today on all targets |
| React + WASM build pipeline | Two toolchains (npm + cargo) must stay in sync | Single `cargo build`, no npm |
| Binary size | Tauri bundles a WebKit/Chromium runtime | egui is a ~2 MB static binary |

---

## Architecture

```
egui-app/               ← new crate (binary)
├── Cargo.toml
└── src/
    ├── main.rs         ← eframe entry point
    ├── app.rs          ← top-level MmlApp struct implementing eframe::App
    ├── editor.rs       ← code editor widget (egui_code_editor or custom)
    ├── panels/
    │   ├── error_list.rs
    │   ├── part_counter.rs
    │   ├── playback.rs
    │   ├── compile_options.rs
    │   ├── mixer.rs
    │   ├── waveform.rs
    │   ├── midi_keyboard.rs
    │   └── debug.rs
    ├── audio.rs        ← thin wrapper around mml2vgm-rs audio + cpal
    ├── midi.rs         ← midir input/output manager
    ├── compiler.rs     ← thin wrapper around mml2vgm-rs compile API
    ├── document.rs     ← multi-document / tab state
    └── settings.rs     ← persist settings to ~/.config/mml2vgm/settings.toml
```

The new app is a sibling crate in the workspace. The `mml2vgm-rs` library crate is a
direct Cargo dependency — no WASM, no IPC, no serialization boundary.

### Workspace layout after migration

```
mml2vgm/
├── mml2vgm-rs/        ← compiler + player library (unchanged)
├── mml2vgm-wasm/      ← still built for browser-ide (unchanged)
├── browser-ide/       ← web IDE (unchanged)
├── egui-app/          ← NEW: replaces tauri-app/
└── tauri-app/         ← keep until egui-app reaches feature parity, then delete
```

---

## Key Crates

| Purpose | Crate | Notes |
|---|---|---|
| GUI framework | `egui` + `eframe` | Immediate-mode; `eframe` handles OS window + OpenGL/Metal/DX12 |
| Extra widgets | `egui_extras` | Tables, date picker, image loading |
| Code editor | `egui_code_editor` | Syntax-highlighted editor widget; or roll a thin custom one |
| Syntax highlighting | `syntect` | If using a custom editor; `.sublime-syntax` grammar |
| Native file dialogs | `rfd` | Async-friendly; works on macOS, Windows, Linux |
| Audio output | `cpal` | Already planned in `mml2vgm-rs` Phase 5; reuse |
| MIDI I/O | `midir` | CoreMIDI / WinMM / ALSA; no browser flag required |
| MIDI parsing | `midly` | Parse raw MIDI bytes from input ports |
| Settings persistence | `serde` + `toml` | Same deps already in `mml2vgm-rs` |
| Icon/image loading | `image` | Load PNG icons into egui textures |
| Waveform drawing | egui custom paint | Draw waveform directly onto egui `Painter` canvas |
| Socket IPC | `serde_json` | Newline-delimited JSON protocol over `std::net::TcpListener` |

---

## Feature Parity Checklist

Features carried over from the browser-ide / Tauri app, mapped to implementation approach.

### Editor

- [ ] Multi-document tabs — `document.rs` holds a `Vec<Document>`, active index
- [ ] Syntax-highlighted code editor — `egui_code_editor` with custom `.gwi` grammar, or `syntect` fallback
- [ ] Cursor position display in status bar
- [ ] Undo/redo — `egui_code_editor` provides this; or maintain a `Vec<String>` snapshot stack
- [ ] Find/replace — egui overlay panel

### Compilation

- [ ] Compile on demand (Ctrl+B / button) — calls `mml2vgm_rs::compile()` directly on a rayon thread
- [ ] Auto-compile on change (debounced) — 500 ms timer, reset on each keystroke
- [ ] Chip selector — combo box populated from `mml2vgm_rs::supported_chips()`
- [ ] Format selector (VGM / XGM / XGM2 / ZGM) — combo box
- [ ] Error list panel — parse `Vec<MmlError>` from compile result, click to jump to line
- [ ] Compilation status in status bar

### Playback

- [ ] Play / Pause / Stop buttons — send commands to audio thread via `std::sync::mpsc`
- [ ] Seek bar — display sample position; drag to seek
- [ ] Loop toggle — pass flag to player
- [ ] Volume slider — scale output samples
- [ ] Waveform panel — draw `f32` samples from a lock-free ring buffer using egui `Painter`
- [ ] Part counter panel — display active parts per chip channel from player tick callback

### MIDI

- [ ] MIDI input port selector — list ports from `midir::MidiInput`; select and open
- [ ] MIDI keyboard panel — on-screen piano keyboard; click or use MIDI input to preview notes
- [ ] MIDI output — send compiled VGM events as MIDI to a selected output port
- [ ] Real-time MIDI preview — while editing, play note under cursor via MIDI out or chip emulator

### File Management

- [ ] Open file — `rfd::AsyncFileDialog`; filter `*.gwi *.mml *.muc`
- [ ] Save / Save As — `rfd::AsyncFileDialog` for save path
- [ ] Recent files list — persist to settings
- [ ] Drag-and-drop open — `eframe` exposes dropped files via `egui::RawInput::dropped_files`
- [ ] Export VGM/XGM/ZGM — save compiled bytes with native dialog

### Settings

- [ ] Theme selector (light / dark)
- [ ] Font size
- [ ] Default chip / format
- [ ] Audio output device selector (enumerate from cpal)
- [ ] MIDI input/output port preference (persist by name, re-connect on launch)
- [ ] Auto-compile toggle and debounce delay

### Misc

- [ ] Status bar (file name, cursor position, chip/format, compile status)
- [ ] Debug panel (raw VGM hex dump, register trace)
- [ ] Mixer panel (per-channel volume, mute, solo — feeds into chip emulator mixing)
- [ ] i18n — defer; English-only initially

### Testing / Automation

- [ ] Socket server (`socket.rs`) — newline-delimited JSON over TCP; gated on `--socket` flag
- [ ] `SocketCommand` dispatch from main thread via `mpsc`
- [ ] `get_state`, `compile`, `play`, `stop`, `get_errors`, `get_playback`, `open_file`, `quit` commands
- [ ] Smoke test binary/script suite — connects to socket, exercises golden-path scenarios
- [ ] CI job: start `egui-app --socket --headless`, run smoke tests, assert exit 0

---

## Audio Architecture

The audio thread is owned by `audio.rs` and runs independently of the egui render loop.

```
Main thread (egui)
  │  compile MML → VgmData
  │  mpsc::Sender<AudioCommand> ──────────────────────────────────┐
  │                                                               ▼
  │                                               Audio thread (cpal callback)
  │                                                 VgmPlayer::tick() → f32 samples
  │                                                 ring buffer write
  │  ring buffer read → waveform panel ◄─────────────────────────┘
  │  AtomicU64 sample_pos → seek bar ◄────────────────────────────┘
```

`AudioCommand` enum:
```rust
enum AudioCommand {
    Load(VgmData),
    Play,
    Pause,
    Stop,
    Seek(u64),       // sample index
    SetVolume(f32),
    SetLoop(bool),
}
```

The ring buffer for waveform display uses `ringbuf` crate (lock-free SPSC).

---

## MIDI Architecture

```
midir::MidiInput connection
  │  raw bytes → midly::LiveEvent::parse()
  │  NoteOn/NoteOff/CC → MidiEvent enum
  │  mpsc::Sender<MidiEvent>
  ▼
Main thread: update midi_keyboard panel highlight + trigger preview note
  │  if preview note: call chip emulator directly (no VGM compile needed)
  ▼
Audio thread: render preview note
```

MIDI output (for VGM-as-MIDI export):
- Walk the compiled VgmData command stream
- Translate register writes to MIDI note on/off + program change
- Stream to `midir::MidiOutput` connection in real time

---

## Socket Interface

A local TCP socket server embedded in `egui-app` exposes the running app's state to external processes. This makes headless smoke tests straightforward: start the app, connect to the socket, issue commands, assert on JSON responses.

```
egui-app (running)
  │
  ├── TcpListener on 127.0.0.1:7878 (configurable)
  │     accepts newline-delimited JSON requests
  │     handled on a dedicated thread; state access via Arc<Mutex<AppState>>
  │
  └── SocketCommand enum
        {"cmd": "ping"}                         → {"ok": true}
        {"cmd": "get_state"}                    → compile/playback/document state snapshot
        {"cmd": "compile", "content": "..."}    → trigger compile, return errors[]
        {"cmd": "play"}  / {"cmd": "stop"}      → send AudioCommand
        {"cmd": "get_errors"}                   → last Vec<MmlError>
        {"cmd": "get_playback"}                 → {position, duration, playing, loop}
        {"cmd": "open_file", "path": "..."}     → load file into DocumentStore
        {"cmd": "quit"}                         → graceful shutdown
```

`socket.rs` owns the listener loop. Each request is deserialized with `serde_json`, dispatched through a `mpsc::Sender<SocketCommand>` to the main thread (or handled directly with a read lock), and the response is serialized and written back on the same connection.

The socket server is only started when the `--socket` CLI flag is passed (or via `settings.toml`), so it adds zero overhead in normal use.

```
egui-app --socket [--socket-port 7878]
```

---

## Implementation Phases

### Phase 1: Skeleton (1 week)

Goal: empty window builds and runs.

- [ ] Add `egui-app` binary crate to workspace `Cargo.toml`
- [ ] `Cargo.toml` deps: `eframe`, `egui`, `egui_extras`, `rfd`, `serde`, `toml`, `log`, `env_logger`
- [ ] `main.rs`: `eframe::run_native(...)` with `MmlApp::default()`
- [ ] `app.rs`: stub `eframe::App` impl with empty `update()`
- [ ] Justfile target: `just egui-dev` → `cargo run -p egui-app`
- [ ] Verify builds on macOS

### Phase 2: Editor + Documents (1-2 weeks)

Goal: open a `.gwi` file and edit it.

- [ ] `document.rs`: `Document { id, path, content, modified, cursor }`, `DocumentStore`
- [ ] Tab bar rendered from `DocumentStore` — click to switch, × to close
- [ ] `editor.rs`: integrate `egui_code_editor`; wire content ↔ `DocumentStore`
- [ ] File open via `rfd` — load into new `Document`
- [ ] File save / Save As
- [ ] Drag-and-drop open
- [ ] Recent files (persist in settings)
- [ ] Status bar: file name, modified indicator, cursor pos

### Phase 3: Compilation (1 week)

Goal: compile MML and show errors.

- [ ] `compiler.rs`: spawn compile on `rayon::spawn` with `mml2vgm_rs::compile()`
- [ ] Return `CompileResult` to main thread via `mpsc`
- [ ] `compile_options.rs` panel: chip selector, format selector, auto-compile toggle
- [ ] `error_list.rs` panel: display `Vec<MmlError>`, click to jump to line in editor
- [ ] Status bar compile status indicator
- [ ] Debounced auto-compile (500 ms) when content changes

### Phase 4: Audio Playback (1-2 weeks)

Goal: play compiled VGM.

- [ ] `audio.rs`: spawn `cpal` output stream; `VgmPlayer` driven from callback
- [ ] `AudioCommand` channel from main thread
- [ ] `playback.rs` panel: Play / Pause / Stop / Loop buttons, seek bar, volume slider
- [ ] `waveform.rs` panel: read from ring buffer, draw onto `egui::Painter` using `Mesh` or `Shape::line`
- [ ] `part_counter.rs` panel: read active channel state from player via shared `Arc<Mutex<PlayerState>>`

### Phase 5: MIDI (1-2 weeks)

Goal: real MIDI input and keyboard preview.

- [ ] `midi.rs`: enumerate ports with `midir`; open selected input port; parse with `midly`
- [ ] `midi_keyboard.rs` panel: draw 2-octave piano keyboard; highlight active keys
- [ ] Click-to-preview: send note directly to audio thread chip emulator
- [ ] MIDI input preview: highlight key on panel + trigger preview note
- [ ] Settings: persist preferred MIDI port name; reconnect on startup

### Phase 6: Settings + Polish (1 week)

Goal: settings persistence and general polish.

- [ ] `settings.rs`: `Settings` struct, load/save `~/.config/mml2vgm/settings.toml`
- [ ] Settings panel: theme, font size, audio device, MIDI port, default chip/format
- [ ] Audio device selector from `cpal::available_hosts()` / device enumeration
- [ ] `mixer.rs` panel: per-channel volume/mute/solo sliders feeding chip emulator
- [ ] `debug.rs` panel: hex dump of last compiled VGM bytes
- [ ] Keyboard shortcuts (Ctrl+B compile, Ctrl+O open, Ctrl+S save, Space play/pause)
- [ ] App icon set

### Phase 8: Socket Interface (1 week)

Goal: external processes can interrogate and control the running app over a local TCP socket.

- [ ] `socket.rs`: `TcpListener` on `127.0.0.1:7878`; accept connections on a dedicated thread
- [ ] `SocketCommand` / `SocketResponse` enums; `serde_json` serialization
- [ ] `mpsc::Sender<SocketCommand>` from socket thread → main thread dispatch
- [ ] Implement `ping`, `get_state`, `compile`, `play`, `stop`, `get_errors`, `get_playback`, `open_file`, `quit`
- [ ] CLI flag `--socket` (+ optional `--socket-port <N>`) to enable; disabled by default
- [ ] Optional `--headless` flag: skip `eframe::run_native`, run only audio + socket loop (for CI)
- [ ] Justfile target: `just egui-socket` → `cargo run -p egui-app -- --socket`

### Phase 9: Smoke Test Suite (1 week)

Goal: automated end-to-end tests that start the app and verify core behaviour via the socket.

- [ ] `egui-app/tests/smoke/` directory (or a standalone `smoke-tests/` crate)
- [ ] Test harness: spawn `egui-app --socket --headless`, wait for `ping` to succeed (up to 5 s)
- [ ] Golden-path test: load `tests/fixtures/hello.gwi`, compile, assert zero errors, assert VGM bytes non-empty
- [ ] Playback test: `play` → poll `get_playback` until `position > 0`, then `stop`
- [ ] Error test: compile deliberately invalid MML, assert `errors` array non-empty
- [ ] Quit test: send `quit`, assert process exits with code 0 within 2 s
- [ ] Justfile target: `just smoke` → builds app then runs test suite; exits non-zero on failure
- [ ] CI: add `smoke` step after `test` step in GitHub Actions workflow

### Phase 7: Feature Freeze on Tauri + Migration (1 week)

- [ ] Mark `tauri-app/` as deprecated in README
- [ ] Update Justfile to build `egui-app` by default for desktop targets
- [ ] Remove `tauri-app/` once `egui-app` passes smoke tests
- [ ] Update `docs/Tauri_Desktop_Setup.md` → redirect to egui setup

---

## Dependencies (Cargo.toml excerpt)

```toml
[package]
name = "egui-app"
version = "0.1.0"
edition = "2021"

[dependencies]
mml2vgm-rs = { path = "../mml2vgm-rs" }

eframe = { version = "0.29", features = ["persistence"] }
egui = "0.29"
egui_extras = { version = "0.29", features = ["all_loaders"] }
egui_code_editor = "0.3"

rfd = "0.15"
midir = "0.10"
midly = "0.5"
cpal = "0.15"
ringbuf = "0.4"

serde = { version = "1", features = ["derive"] }
toml = "0.8"
log = "0.4"
env_logger = "0.11"
image = { version = "0.25", default-features = false, features = ["png"] }
serde_json = "1"
```

---

## Not in Scope (Deferred)

- i18n / Japanese UI — browser-ide's `i18nService` is deferred until core egui UI is stable
- GIMIC / SCCI real chip hardware — deferred (Windows-only, requires C FFI)
- Scripting panel — deferred (depends on scripting engine decisions)
- Lyrics panel — low priority; plain text editor widget is sufficient short-term

---

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `egui_code_editor` lacks features we need (bracket matching, multi-cursor) | Fall back to a simple `egui::TextEdit::multiline` + `syntect` for highlight-only pass; we can iterate |
| cpal audio latency on Linux (ALSA) | Test early; offer PulseAudio / PipeWire device selection in settings |
| midir CoreMIDI permissions on macOS | Add `NSMicrophoneUsageDescription` + `com.apple.security.cs.allow-unsigned-executable-memory` to entitlements if needed |
| egui immediate-mode re-render cost with large waveforms | Cap waveform draw at 512 points; only re-render when new audio data arrives (use `ctx.request_repaint_after`) |
| mml2vgm-rs compile API not yet stable | Phase 3 depends on Phase 4 (audio) work in mml2vgm-rs; can stub with dummy `CompileResult` and integrate later |

---

## Success Criteria

- App builds with `cargo build -p egui-app` on macOS, Linux, and Windows (CI)
- Can open a `.gwi` file, compile it, and hear audio output within 5 seconds of startup
- MIDI input device is selectable and preview notes play with < 15 ms perceived latency
- All panels from the browser-ide have functional equivalents
- `tauri-app/` directory is removed
- `just smoke` passes: app starts in headless+socket mode, golden-path compile+playback scenario exits 0
- Socket interface responds to all defined commands within 100 ms on localhost
