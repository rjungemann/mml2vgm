//! Smoke test suite for egui-app socket interface.
//!
//! Spawns `egui-app --socket --headless --socket-port <PORT>`, exercises the
//! JSON socket API, and then sends `quit` to shut the process down cleanly.
//!
//! Run with:
//!   cargo test --test smoke -- --test-threads=1
//!
//! Or via Justfile:
//!   just egui-smoke

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PORT: u16 = 17878;

// ── server fixture ────────────────────────────────────────────────────────────

struct Server {
    child: Child,
}

impl Server {
    fn spawn() -> Self {
        let bin = env!("CARGO_BIN_EXE_egui-app");
        let child = Command::new(bin)
            .args([
                "--socket",
                "--headless",
                "--socket-port",
                &PORT.to_string(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit()) // keep stderr for debugging
            .spawn()
            .expect("Failed to spawn egui-app binary");
        Self { child }
    }

    /// Poll until the socket is accepting connections (up to 15 s).
    fn wait_for_ready(&self) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if TcpStream::connect(format!("127.0.0.1:{PORT}")).is_ok() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "egui-app socket did not become ready within 15 seconds"
            );
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Send a JSON command string and return the parsed JSON response.
///
/// Opens a new TCP connection per call so tests are independent.
fn send(cmd_json: &str) -> serde_json::Value {
    let mut stream =
        TcpStream::connect(format!("127.0.0.1:{PORT}")).expect("connect to socket server");
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .ok();

    writeln!(stream, "{cmd_json}").expect("send command");

    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut line = String::new();
    reader.read_line(&mut line).expect("read response");

    serde_json::from_str(line.trim()).expect("invalid JSON response from server")
}

// ── valid MML fixture ─────────────────────────────────────────────────────────

const VALID_MML: &str = include_str!("fixtures/valid.gwi");

/// Minimal invalid MML — triggers "Expected '}'" parse error.
///
/// A `{` that is NOT on its own line and NOT preceded by `'` passes through the
/// preprocessor unchanged, reaches the parser as a `LeftBrace` token, and
/// causes a parse error when no matching `}` is found.
const INVALID_MML: &str = "{ this block is never closed";

// ── smoke tests ───────────────────────────────────────────────────────────────

/// Golden path: ping → compile valid → errors=0 → bytes>0 → compile invalid → errors>0 → quit.
#[test]
fn smoke() {
    let server = Server::spawn();
    server.wait_for_ready();

    // 1. Ping
    let resp = send(r#"{"cmd":"ping"}"#);
    assert_eq!(resp["ok"], true, "ping failed: {resp}");

    // 2. Compile valid MML — expect no errors and non-empty bytes.
    let compile_cmd = serde_json::json!({
        "cmd": "compile",
        "content": VALID_MML,
        "format": "vgm"
    });
    let resp = send(&compile_cmd.to_string());
    assert_eq!(resp["ok"], true, "compile (valid) failed: {resp}");
    let errors = resp["errors"].as_array().expect("errors should be array");
    assert!(
        errors.is_empty(),
        "expected zero errors for valid MML, got: {errors:?}"
    );
    let bytes_len = resp["bytes_len"].as_u64().unwrap_or(0);
    assert!(
        bytes_len > 0,
        "expected non-empty VGM bytes, got bytes_len={bytes_len}"
    );

    // 3. get_errors should return empty list (no active document in headless mode,
    //    but the command should still succeed).
    let resp = send(r#"{"cmd":"get_errors"}"#);
    assert_eq!(resp["ok"], true, "get_errors failed: {resp}");

    // 4. get_state
    let resp = send(r#"{"cmd":"get_state"}"#);
    assert_eq!(resp["ok"], true, "get_state failed: {resp}");

    // 5. get_playback
    let resp = send(r#"{"cmd":"get_playback"}"#);
    assert_eq!(resp["ok"], true, "get_playback failed: {resp}");
    let _ = resp["is_playing"].as_bool().expect("is_playing field missing");

    // 6. Compile invalid MML — expect errors non-empty.
    let invalid_cmd = serde_json::json!({
        "cmd": "compile",
        "content": INVALID_MML,
        "format": "vgm"
    });
    let resp = send(&invalid_cmd.to_string());
    assert_eq!(resp["ok"], true, "compile (invalid) failed: {resp}");
    let errors = resp["errors"].as_array().expect("errors should be array");
    assert!(
        !errors.is_empty(),
        "expected at least one error for invalid MML, got none"
    );

    // 7. Quit — server should respond ok and exit cleanly.
    let resp = send(r#"{"cmd":"quit"}"#);
    assert_eq!(resp["ok"], true, "quit failed: {resp}");

    // Give the process up to 2 s to exit.
    let mut child = std::mem::ManuallyDrop::new(server);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "egui-app did not exit successfully: {status}");
                return;
            }
            Ok(None) => {}
            Err(e) => panic!("wait error: {e}"),
        }
        assert!(
            Instant::now() < deadline,
            "egui-app did not exit within 2 s after quit"
        );
        thread::sleep(Duration::from_millis(50));
    }
}
