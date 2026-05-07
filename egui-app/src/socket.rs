//! TCP socket interface for the egui-app.
//!
//! When launched with `--socket [--socket-port N]`, the app starts a
//! newline-delimited JSON request/response server on `127.0.0.1:<port>` (default
//! 7878).  Each TCP connection handles one or more request–response pairs; the
//! connection is closed by the remote end or when the `quit` command is received.
//!
//! Protocol
//! --------
//! Each line sent by the client is a JSON object with a `"cmd"` field.
//! The server responds with a JSON object on a single line, always containing `"ok"`.
//!
//! Example session:
//! ```text
//! → {"cmd":"ping"}
//! ← {"ok":true}
//! → {"cmd":"compile","content":"...","format":"vgm"}
//! ← {"ok":true,"errors":[],"bytes_len":1234}
//! → {"cmd":"get_errors"}
//! ← {"ok":true,"errors":[]}
//! → {"cmd":"quit"}
//! ← {"ok":true}
//! ```

use crate::compiler;
use crate::document::CompileError;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, SyncSender};
use std::time::Duration;

// ── request types ─────────────────────────────────────────────────────────────

/// A command received from a socket client.
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum SocketRequest {
    /// Check that the server is alive.
    Ping,
    /// Return general app state (compile status, audio state).
    GetState,
    /// Compile MML source text and return errors / byte count.
    Compile {
        content: String,
        /// Output format: "vgm", "xgm", "xgm2", or "zgm". Defaults to "vgm".
        #[serde(default)]
        format: Option<String>,
    },
    /// Start or resume audio playback.
    Play,
    /// Stop audio playback.
    Stop,
    /// Return the errors from the last compile.
    GetErrors,
    /// Return current playback position and duration.
    GetPlayback,
    /// Open a file from `path` and set it as the active document.
    OpenFile { path: String },
    /// Shut down the application.
    Quit,
}

// ── internal dispatch types ───────────────────────────────────────────────────

/// A socket command dispatched to the main thread / headless loop.
///
/// The `resp_tx` channel carries the JSON response back to the socket handler.
pub struct SocketCmd {
    pub req: SocketRequest,
    pub resp_tx: SyncSender<Value>,
}

// ── server ────────────────────────────────────────────────────────────────────

/// Start the socket server in a background thread.
///
/// Incoming commands are dispatched through `cmd_tx`; the handler thread waits
/// for a response on the embedded `SyncSender<Value>`.
pub fn run(port: u16, cmd_tx: mpsc::Sender<SocketCmd>) {
    std::thread::spawn(move || {
        let addr = format!("127.0.0.1:{port}");
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                log::error!("Socket server: bind on {addr} failed: {e}");
                return;
            }
        };
        log::info!("Socket server listening on {addr}");
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let tx = cmd_tx.clone();
                    std::thread::spawn(move || handle_connection(s, tx));
                }
                Err(e) => log::warn!("Socket accept error: {e}"),
            }
        }
    });
}

// ── connection handler ────────────────────────────────────────────────────────

fn handle_connection(stream: std::net::TcpStream, cmd_tx: mpsc::Sender<SocketCmd>) {
    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Socket: try_clone failed: {e}");
            return;
        }
    };
    let mut writer = stream;
    let reader = BufReader::new(reader_stream);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse the request once. Route `compile` directly to avoid blocking
        // the main thread with compilation work.
        let resp = match serde_json::from_str::<SocketRequest>(line) {
            Ok(SocketRequest::Compile { content, format }) => {
                compile_inline(&content, format.as_deref())
            }
            Ok(req) => dispatch_request(req, &cmd_tx),
            Err(e) => json!({"ok": false, "error": format!("parse error: {e}")}),
        };

        let resp_str = resp.to_string();
        if writeln!(writer, "{resp_str}").is_err() {
            break;
        }
    }
}

/// Handle `compile` directly in the socket handler thread (non-blocking for main).
fn compile_inline(content: &str, format: Option<&str>) -> Value {
    let fmt = format.unwrap_or("vgm");
    match compiler::compile_content(content, fmt) {
        Ok(out) => {
            json!({
                "ok": true,
                "errors": serde_json::Value::Array(vec![]),
                "warnings": out.warnings,
                "bytes_len": out.bytes.len(),
            })
        }
        Err(errors) => {
            let errs: Vec<Value> = errors.iter().map(|e| serialize_error(e)).collect();
            json!({
                "ok": true,
                "errors": errs,
                "bytes_len": 0,
            })
        }
    }
}

/// Dispatch a pre-parsed request through the channel to the main thread.
fn dispatch_request(req: SocketRequest, cmd_tx: &mpsc::Sender<SocketCmd>) -> Value {
    let (resp_tx, resp_rx) = mpsc::sync_channel(1);
    if cmd_tx.send(SocketCmd { req, resp_tx }).is_err() {
        return json!({"ok": false, "error": "server shutting down"});
    }
    match resp_rx.recv_timeout(Duration::from_secs(30)) {
        Ok(v) => v,
        Err(_) => json!({"ok": false, "error": "response timeout"}),
    }
}

// ── headless runtime ──────────────────────────────────────────────────────────

/// State maintained by the headless runtime (no GUI).
pub struct HeadlessState {
    pub audio: Option<crate::audio::AudioEngine>,
    /// Errors from the last socket-initiated compile.
    pub last_errors: Vec<CompileError>,
    /// Bytes from the last socket-initiated compile.
    pub last_bytes: Option<Vec<u8>>,
}

impl HeadlessState {
    pub fn new() -> Self {
        Self {
            audio: crate::audio::AudioEngine::new(),
            last_errors: Vec::new(),
            last_bytes: None,
        }
    }
}

/// Process one `SocketCmd` in the headless runtime.
///
/// Returns `true` when the app should quit.
pub fn handle_headless_cmd(cmd: SocketCmd, state: &mut HeadlessState) -> bool {
    let resp = match &cmd.req {
        SocketRequest::Ping => json!({"ok": true}),

        SocketRequest::GetState => {
            let playing = state.audio.as_ref().map(|a| a.is_playing()).unwrap_or(false);
            let pos = state.audio.as_ref().map(|a| a.position_secs()).unwrap_or(0.0);
            let dur = state.audio.as_ref().map(|a| a.duration_secs()).unwrap_or(0.0);
            let error_count = state.last_errors.len();
            json!({
                "ok": true,
                "is_playing": playing,
                "position_secs": pos,
                "duration_secs": dur,
                "error_count": error_count,
                "has_bytes": state.last_bytes.is_some(),
            })
        }

        SocketRequest::Play => {
            if let Some(audio) = &mut state.audio {
                if let Some(bytes) = &state.last_bytes {
                    // Reload PCM if not already loaded (audio engine may have lost it).
                    if !audio.has_buffer() {
                        let pcm = crate::app::render_pcm_pub(bytes);
                        if !pcm.is_empty() {
                            audio.load(crate::audio::AudioBuffer {
                                samples: pcm,
                                sample_rate: 44100,
                                channels: 2,
                            });
                        }
                    }
                }
                audio.play();
                let pos = audio.position_secs();
                json!({"ok": true, "position_secs": pos})
            } else {
                json!({"ok": false, "error": "no audio device"})
            }
        }

        SocketRequest::Stop => {
            if let Some(audio) = &mut state.audio {
                audio.stop();
            }
            json!({"ok": true})
        }

        SocketRequest::GetErrors => {
            let errs: Vec<Value> = state.last_errors.iter().map(|e| serialize_error(e)).collect();
            json!({"ok": true, "errors": errs})
        }

        SocketRequest::GetPlayback => {
            if let Some(audio) = &state.audio {
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
                    // Compile immediately and store result.
                    match compiler::compile_content(&content, "vgm") {
                        Ok(out) => {
                            let pcm = crate::app::render_pcm_pub(&out.bytes);
                            if let Some(audio) = &mut state.audio {
                                if !pcm.is_empty() {
                                    audio.load(crate::audio::AudioBuffer {
                                        samples: pcm,
                                        sample_rate: 44100,
                                        channels: 2,
                                    });
                                }
                            }
                            state.last_errors = Vec::new();
                            state.last_bytes = Some(out.bytes);
                            json!({"ok": true})
                        }
                        Err(errors) => {
                            state.last_errors = errors.clone();
                            state.last_bytes = None;
                            let errs: Vec<Value> = errors.iter().map(|e| serialize_error(e)).collect();
                            json!({"ok": true, "errors": errs})
                        }
                    }
                }
                Err(e) => json!({"ok": false, "error": format!("read error: {e}")}),
            }
        }

        SocketRequest::Compile { .. } => {
            // compile is routed to compile_inline() before reaching handle_headless_cmd.
            json!({"ok": false, "error": "internal: compile should be handled inline"})
        }

        SocketRequest::Quit => {
            let _ = cmd.resp_tx.send(json!({"ok": true}));
            return true;
        }
    };

    let _ = cmd.resp_tx.send(resp);
    false
}

/// Run the headless event loop: start socket, process commands until quit.
pub fn run_headless(port: u16) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<SocketCmd>();
    run(port, cmd_tx);

    let mut state = HeadlessState::new();
    log::info!("Headless mode active — socket on port {port}");

    loop {
        match cmd_rx.recv_timeout(Duration::from_millis(16)) {
            Ok(cmd) => {
                if handle_headless_cmd(cmd, &mut state) {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Tick audio engine to handle loop restarts.
                if let Some(audio) = &mut state.audio {
                    audio.tick();
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    log::info!("Headless mode exiting");
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn serialize_error(e: &CompileError) -> Value {
    json!({
        "message": e.message,
        "line": e.line,
        "col": e.col,
    })
}
