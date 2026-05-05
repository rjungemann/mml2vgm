//! Integration test: Compile all example MML files
//!
//! This test verifies that all example MML files in the browser-ide can be
//! compiled successfully using the mml2vgm compiler.
//!
//! Run with: cargo test --test compile_examples -- --nocapture

use std::fs;
use std::path::PathBuf;
use mml2vgm::{CompileOptions, OutputFormat};
use mml2vgm::compiler::compiler::MmlCompiler;

#[test]
fn test_compile_all_examples() {
    let samples_dir = PathBuf::from("../browser-ide/public/samples");

    if !samples_dir.exists() {
        eprintln!("Warning: samples directory not found at {:?}", samples_dir);
        println!("Samples directory: {:?}", samples_dir);
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

    println!("\n========== Compiling {} example files ==========\n", gwi_files.len());

    let mut success_count = 0;
    let mut failure_count = 0;

    for entry in gwi_files {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();

        // Read the file
        let mml_content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                println!("❌ {} - Failed to read: {}", filename, e);
                failure_count += 1;
                continue;
            }
        };

        // Compile with default options
        let options = CompileOptions {
            format: OutputFormat::VGM,
            ..Default::default()
        };

        let compiler = MmlCompiler::new(options);

        match compiler.compile_from_source(&mml_content) {
            Ok(result) => {
                println!(
                    "✓ {} - Compiled successfully ({} bytes, {} parts, {} commands, {:.2}s)",
                    filename,
                    result.data.len(),
                    result.info.part_count,
                    result.info.command_count,
                    result.info.duration_seconds
                );
                success_count += 1;
            }
            Err(e) => {
                println!("❌ {} - Compilation failed: {}", filename, e);
                failure_count += 1;
            }
        }
    }

    println!("\n========== Results ==========");
    println!("✓ Successful: {}", success_count);
    println!("❌ Failed: {}", failure_count);
    println!("Total: {}\n", success_count + failure_count);

    // Fail the test if any compilation failed
    assert_eq!(
        failure_count, 0,
        "{} example file(s) failed to compile",
        failure_count
    );
}
