//! Performance benchmark for MML compilation
//!
//! Run with: cargo bench --bench compilation_benchmark
//! Or with detailed output: cargo bench --bench compilation_benchmark -- --verbose

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::{CompileOptions, OutputFormat};
use std::fs;
use std::path::Path;

fn load_example_file(filename: &str) -> String {
    let path = Path::new("../browser-ide/public/samples").join(filename);
    fs::read_to_string(path).expect("Failed to read example file")
}

fn benchmark_individual_compilation(c: &mut Criterion) {
    let examples = vec![
        "hello_world.gwi",
        "arpeggio.gwi",
        "chord_progression.gwi",
        "drum_pattern.gwi",
        "general_test.gwi",
        "ay8910_test.gwi",
        "c140_test.gwi",
        "pcm_test.gwi",
        "pcm_test_2.gwi",
        "sega_pcm_test.gwi",
    ];

    let mut group = c.benchmark_group("individual_files");
    group.sample_size(10); // Reduce sample size since compilation is slow

    for filename in examples {
        if let Ok(mml_content) =
            fs::read_to_string(format!("../browser-ide/public/samples/{}", filename))
        {
            group.bench_with_input(BenchmarkId::from_parameter(filename), filename, |b, _| {
                b.iter(|| {
                    let options = CompileOptions {
                        format: OutputFormat::VGM,
                        ..Default::default()
                    };
                    let compiler = MmlCompiler::new(black_box(options));
                    compiler.compile_from_source(black_box(&mml_content))
                });
            });
        }
    }
    group.finish();
}

fn benchmark_parse_vs_codegen(c: &mut Criterion) {
    let mml = load_example_file("general_test.gwi");
    let options = CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    };

    let mut group = c.benchmark_group("compilation_phases");
    group.sample_size(10);

    group.bench_function("full_compilation", |b| {
        b.iter(|| {
            let compiler = MmlCompiler::new(black_box(options.clone()));
            compiler.compile_from_source(black_box(&mml))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_individual_compilation,
    benchmark_parse_vs_codegen
);
criterion_main!(benches);
