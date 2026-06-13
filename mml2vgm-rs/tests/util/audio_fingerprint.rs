//! Shared helpers for Layer 2 (spectral fingerprint) and Layer 4 (invariant)
//! audio tests. See `docs/dev/Golden_Master_Test_Plan.md`.
//!
//! This file is intentionally NOT a top-level integration test crate. Cargo
//! only compiles `tests/*.rs` as test binaries; files under `tests/util/` are
//! pulled in by consumers with
//! `#[path = "util/audio_fingerprint.rs"] mod audio_fingerprint;`.
//!
//! The "FFT" here is a direct single-bin DFT (Goertzel-style) evaluated only
//! at the handful of frequencies a test cares about. There are no external
//! FFT dependencies, and for a 0.5 s window at a few target bins the cost is
//! sub-millisecond. The returned magnitude is a windowed amplitude estimate:
//! a pure sine of amplitude A at the target frequency reads back ~A, so the
//! thresholds in each test stay physically interpretable.

#![allow(dead_code)] // not every consumer uses every helper

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::player::vgm_player::VgmPlayer;
use mml2vgm::{CompileOptions, OutputFormat};

pub const SAMPLE_RATE: u32 = 44100;

/// Compile MML source to a VGM binary, panicking with the compiler error on
/// failure (these are test fixtures; a compile failure is a test failure).
pub fn compile_vgm(mml: &str) -> Vec<u8> {
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    compiler
        .compile_from_source(mml)
        .unwrap_or_else(|e| panic!("compile failed: {e}"))
        .data
}

/// Stereo-interleaved PCM rendered from MML, as produced by the VGM player.
pub fn render_stereo(mml: &str) -> Vec<f32> {
    let vgm = compile_vgm(mml);
    let mut player = VgmPlayer::new();
    player.load(&vgm).expect("VgmPlayer::load failed");
    player.init_chips_from_header();
    player
        .render_to_pcm(SAMPLE_RATE)
        .expect("render_to_pcm failed")
}

/// Full mono mixdown (average of the two stereo channels) of a rendered clip.
pub fn render_mono(mml: &str) -> Vec<f32> {
    let stereo = render_stereo(mml);
    let frames = stereo.len() / 2;
    let mut mono = Vec::with_capacity(frames);
    for f in 0..frames {
        mono.push(0.5 * (stereo[f * 2] + stereo[f * 2 + 1]));
    }
    mono
}

/// Render MML and return up to `seconds` of mono samples from the *sustained*
/// portion of the note. The clip can contain leading silence and an attack
/// transient before the note settles, and the note may not start at t=0, so a
/// fixed offset is unreliable. Instead this locates the note onset (first
/// sample above half the global peak), skips ~10 ms of attack, and returns the
/// window from there. Panics if the clip is silent — a silent fixture is a
/// test failure, not something to paper over with an empty buffer.
pub fn render_for_seconds(mml: &str, seconds: f32) -> Vec<f32> {
    let mono = render_mono(mml);
    let frames = mono.len();
    assert!(frames > 0, "render produced no frames (empty PCM)");

    let peak = mono.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    assert!(
        peak > 0.0,
        "rendered clip is completely silent — fixture produced no audible note"
    );

    // Onset = first sample reaching half the global peak.
    let onset = mono
        .iter()
        .position(|&s| s.abs() >= 0.5 * peak)
        .unwrap_or(0);
    // Skip ~10 ms of attack transient past the onset.
    let attack_skip = (SAMPLE_RATE / 100) as usize;
    let start = (onset + attack_skip).min(frames.saturating_sub(1));
    let want = (seconds * SAMPLE_RATE as f32) as usize;
    let end = (start + want).min(frames);
    mono[start..end].to_vec()
}

/// Hann window coefficient for sample `n` of `len`.
fn hann(n: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }
    let x = (std::f32::consts::PI * 2.0 * n as f32) / (len as f32 - 1.0);
    0.5 - 0.5 * x.cos()
}

/// Windowed single-bin amplitude estimate for one frequency.
///
/// Applies a Hann window (to suppress spectral leakage from a finite,
/// non-integer-period clip) and evaluates the DFT at exactly `freq`. The
/// result is normalised by the window's coherent gain so a clean sine of
/// amplitude A reads back ~A regardless of clip length.
pub fn magnitude_one(mono: &[f32], freq: f32) -> f32 {
    let n = mono.len();
    if n == 0 {
        return 0.0;
    }
    let w = 2.0 * std::f32::consts::PI * freq / SAMPLE_RATE as f32;
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    let mut wsum = 0.0f64;
    for (i, &s) in mono.iter().enumerate() {
        let win = hann(i, n) as f64;
        wsum += win;
        let sample = s as f64 * win;
        let angle = w as f64 * i as f64;
        re += sample * angle.cos();
        im += sample * angle.sin();
    }
    if wsum == 0.0 {
        return 0.0;
    }
    // 2/coherent-gain normalisation → amplitude of a sine at this bin.
    (2.0 * (re * re + im * im).sqrt() / wsum) as f32
}

/// Convenience: magnitude estimates for several frequencies at once.
pub fn magnitude_at(mono: &[f32], freqs: &[f32]) -> Vec<f32> {
    freqs.iter().map(|&f| magnitude_one(mono, f)).collect()
}

/// Root-mean-square level of a mono buffer.
pub fn rms(mono: &[f32]) -> f32 {
    if mono.is_empty() {
        return 0.0;
    }
    let sum: f64 = mono.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum / mono.len() as f64).sqrt() as f32
}

/// Broadband energy estimate: RMS minus the energy explained by a set of
/// discrete tonal bins. A pure tone has most of its energy in its bins, so
/// this stays small; broadband noise spreads energy across the spectrum and
/// leaves a large residual. Used to separate PSG noise from PSG tone.
pub fn broadband_residual(mono: &[f32], tonal_bins: &[f32]) -> f32 {
    let total = rms(mono);
    // Each bin amplitude A contributes A/sqrt(2) RMS; sum in quadrature.
    let tonal_power: f32 = tonal_bins
        .iter()
        .map(|&f| {
            let a = magnitude_one(mono, f);
            0.5 * a * a
        })
        .sum();
    let total_power = total * total;
    (total_power - tonal_power).max(0.0).sqrt()
}
