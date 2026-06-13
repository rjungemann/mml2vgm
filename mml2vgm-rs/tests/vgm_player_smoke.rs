//! VgmPlayer lifecycle / state-machine tests (Golden Master Test Plan, Phase A).
//!
//! Scope: the player's *control* behaviour — load/init, play/pause/resume/stop
//! transitions, position advancement, and that `duration()` matches the VGM
//! header. These are legitimate behavioural tests of the player state machine.
//!
//! Scope explicitly EXCLUDED: any assertion about the *content* of the rendered
//! audio. The original version of this file asserted `samples.iter().any(|s| s
//! != 0.0)` ("not silent") and treated that as audio coverage. That check is
//! green for any stub that emits a single click — it is exactly what let the
//! FM "every instrument is a sine" bug survive for years. Audio correctness now
//! lives in `chip_audio_fingerprint.rs` (Layer 2, spectral) and
//! `chip_audio_invariants.rs` (Layer 4, invariants). Format/header structural
//! checks live in `binary_header_smoke.rs`. See
//! `docs/dev/Golden_Master_Test_Plan.md`.

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::player::vgm_player::VgmPlayer;
use mml2vgm::{CompileOptions, OutputFormat};

const SAMPLE_RATE: u32 = 44100;

fn compile_vgm(mml: &str) -> Vec<u8> {
    MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml)
    .unwrap_or_else(|e| panic!("compile failed: {e}"))
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

// ── compile metadata ──────────────────────────────────────────────────────────

#[test]
fn compile_info_reports_positive_duration_and_commands() {
    let result = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source("@0 t120 o4 c4 d4 e4 f4")
    .expect("compile failed");
    assert!(
        result.info.duration_seconds > 0.0,
        "CompileInfo.duration_seconds should be positive"
    );
    assert!(
        result.info.command_count > 0,
        "CompileInfo.command_count should be positive"
    );
}

// ── player lifecycle ──────────────────────────────────────────────────────────

#[test]
fn player_load_and_init_does_not_panic() {
    let vgm = compile_vgm("@0 t120 o4 c4 d4 e4 f4");
    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");
    player.init_chips_from_header();
    // no panic means success
}

#[test]
fn player_state_transitions() {
    use mml2vgm::player::vgm_player::PlayerState;

    let vgm = compile_vgm("@0 t120 o4 c4 d4");
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
fn player_position_advances_during_render() {
    let vgm = compile_vgm("@0 t120 o4 c2 d2 e2");
    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");
    player.init_chips_from_header();
    player.play().expect("play failed");

    let initial_pos = player.position();
    let mut buf = vec![0.0f32; 512];
    player
        .generate_samples(&mut buf, 256)
        .expect("generate_samples failed");
    let after_pos = player.position();

    assert!(
        after_pos > initial_pos,
        "position should advance after generate_samples (was {initial_pos}, now {after_pos})"
    );
}

#[test]
fn player_duration_matches_header_total_samples() {
    let vgm = compile_vgm("@0 t120 o4 c4 d4 e4 f4");
    let total_samples_from_header = read_u32(&vgm, 0x18);

    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");

    assert_eq!(
        player.duration(),
        total_samples_from_header,
        "VgmPlayer::duration() should match total_samples in VGM header"
    );
}

#[test]
fn render_to_pcm_returns_even_stereo_frame_count() {
    // A structural property of the renderer's output shape (stereo interleave),
    // NOT a claim about audio content. Content is covered by Layer 2/4.
    let vgm = compile_vgm("@0 t120 o4 c4 d4 e4 f4");
    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("load failed");
    player.init_chips_from_header();
    let samples = player
        .render_to_pcm(SAMPLE_RATE)
        .expect("render_to_pcm failed");
    assert_eq!(
        samples.len() % 2,
        0,
        "stereo-interleaved output must have an even sample count"
    );
}
