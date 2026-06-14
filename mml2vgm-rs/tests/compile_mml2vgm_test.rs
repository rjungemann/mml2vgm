//! Integration test: Compile reference MML files from mml2vgmTest/
//!
//! Run with: cargo test --test compile_mml2vgm_test -- --nocapture

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::{CompileOptions, OutputFormat};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Per-file compile timeout in seconds.
const TIMEOUT_PER_FILE_SECS: u64 = 10;
/// Total test timeout in seconds.
const TOTAL_TIMEOUT_SECS: u64 = 90;

/// VGM magic bytes.
const VGM_MAGIC: &[u8] = b"Vgm ";

fn compile_file(path: &PathBuf) -> (bool, String, usize, Duration) {
    let filename = path.file_name().unwrap().to_string_lossy().into_owned();

    let source = match std::fs::read(path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => return (false, format!("read error: {}", e), 0, Duration::ZERO),
    };

    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });

    let start = Instant::now();
    let result = compiler.compile_from_source(&source);
    let elapsed = start.elapsed();

    if elapsed.as_secs() >= TIMEOUT_PER_FILE_SECS {
        return (
            false,
            format!("TIMEOUT after {:.2}s", elapsed.as_secs_f64()),
            0,
            elapsed,
        );
    }

    match result {
        Ok(r) => (true, filename, r.data.len(), elapsed),
        Err(e) => (false, format!("{}: {}", filename, e), 0, elapsed),
    }
}

// ── Individual file tests ─────────────────────────────────────────────────────

#[test]
fn compile_c140sample() {
    let path = PathBuf::from("../mml2vgmTest/c140sample.gwi");
    if !path.exists() {
        eprintln!("Skipping: {:?} not found", path);
        return;
    }
    let (ok, msg, bytes, elapsed) = compile_file(&path);
    println!(
        "c140sample.gwi: {} bytes in {:.3}s",
        bytes,
        elapsed.as_secs_f64()
    );
    assert!(ok, "c140sample.gwi failed: {}", msg);
    assert!(bytes > 0, "c140sample.gwi produced empty output");
}

#[test]
fn compile_ay8910_test() {
    let path = PathBuf::from("../mml2vgmTest/testay8910/testAY38910.gwi");
    if !path.exists() {
        eprintln!("Skipping: {:?} not found", path);
        return;
    }
    let (ok, msg, bytes, elapsed) = compile_file(&path);
    println!(
        "testAY38910.gwi: {} bytes in {:.3}s",
        bytes,
        elapsed.as_secs_f64()
    );
    assert!(ok, "testAY38910.gwi failed: {}", msg);
    assert!(bytes > 0);
}

#[test]
fn compile_testcase3_pcm() {
    let path = PathBuf::from("../mml2vgmTest/testcase3/testPCM.gwi");
    if !path.exists() {
        eprintln!("Skipping: {:?} not found", path);
        return;
    }
    let (ok, msg, bytes, elapsed) = compile_file(&path);
    println!(
        "testcase3/testPCM.gwi: {} bytes in {:.3}s",
        bytes,
        elapsed.as_secs_f64()
    );
    assert!(ok, "testcase3/testPCM.gwi failed: {}", msg);
    assert!(bytes > 0);
}

#[test]
fn compile_testcase4_pcm() {
    let path = PathBuf::from("../mml2vgmTest/testcase4/testPCM.gwi");
    if !path.exists() {
        eprintln!("Skipping: {:?} not found", path);
        return;
    }
    let (ok, msg, bytes, elapsed) = compile_file(&path);
    println!(
        "testcase4/testPCM.gwi: {} bytes in {:.3}s",
        bytes,
        elapsed.as_secs_f64()
    );
    assert!(ok, "testcase4/testPCM.gwi failed: {}", msg);
    assert!(bytes > 0);
}

#[test]
fn compile_mml2vgm_subdir_sample() {
    let path = PathBuf::from("../mml2vgmTest/mml2vgm/c140sample.gwi");
    if !path.exists() {
        eprintln!("Skipping: {:?} not found", path);
        return;
    }
    let (ok, msg, bytes, elapsed) = compile_file(&path);
    println!(
        "mml2vgm/c140sample.gwi: {} bytes in {:.3}s",
        bytes,
        elapsed.as_secs_f64()
    );
    assert!(ok, "mml2vgm/c140sample.gwi failed: {}", msg);
    assert!(bytes > 0);
}

// ── VGM header validation ─────────────────────────────────────────────────────

#[test]
fn all_mml2vgm_test_files_have_valid_vgm_header() {
    let test_root = PathBuf::from("../mml2vgmTest");
    if !test_root.exists() {
        eprintln!("Skipping: mml2vgmTest directory not found");
        return;
    }

    let total_start = Instant::now();
    let candidates: Vec<PathBuf> = vec![
        test_root.join("c140sample.gwi"),
        test_root.join("testay8910/testAY38910.gwi"),
        test_root.join("testcase3/testPCM.gwi"),
        test_root.join("testcase4/testPCM.gwi"),
        test_root.join("mml2vgm/c140sample.gwi"),
    ];

    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });

    let mut failures = 0;
    for path in &candidates {
        if !path.exists() {
            continue;
        }
        if total_start.elapsed().as_secs() >= TOTAL_TIMEOUT_SECS {
            panic!("Total timeout reached after {}s", TOTAL_TIMEOUT_SECS);
        }

        let source =
            String::from_utf8_lossy(&std::fs::read(path).expect("read failed")).into_owned();
        let start = Instant::now();
        match compiler.compile_from_source(&source) {
            Ok(result) => {
                if start.elapsed().as_secs() >= TIMEOUT_PER_FILE_SECS {
                    eprintln!("TIMEOUT: {:?}", path);
                    failures += 1;
                } else if result.data.len() < 4 || &result.data[0..4] != VGM_MAGIC {
                    eprintln!("Bad VGM header: {:?}", path);
                    failures += 1;
                } else {
                    println!(
                        "✓ {:?} — {} bytes",
                        path.file_name().unwrap(),
                        result.data.len()
                    );
                }
            }
            Err(e) => {
                eprintln!("✗ {:?}: {}", path, e);
                failures += 1;
            }
        }
    }

    assert_eq!(failures, 0, "{} file(s) failed VGM header check", failures);
}

// ── All files compile within per-file and total timeouts ─────────────────────

#[test]
fn all_mml2vgm_test_files_compile_within_timeout() {
    let candidates: Vec<PathBuf> = vec![
        PathBuf::from("../mml2vgmTest/c140sample.gwi"),
        PathBuf::from("../mml2vgmTest/testay8910/testAY38910.gwi"),
        PathBuf::from("../mml2vgmTest/testcase3/testPCM.gwi"),
        PathBuf::from("../mml2vgmTest/testcase4/testPCM.gwi"),
        PathBuf::from("../mml2vgmTest/mml2vgm/c140sample.gwi"),
    ];

    let total_start = Instant::now();

    for path in &candidates {
        if !path.exists() {
            continue;
        }
        assert!(
            total_start.elapsed().as_secs() < TOTAL_TIMEOUT_SECS,
            "Total timeout reached"
        );

        let (ok, msg, _bytes, elapsed) = compile_file(path);
        assert!(
            elapsed.as_secs() < TIMEOUT_PER_FILE_SECS,
            "{:?} exceeded per-file timeout: {:.2}s",
            path,
            elapsed.as_secs_f64()
        );
        assert!(ok, "compile failed for {:?}: {}", path, msg);
    }
}
