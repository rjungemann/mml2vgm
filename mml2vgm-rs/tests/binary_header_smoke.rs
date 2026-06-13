//! Binary-container structural smoke tests (Golden Master Test Plan, Phase A).
//!
//! These checks deliberately do NOT make any claim about audio correctness.
//! They assert only that the compiler emits a structurally valid container for
//! each output format: correct magic bytes, a consistent EOF offset, a sane
//! version field, and that every targeted chip compiles to a loadable VGM.
//!
//! The Golden Master plan calls format-magic and "produced some bytes" checks
//! "coverage theatre" when they live next to (and masquerade as) audio tests.
//! They are kept here, isolated and honestly labelled, because a wrong magic
//! byte or a truncated header IS a real (if shallow) regression — just one
//! that has nothing to do with whether the rendered audio is correct. Audible
//! correctness is the job of `chip_audio_fingerprint.rs` (Layer 2) and
//! `chip_audio_invariants.rs` (Layer 4).

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::{CompileOptions, OutputFormat, SoundChip};

const VGM_HEADER_MIN: usize = 0x40;

// ── helpers ───────────────────────────────────────────────────────────────────

fn compile_format(mml: &str, fmt: OutputFormat) -> Vec<u8> {
    let compiler = MmlCompiler::new(CompileOptions {
        format: fmt,
        ..Default::default()
    });
    compiler
        .compile_from_source(mml)
        .unwrap_or_else(|e| panic!("compile_format {:?} failed: {}", fmt, e))
        .data
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Structural validation of a VGM container: size, magic, EOF offset, version.
fn assert_vgm_structure(data: &[u8]) {
    assert!(
        data.len() >= VGM_HEADER_MIN,
        "output too small: {} bytes",
        data.len()
    );
    assert_eq!(&data[0..4], b"Vgm ", "missing VGM magic");
    let eof_offset = read_u32(data, 4) as usize;
    assert_eq!(
        eof_offset + 4,
        data.len(),
        "VGM EOF offset mismatch (eof_offset={eof_offset}, data.len()={})",
        data.len()
    );
    let version = read_u32(data, 8);
    assert!(version >= 0x0150, "unexpected VGM version: {version:#x}");
}

fn compile_chip(chip: SoundChip) -> Vec<u8> {
    MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![chip]),
    )
    .compile_from_source("@0 t120 o4 c4 d4 e4")
    .unwrap_or_else(|e| panic!("{chip:?} compile failed: {e}"))
    .data
}

// ── per-format magic byte tests ───────────────────────────────────────────────

#[test]
fn format_vgm_has_correct_magic_and_structure() {
    let data = compile_format("@0 t120 o4 c4 d4", OutputFormat::VGM);
    assert_vgm_structure(&data);
}

#[test]
fn format_xgm_has_correct_magic() {
    let data = compile_format("@0 t120 o4 c4 d4", OutputFormat::XGM);
    assert!(data.len() > 16, "XGM output too short: {} bytes", data.len());
    assert_eq!(&data[0..4], b"XGM ", "XGM magic mismatch");
}

#[test]
fn format_xgm2_has_correct_magic() {
    let data = compile_format("@0 t120 o4 c4 d4", OutputFormat::XGM2);
    assert!(data.len() >= 4, "XGM2 output too short");
    assert_eq!(&data[0..4], b"XGM2", "XGM2 magic mismatch");
}

#[test]
fn format_zgm_has_correct_magic() {
    let data = compile_format("@0 t120 o4 c4 d4", OutputFormat::ZGM);
    assert!(data.len() > 16, "ZGM output too short: {} bytes", data.len());
    assert_eq!(&data[0..4], b"ZGM ", "ZGM magic mismatch");
}

// ── header field tests ────────────────────────────────────────────────────────

#[test]
fn note_bearing_mml_has_nonzero_total_samples() {
    let data = compile_format("@0 t120 o4 c4 d4 e4 f4 g4", OutputFormat::VGM);
    assert_vgm_structure(&data);
    let total_samples = read_u32(&data, 0x18);
    assert!(
        total_samples > 0,
        "VGM header total_samples==0; wait commands likely missing from codegen"
    );
}

#[test]
fn rest_only_mml_has_positive_duration() {
    let data = compile_format("@0 t120 r1 r1 r1", OutputFormat::VGM);
    assert_vgm_structure(&data);
    let total_samples = read_u32(&data, 0x18);
    assert!(
        total_samples > 0,
        "rest-only MML should still produce wait samples"
    );
}

// ── per-chip-family container smoke tests ─────────────────────────────────────
//
// Each asserts only that the chip compiles to a structurally valid VGM
// container — NOT that the chip renders correct (or any) audio.

#[test]
fn chip_sn76489_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::SN76489));
}

#[test]
fn chip_ym2612_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::YM2612));
}

#[test]
fn chip_ym2608_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::YM2608));
}

#[test]
fn chip_ym2151_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::YM2151));
}

#[test]
fn chip_ym3812_opl2_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::YM3812));
}

#[test]
fn chip_ymf262_opl3_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::YMF262));
}

#[test]
fn chip_ym2413_opll_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::YM2413));
}

#[test]
fn chip_ay8910_compiles_to_valid_container() {
    assert_vgm_structure(&compile_chip(SoundChip::AY8910));
}

// ── multi-part / loop / edge cases ────────────────────────────────────────────

#[test]
fn multi_part_compile_produces_valid_container() {
    let mml = "'A t120 o4 c4 e4 g4\n'B t120 o3 c2 g2";
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml)
    .expect("multi-part compile failed");

    assert_vgm_structure(&result.data);
    assert!(
        result.info.part_count >= 1,
        "expected at least 1 part, got {}",
        result.info.part_count
    );
}

#[test]
fn loop_mml_compiles_to_valid_container() {
    let mml = "@0 t120 o4 (c8 d8 e8)4";
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml);
    assert!(result.is_ok(), "loop MML compile failed: {:?}", result.err());
    assert_vgm_structure(&result.unwrap().data);
}

#[test]
fn empty_mml_compiles_to_minimal_vgm() {
    let data = compile_format("", OutputFormat::VGM);
    assert!(data.len() >= VGM_HEADER_MIN);
    assert_eq!(&data[0..4], b"Vgm ");
}

#[test]
fn invalid_mml_returns_error_not_panic() {
    let mml = "@@@ ??? ###";
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml);
    match result {
        Ok(r) => assert!(r.data.len() >= 4, "non-error result should have valid header"),
        Err(_) => {} // error is acceptable; the contract is "must not panic"
    }
}
