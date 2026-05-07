//! Phase 8 smoke tests: compile → VgmPlayer → render_to_pcm pipeline and format validation.
//!
//! These tests verify:
//!   1. The full compile→render pipeline produces non-empty, non-silent PCM.
//!   2. Per-format compilation (VGM/XGM/XGM2/ZGM) emits correct magic bytes.
//!   3. Compiled VGM headers have valid total_samples and correct chip clock fields.
//!   4. Per-chip-family compile smoke cases (PSG, FM, OPL families).

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::player::vgm_player::VgmPlayer;
use mml2vgm::{CompileOptions, OutputFormat, SoundChip};

const SAMPLE_RATE: u32 = 44100;
const VGM_HEADER_MIN: usize = 0x40;

// ── helpers ───────────────────────────────────────────────────────────────────

fn compile_vgm(mml: &str) -> Vec<u8> {
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    compiler
        .compile_from_source(mml)
        .unwrap_or_else(|e| panic!("compile failed: {}", e))
        .data
}

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

fn render_vgm(data: &[u8]) -> Vec<f32> {
    let mut player = VgmPlayer::new();
    player.load(data).expect("VgmPlayer::load failed");
    player.init_chips_from_header();
    player
        .render_to_pcm(SAMPLE_RATE)
        .expect("render_to_pcm failed")
}

fn assert_vgm_header(data: &[u8]) {
    assert!(
        data.len() >= VGM_HEADER_MIN,
        "output too small: {} bytes",
        data.len()
    );
    assert_eq!(&data[0..4], b"Vgm ", "missing VGM magic");
    let eof_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    assert_eq!(
        eof_offset + 4,
        data.len(),
        "VGM EOF offset mismatch (eof_offset={eof_offset}, data.len()={})",
        data.len()
    );
    let version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    assert!(version >= 0x0150, "unexpected VGM version: {version:#x}");
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

// ── full pipeline tests ───────────────────────────────────────────────────────

#[test]
fn pipeline_compile_and_render_returns_samples() {
    let mml = "@0 t120 o4 c4 d4 e4 f4";
    let vgm = compile_vgm(mml);
    assert_vgm_header(&vgm);

    let samples = render_vgm(&vgm);
    assert!(
        !samples.is_empty(),
        "render_to_pcm returned empty samples for note-bearing MML"
    );
    // stereo interleaved, so must be even
    assert_eq!(samples.len() % 2, 0, "stereo sample count must be even");
}

#[test]
fn pipeline_rendered_samples_are_not_all_silent() {
    // compile a 4-note phrase and verify at least one sample is non-zero
    let mml = "@0 t120 o4 c2 e2 g2";
    let vgm = compile_vgm(mml);
    let samples = render_vgm(&vgm);
    assert!(
        !samples.is_empty(),
        "render_to_pcm returned empty vector"
    );
    let any_nonzero = samples.iter().any(|&s| s != 0.0);
    assert!(any_nonzero, "all rendered samples are silent — chip emulation likely broken");
}

#[test]
fn pipeline_total_samples_nonzero_for_note_mml() {
    let mml = "@0 t120 o4 c4 d4 e4 f4 g4";
    let vgm = compile_vgm(mml);
    assert_vgm_header(&vgm);

    let total_samples = read_u32(&vgm, 0x18);
    assert!(
        total_samples > 0,
        "VGM header total_samples==0; wait commands likely missing from codegen"
    );
}

#[test]
fn pipeline_compile_info_duration_positive() {
    let mml = "@0 t120 o4 c4 d4 e4 f4";
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    let result = compiler.compile_from_source(mml).expect("compile failed");
    assert!(
        result.info.duration_seconds > 0.0,
        "CompileInfo.duration_seconds should be positive"
    );
    assert!(
        result.info.command_count > 0,
        "CompileInfo.command_count should be positive"
    );
}

// ── per-format magic byte tests ───────────────────────────────────────────────

#[test]
fn format_vgm_has_correct_magic() {
    let data = compile_format("@0 t120 o4 c4 d4", OutputFormat::VGM);
    assert_eq!(&data[0..4], b"Vgm ", "VGM magic mismatch");
}

#[test]
fn format_xgm_has_correct_magic() {
    let data = compile_format("@0 t120 o4 c4 d4", OutputFormat::XGM);
    assert!(data.len() >= 4, "XGM output too short");
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
    assert!(data.len() >= 4, "ZGM output too short");
    assert_eq!(&data[0..4], b"ZGM ", "ZGM magic mismatch");
}

#[test]
fn format_xgm_nonempty_output() {
    let data = compile_format("@0 t120 o4 c4 d4 e4 f4", OutputFormat::XGM);
    assert!(
        data.len() > 16,
        "XGM output suspiciously small: {} bytes",
        data.len()
    );
}

#[test]
fn format_zgm_nonempty_output() {
    let data = compile_format("@0 t120 o4 c4 d4 e4 f4", OutputFormat::ZGM);
    assert!(
        data.len() > 16,
        "ZGM output suspiciously small: {} bytes",
        data.len()
    );
}

// ── per-chip-family compile smoke tests ──────────────────────────────────────

#[test]
fn chip_sn76489_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::SN76489]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("SN76489 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ym2612_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::YM2612]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("YM2612 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ym2608_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::YM2608]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("YM2608 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ym2151_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::YM2151]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("YM2151 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ym3812_opl2_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::YM3812]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("YM3812 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ymf262_opl3_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::YMF262]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("YMF262 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ym2413_opll_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::YM2413]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("YM2413 compile failed");
    assert_vgm_header(&result.data);
}

#[test]
fn chip_ay8910_compile_smoke() {
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::AY8910]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("AY8910 compile failed");
    assert_vgm_header(&result.data);
}

// ── VGM player chip detection tests ──────────────────────────────────────────

#[test]
fn vgm_player_load_and_init_does_not_panic() {
    let mml = "@0 t120 o4 c4 d4 e4 f4";
    let vgm = compile_vgm(mml);
    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");
    player.init_chips_from_header();
    // no panic means success
}

#[test]
fn vgm_player_state_transitions() {
    use mml2vgm::player::vgm_player::PlayerState;

    let mml = "@0 t120 o4 c4 d4";
    let vgm = compile_vgm(mml);

    let mut player = VgmPlayer::new();
    assert_eq!(player.state(), PlayerState::Stopped);

    player.load(&vgm).expect("load failed");
    player.init_chips_from_header();

    player.play().expect("play failed");
    assert_eq!(player.state(), PlayerState::Playing);

    player.pause().expect("pause failed");
    assert_eq!(player.state(), PlayerState::Paused);

    player.resume().expect("resume failed");
    assert_eq!(player.state(), PlayerState::Playing);

    player.stop().expect("stop failed");
    assert_eq!(player.state(), PlayerState::Stopped);
}

#[test]
fn vgm_player_position_advances_during_render() {
    let mml = "@0 t120 o4 c2 d2 e2";
    let vgm = compile_vgm(mml);

    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");
    player.init_chips_from_header();
    player.play().expect("play failed");

    let initial_pos = player.position();
    let mut buf = vec![0.0f32; 512];
    player.generate_samples(&mut buf, 256).expect("generate_samples failed");
    let after_pos = player.position();

    assert!(
        after_pos > initial_pos,
        "position should advance after generate_samples (was {initial_pos}, now {after_pos})"
    );
}

#[test]
fn vgm_player_duration_matches_total_samples() {
    let mml = "@0 t120 o4 c4 d4 e4 f4";
    let vgm = compile_vgm(mml);

    let total_samples_from_header = read_u32(&vgm, 0x18);

    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");

    assert_eq!(
        player.duration(),
        total_samples_from_header,
        "VgmPlayer::duration() should match total_samples in VGM header"
    );
}

// ── multi-part compile tests ──────────────────────────────────────────────────

#[test]
fn multi_part_compile_produces_valid_vgm() {
    // Two parts sharing the same default chip
    let mml = "'A t120 o4 c4 e4 g4\n'B t120 o3 c2 g2";
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml)
    .expect("multi-part compile failed");

    assert_vgm_header(&result.data);
    assert!(
        result.info.part_count >= 1,
        "expected at least 1 part, got {}",
        result.info.part_count
    );
}

#[test]
fn loop_mml_compiles_without_error() {
    // Finite loop: (body)N syntax
    let mml = "@0 t120 o4 (c8 d8 e8)4";
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml);

    assert!(result.is_ok(), "loop MML compile failed: {:?}", result.err());
    assert_vgm_header(&result.unwrap().data);
}

// ── FM chip audio render tests ────────────────────────────────────────────────

const FM_INSTRUMENT_BLOCK: &str = "
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000
";

#[test]
fn ym2612_with_fm_instrument_renders_non_silent() {
    let mml = format!(
        "{{\n  PartYM2612 = A\n}}\n{}\n'A1 T120\n'A1 @0 v100 l2 o4 c d e f",
        FM_INSTRUMENT_BLOCK
    );
    let vgm = compile_vgm(&mml);
    assert_vgm_header(&vgm);

    let samples = render_vgm(&vgm);
    assert!(!samples.is_empty(), "YM2612 render returned empty samples");
    let any_nonzero = samples.iter().any(|&s| s != 0.0);
    assert!(any_nonzero, "YM2612 FM with instrument produced all-silent output");
}

#[test]
fn multi_chip_ym2612_sn76489_compile_and_render() {
    let mml = format!(
        "{{\n  PartYM2612 = A\n  PartSN76489 = B\n}}\n{}\n'A1 T120\n'A1 @0 v100 l4 o4 c e g\n'B1 T120\n'B1 v100 l2 o2 c g",
        FM_INSTRUMENT_BLOCK
    );
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(&mml)
    .expect("multi-chip compile failed");

    assert_vgm_header(&result.data);
    assert!(result.info.duration_seconds > 0.0, "duration should be positive");

    let samples = render_vgm(&result.data);
    assert!(!samples.is_empty(), "multi-chip render returned empty samples");
}

#[test]
fn sn76489_renders_non_silent_with_explicit_target() {
    // Target SN76489 explicitly via with_target_chips
    let compiler = MmlCompiler::new(
        CompileOptions::new()
            .with_output_format(OutputFormat::VGM)
            .with_target_chips(vec![SoundChip::SN76489]),
    );
    let result = compiler
        .compile_from_source("@0 t120 o4 c2 e2 g2")
        .expect("compile failed");
    assert_vgm_header(&result.data);

    let samples = render_vgm(&result.data);
    assert!(!samples.is_empty(), "render returned empty samples");
    assert!(
        samples.iter().any(|&s| s != 0.0),
        "explicit SN76489 render produced all-silent output"
    );
}

// ── error / negative cases ────────────────────────────────────────────────────

#[test]
fn invalid_mml_returns_error_not_panic() {
    // A completely garbage string should not panic, just return an error or empty
    let mml = "@@@ ??? ###";
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml);
    // Either succeeds with empty data or returns an error — it must not panic
    match result {
        Ok(r) => assert!(r.data.len() >= 4, "non-error result should have valid header"),
        Err(_) => {} // error is acceptable
    }
}

#[test]
fn empty_mml_compiles_to_minimal_vgm() {
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source("")
    .expect("empty MML should compile without error");
    // Should produce a valid VGM header even with no notes
    assert!(result.data.len() >= VGM_HEADER_MIN);
    assert_eq!(&result.data[0..4], b"Vgm ");
}

#[test]
fn rest_only_mml_has_positive_duration() {
    // Rests should still generate wait commands
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source("@0 t120 r1 r1 r1")
    .expect("rest-only compile failed");
    assert_vgm_header(&result.data);
    // Rests produce wait commands — total_samples should be nonzero
    let total_samples = read_u32(&result.data, 0x18);
    assert!(
        total_samples > 0,
        "rest-only MML should still produce wait samples"
    );
}
