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
