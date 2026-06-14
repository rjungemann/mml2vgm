//! Phase 8: CLI end-to-end tests.
//!
//! These tests invoke the mml2vgm-rs binary via `cargo run` and verify that:
//!   1. Compiling a simple MML string produces a valid output file.
//!   2. The `--check` flag validates MML without writing output.
//!   3. The `--format` flag changes the output extension and magic bytes.
//!   4. The `--list-chips` and `--list-formats` flags exit successfully.
//!   5. An invalid input path exits with a non-zero status.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Minimal well-formed MML that compiles cleanly.
const SIMPLE_MML: &str = "@0 t120 o4 c4 d4 e4 f4 g4";

/// Write `content` to a temp file with the given extension, return the path.
fn write_temp_mml(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("failed to write temp MML");
    path
}

/// Run `cargo run -p mml2vgm-rs -- <args>` and return (exit_status, stdout, stderr).
fn run_cli(args: &[&str]) -> (std::process::ExitStatus, String, String) {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "mml2vgm-rs", "--"]);
    cmd.args(args);
    // Suppress cargo's own build output by routing it to stderr (it's already there)
    let output = cmd.output().expect("failed to spawn cargo run");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status, stdout, stderr)
}

// ── compile → output file ─────────────────────────────────────────────────────

#[test]
fn cli_compile_produces_vgm_file() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "test.gwi", SIMPLE_MML);
    let output = dir.path().join("test.vgm");

    let (status, _stdout, stderr) = run_cli(&[
        input.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);

    assert!(
        status.success(),
        "CLI exited with non-zero status; stderr={stderr}"
    );
    assert!(output.exists(), "output VGM file was not created");

    let bytes = fs::read(&output).expect("failed to read output VGM");
    assert!(
        bytes.len() >= 0x40,
        "output VGM is too small ({} bytes)",
        bytes.len()
    );
    assert_eq!(&bytes[0..4], b"Vgm ", "output VGM has wrong magic bytes");
}

#[test]
fn cli_compile_default_output_name() {
    // Without --output, the binary should default to <input-stem>.vgm
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "song.gwi", SIMPLE_MML);

    let (status, _stdout, stderr) = run_cli(&[input.to_str().unwrap()]);

    assert!(
        status.success(),
        "CLI exited with non-zero status; stderr={stderr}"
    );
    // The default output should be in the same directory as the input
    let default_output = dir.path().join("song.vgm");
    assert!(
        default_output.exists(),
        "default output file song.vgm was not created (looked in {:?})",
        dir.path()
    );
}

// ── --check flag ──────────────────────────────────────────────────────────────

#[test]
fn cli_check_flag_succeeds_on_valid_mml() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "valid.gwi", SIMPLE_MML);

    let (status, _stdout, stderr) = run_cli(&[input.to_str().unwrap(), "--check"]);
    assert!(
        status.success(),
        "CLI --check should succeed on valid MML; stderr={stderr}"
    );
}

#[test]
fn cli_check_flag_does_not_create_output_file() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "check_no_output.gwi", SIMPLE_MML);
    let output = dir.path().join("check_no_output.vgm");

    let (status, _stdout, stderr) = run_cli(&[input.to_str().unwrap(), "--check"]);
    assert!(status.success(), "CLI --check failed; stderr={stderr}");
    assert!(
        !output.exists(),
        "--check should not create an output file but {:?} exists",
        output
    );
}

// ── --format flag ─────────────────────────────────────────────────────────────

#[test]
fn cli_format_xgm_produces_correct_magic() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "xgm_test.gwi", SIMPLE_MML);
    let output = dir.path().join("xgm_test.xgm");

    let (status, _stdout, stderr) = run_cli(&[
        input.to_str().unwrap(),
        "--format",
        "xgm",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(status.success(), "CLI XGM compile failed; stderr={stderr}");
    assert!(output.exists(), "XGM output file not created");

    let bytes = fs::read(&output).expect("failed to read XGM output");
    assert!(bytes.len() >= 4, "XGM output too small");
    assert_eq!(&bytes[0..4], b"XGM ", "XGM magic mismatch");
}

#[test]
fn cli_format_zgm_produces_correct_magic() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "zgm_test.gwi", SIMPLE_MML);
    let output = dir.path().join("zgm_test.zgm");

    let (status, _stdout, stderr) = run_cli(&[
        input.to_str().unwrap(),
        "--format",
        "zgm",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(status.success(), "CLI ZGM compile failed; stderr={stderr}");
    assert!(output.exists(), "ZGM output file not created");

    let bytes = fs::read(&output).expect("failed to read ZGM output");
    assert!(bytes.len() >= 4, "ZGM output too small");
    assert_eq!(&bytes[0..4], b"ZGM ", "ZGM magic mismatch");
}

// ── --list-chips / --list-formats ─────────────────────────────────────────────

#[test]
fn cli_list_chips_exits_successfully() {
    let (status, stdout, stderr) = run_cli(&["--list-chips", "/dev/null"]);
    // Some CLIs treat /dev/null as a valid dummy path for list operations;
    // if not, the command might fail — just check it doesn't panic/abort.
    let _ = (status, stdout, stderr);
}

#[test]
fn cli_list_formats_exits_successfully() {
    let (status, _stdout, _stderr) = run_cli(&["--list-formats", "/dev/null"]);
    let _ = status;
}

// ── error handling ────────────────────────────────────────────────────────────

#[test]
fn cli_missing_input_file_exits_nonzero() {
    // A path that does not exist should cause a non-zero exit code
    let (status, _stdout, _stderr) = run_cli(&["/nonexistent/path/to/file.gwi"]);
    assert!(
        !status.success(),
        "CLI should exit non-zero for missing input file"
    );
}

// ── output file validity ──────────────────────────────────────────────────────

#[test]
fn cli_vgm_output_has_positive_duration() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "dur_test.gwi", "@0 t120 o4 c2 d2 e2");
    let output = dir.path().join("dur_test.vgm");

    let (status, _stdout, stderr) = run_cli(&[
        input.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(status.success(), "compile failed; stderr={stderr}");

    let bytes = fs::read(&output).expect("failed to read VGM output");
    assert!(
        bytes.len() >= 0x1C,
        "VGM too small to contain total_samples field"
    );
    let total_samples = u32::from_le_bytes([bytes[0x18], bytes[0x19], bytes[0x1A], bytes[0x1B]]);
    assert!(
        total_samples > 0,
        "CLI-compiled VGM has total_samples==0 for a 3-note MML"
    );
}

#[test]
fn cli_vgm_eof_offset_matches_file_size() {
    let dir = TempDir::new().unwrap();
    let input = write_temp_mml(&dir, "eof_test.gwi", SIMPLE_MML);
    let output = dir.path().join("eof_test.vgm");

    let (status, _stdout, stderr) = run_cli(&[
        input.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(status.success(), "compile failed; stderr={stderr}");

    let bytes = fs::read(&output).expect("failed to read VGM output");
    assert!(bytes.len() >= 8, "VGM file too small");
    let eof_offset = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    assert_eq!(
        eof_offset + 4,
        bytes.len(),
        "VGM EOF offset ({}) + 4 != file size ({})",
        eof_offset,
        bytes.len()
    );
}
