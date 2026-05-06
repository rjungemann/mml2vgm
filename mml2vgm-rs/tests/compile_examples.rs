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

const VGM_HEADER_SIZE: usize = 0x40;

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn assert_valid_vgm_output(bytes: &[u8]) {
    assert!(
        bytes.len() >= VGM_HEADER_SIZE,
        "expected at least a {}-byte VGM header, got {} bytes",
        VGM_HEADER_SIZE,
        bytes.len()
    );
    assert_eq!(&bytes[0..4], b"Vgm ", "missing VGM magic");

    let eof_offset = read_u32_le(bytes, 4) as usize;
    assert_eq!(
        eof_offset + 4,
        bytes.len(),
        "EOF offset should match total VGM file size"
    );

    let version = read_u32_le(bytes, 8);
    assert!(version >= 0x0000_0150, "unexpected VGM version: {version:#x}");
}

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
                assert_valid_vgm_output(&result.data);
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

#[test]
fn test_simple_compilation() {
    use std::time::Instant;
    
    let mml = "o4 c4 d4 e4 f4 g4";
    let options = CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    };
    
    let compiler = MmlCompiler::new(options);
    
    println!("Starting compilation of simple MML...");
    let start = Instant::now();
    let result = compiler.compile_from_source(mml);
    let elapsed = start.elapsed();
    
    println!("Compilation took: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    
    match result {
        Ok(result) => {
            assert_valid_vgm_output(&result.data);
            println!("Success! Output: {} bytes", result.data.len());
        }
        Err(e) => {
            println!("Error: {}", e);
            panic!("Compilation failed");
        }
    }
}

#[test]
fn test_file_compilation() {
    use std::path::Path;
    use std::time::Instant;
    
    let input_path = Path::new("../browser-ide/public/samples/hello_world.gwi");
    let options = CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    };
    
    let compiler = MmlCompiler::new(options);
    
    println!("Starting file-based compilation of {:?}...", input_path);
    let start = Instant::now();
    let result = compiler.compile(input_path);
    let elapsed = start.elapsed();
    
    println!("Compilation took: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    
    match result {
        Ok(result) => {
            assert_valid_vgm_output(&result.data);
            println!("Success! Output: {} bytes", result.data.len());
        }
        Err(e) => {
            println!("Error: {}", e);
            panic!("Compilation failed");
        }
    }
}

#[test]
fn test_browser_sample_comment_lines_compile_from_source() {
    use std::time::Instant;

    let input_path = PathBuf::from("../browser-ide/public/samples/hello_world.gwi");
    let mml = fs::read_to_string(&input_path).expect("Failed to read hello_world.gwi");
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });

    let start = Instant::now();
    let result = compiler.compile_from_source(&mml).expect("Compilation failed");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs_f64() < 5.0,
        "comment-bearing browser sample compile took too long: {:.2}s",
        elapsed.as_secs_f64()
    );
    assert_valid_vgm_output(&result.data);
}
