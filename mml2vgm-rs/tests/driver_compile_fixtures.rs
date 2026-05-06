//! Integration tests: per-driver fixture compilation to valid VGM output.

use std::time::Instant;

use mml2vgm::drivers::{
    DriverCompileOptions, ExternalDriver, MoonDriver, MuapDriver, MucomDriver, PMDDriver,
};

const VGM_HEADER_SIZE: usize = 0x40;
const MAX_COMPILE_SECS: f64 = 5.0;

fn assert_valid_vgm_output(bytes: &[u8]) {
    assert!(
        bytes.len() >= VGM_HEADER_SIZE,
        "expected at least {} bytes, got {}",
        VGM_HEADER_SIZE,
        bytes.len()
    );
    assert_eq!(&bytes[0..4], b"Vgm ", "missing VGM magic");

    let eof_offset = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    assert_eq!(
        eof_offset + 4,
        bytes.len(),
        "EOF offset should match total file size"
    );
}

fn compile_fixture(driver: &dyn ExternalDriver, source: &str, fixture_name: &str) {
    let start = Instant::now();
    let result = driver
        .compile(source, &DriverCompileOptions::default())
        .unwrap_or_else(|e| panic!("{} compile failed: {}", fixture_name, e));
    let elapsed = start.elapsed().as_secs_f64();

    assert!(
        elapsed < MAX_COMPILE_SECS,
        "{} compile exceeded {:.1}s (took {:.2}s)",
        fixture_name,
        MAX_COMPILE_SECS,
        elapsed
    );
    assert_valid_vgm_output(&result.data);
}

#[test]
fn mucom_fixture_compile() {
    let fixture = "; mucom fixture\n@0 t120 o4 c d e f\n@6 t120 o3 c2 g2\n";
    compile_fixture(&MucomDriver, fixture, "mucom_fixture_compile");
}

#[test]
fn mucom_fixture_psg_noise_compile() {
    let fixture =
        "; mucom psg/noise fixture\n@6 t120 o3 c2 g2\n@7 t120 o3 e2 b2\n@9 t120 o4 c4 c4\n";
    compile_fixture(&MucomDriver, fixture, "mucom_fixture_psg_noise_compile");
}

#[test]
fn moon_fixture_opn2_compile() {
    let fixture = "#MD\n#OPN2\n#TEMPO 120\n@0 o4 c d e f\n";
    compile_fixture(&MoonDriver, fixture, "moon_fixture_opn2_compile");
}

#[test]
fn moon_fixture_opna_compile() {
    let fixture = "#MD\n#OPNA\n#TEMPO 120\n@0 o4 c d e f\n";
    compile_fixture(&MoonDriver, fixture, "moon_fixture_opna_compile");
}

#[test]
fn pmd_fixture_compile() {
    let fixture = "; PMD fixture\n@0 t120 o4 c d e f\n";
    compile_fixture(&PMDDriver, fixture, "pmd_fixture_compile");
}

#[test]
fn pmd_fixture_rhythm_compile() {
    let fixture = "; PMD rhythm fixture\n@RHYTHM\nBD SD TOM HH CYM RIM\n";
    compile_fixture(&PMDDriver, fixture, "pmd_fixture_rhythm_compile");
}

#[test]
fn muap_fixture_compile() {
    let fixture = "@OPNA\n@FM 0 t120 o4 c d e f\n";
    compile_fixture(&MuapDriver, fixture, "muap_fixture_compile");
}

#[test]
fn muap_fixture_sections_compile() {
    let fixture =
        "@OPNA\n@FM0 o4 c4 d4\n@SSG0 o5 e4 f4\n@RHYTHM0 BD HH\n@ADPCM0 c4\n";
    compile_fixture(&MuapDriver, fixture, "muap_fixture_sections_compile");
}