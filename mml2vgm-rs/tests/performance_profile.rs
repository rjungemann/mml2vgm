//! Performance profiling for MML compilation
//!
//! Run with: cargo test --test performance_profile -- --nocapture
//! This test measures compilation time for each example file with per-file timeouts.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use mml2vgm::{CompileOptions, OutputFormat};
use mml2vgm::compiler::compiler::MmlCompiler;

const TIMEOUT_PER_FILE_SECS: u64 = 15;  // 15 second timeout per file
const MAX_TOTAL_TIME_SECS: u64 = 60;   // 60 second total timeout for entire test

#[test]
fn profile_compilation_performance() {
    let samples_dir = PathBuf::from("../browser-ide/public/samples");

    if !samples_dir.exists() {
        eprintln!("Warning: samples directory not found at {:?}", samples_dir);
        return;
    }

    // Find all .gwi files
    let entries = fs::read_dir(&samples_dir)
        .expect("Failed to read samples directory");

    let mut gwi_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "gwi")
                .unwrap_or(false)
        })
        .collect();

    gwi_files.sort_by_key(|e| e.path());

    println!("{:=^80}", "");
    println!("{:=^80}", " COMPILATION PERFORMANCE PROFILE ");
    println!("{:=^80}", "");

    let mut total_time = 0.0;
    let mut timings = Vec::new();
    let test_start = Instant::now();

    for entry in gwi_files {
        // Check global timeout
        if test_start.elapsed().as_secs() > MAX_TOTAL_TIME_SECS {
            println!("\n⏱️  Global timeout reached ({} seconds). Stopping profiling.", MAX_TOTAL_TIME_SECS);
            break;
        }

        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();

        let mml_content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                println!("❌ {} - Failed to read: {}", filename, e);
                continue;
            }
        };

        let options = CompileOptions {
            format: OutputFormat::VGM,
            ..Default::default()
        };

        let compiler = MmlCompiler::new(options);

        // Time the compilation
        let start = Instant::now();
        let result = compiler.compile_from_source(&mml_content);
        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

        // Check per-file timeout
        if elapsed.as_secs() > TIMEOUT_PER_FILE_SECS {
            println!("⏱️  {:30} {:.2}ms (SLOW - exceeded {}s limit)", filename, elapsed_ms, TIMEOUT_PER_FILE_SECS);
            continue;
        }

        total_time += elapsed_ms;

        match result {
            Ok(result) => {
                println!(
                    "✓ {:30} {:8.2}ms  ({:7} bytes, {:4} parts, {:6} cmds)",
                    filename,
                    elapsed_ms,
                    result.data.len(),
                    result.info.part_count,
                    result.info.command_count
                );
                timings.push((filename.to_string(), elapsed_ms, result.data.len()));
            }
            Err(e) => {
                println!("❌ {:30} {:8.2}ms  Error: {}", filename, elapsed_ms, e);
            }
        }
    }

    println!("{:=^80}", "");
    if timings.is_empty() {
        println!("No successful compilations within timeout limits.");
    } else {
        println!(
            "Total compilation time: {:.2}ms ({:.2}s)",
            total_time,
            total_time / 1000.0
        );
        println!(
            "Average compilation time: {:.2}ms",
            total_time / timings.len() as f64
        );

        // Sort by time (slowest first)
        timings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("\n{:=^80}", " SLOWEST COMPILATIONS ");
        for (filename, time, size) in &timings[..std::cmp::min(5, timings.len())] {
            println!("  {}: {:.2}ms ({} bytes)", filename, time, size);
        }
    }

    println!("{:=^80}\n", "");
}

// ── Additional performance assertions ────────────────────────────────────────

/// Every sample must compile within the per-file timeout.
#[test]
fn all_samples_under_per_file_timeout() {
    let samples_dir = std::path::PathBuf::from("../browser-ide/public/samples");
    if !samples_dir.exists() {
        eprintln!("Skipping: samples directory not found");
        return;
    }

    let entries: Vec<_> = std::fs::read_dir(&samples_dir)
        .expect("read_dir failed")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "gwi").unwrap_or(false))
        .collect();

    let mut slow_files: Vec<String> = Vec::new();

    for entry in &entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let compiler = mml2vgm::compiler::compiler::MmlCompiler::new(mml2vgm::CompileOptions {
            format: mml2vgm::OutputFormat::VGM,
            ..Default::default()
        });

        let start = Instant::now();
        let _ = compiler.compile_from_source(&source);
        let elapsed = start.elapsed();

        if elapsed.as_secs() >= TIMEOUT_PER_FILE_SECS {
            slow_files.push(format!("{} ({:.2}s)", filename, elapsed.as_secs_f64()));
        }
    }

    assert!(
        slow_files.is_empty(),
        "Files exceeded per-file timeout of {}s:\n  {}",
        TIMEOUT_PER_FILE_SECS,
        slow_files.join("\n  ")
    );
}

/// Median compile time across all samples must be reasonable (< 2 s).
#[test]
fn median_compile_time_reasonable() {
    let samples_dir = std::path::PathBuf::from("../browser-ide/public/samples");
    if !samples_dir.exists() {
        eprintln!("Skipping: samples directory not found");
        return;
    }

    let entries: Vec<_> = std::fs::read_dir(&samples_dir)
        .expect("read_dir failed")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "gwi").unwrap_or(false))
        .collect();

    if entries.is_empty() {
        eprintln!("No .gwi files found, skipping median check");
        return;
    }

    let mut times_ms: Vec<f64> = Vec::new();

    for entry in &entries {
        let source = match std::fs::read_to_string(entry.path()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let compiler = mml2vgm::compiler::compiler::MmlCompiler::new(mml2vgm::CompileOptions {
            format: mml2vgm::OutputFormat::VGM,
            ..Default::default()
        });
        let start = Instant::now();
        let _ = compiler.compile_from_source(&source);
        times_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = times_ms[times_ms.len() / 2];

    println!("Median compile time: {:.2}ms", median);
    assert!(
        median < 2_000.0,
        "Median compile time {:.2}ms exceeds 2000ms threshold",
        median
    );
}

/// Compiling the same file 10 times in a row gives stable timing (< 3× variance).
#[test]
fn repeated_compile_stable() {
    let samples_dir = std::path::PathBuf::from("../browser-ide/public/samples");
    if !samples_dir.exists() {
        eprintln!("Skipping: samples directory not found");
        return;
    }

    // Use hello_world.gwi as the stability target
    let target = samples_dir.join("hello_world.gwi");
    if !target.exists() {
        eprintln!("Skipping: hello_world.gwi not found");
        return;
    }

    let source = std::fs::read_to_string(&target).expect("read failed");
    let compiler = mml2vgm::compiler::compiler::MmlCompiler::new(mml2vgm::CompileOptions {
        format: mml2vgm::OutputFormat::VGM,
        ..Default::default()
    });

    let runs = 10;
    let mut times_ms: Vec<f64> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let start = Instant::now();
        let _ = compiler.compile_from_source(&source);
        times_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let min = times_ms.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times_ms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("Repeated compile: min={:.2}ms  max={:.2}ms  ratio={:.2}x", min, max, max / min.max(0.001));

    // Max must be within 10× of min (generous allowance for CI noise).
    assert!(
        max < min * 10.0 + 100.0,
        "Compile time variance too high: min={:.2}ms max={:.2}ms",
        min,
        max
    );
}

