//! Layer 4 — property / invariant audio tests (Golden Master Test Plan).
//!
//! These are generic claims that should hold for *every* emulator regardless of
//! chip type, so they need no per-chip fixtures and no captured reference data.
//! Each one is a falsifiable invariant: a stub or a regression that violates it
//! fails loudly. Unlike the Layer 2 fingerprints (which pin specific harmonic
//! content per chip), these pin cross-cutting behaviour — volume scaling,
//! determinism, numeric sanity, and silence-in/silence-out.
//!
//! See `docs/dev/Golden_Master_Test_Plan.md` (Layer 4 / Phase D).

#[path = "util/audio_fingerprint.rs"]
mod audio_fingerprint;
use audio_fingerprint::*;

/// A loud SN76489 square-wave tone at a fixed pitch and volume `v`.
fn sn_tone(v: u32) -> String {
    format!("{{\n  PartSN76489 = A\n}}\n'A1 T120 v{v} l1 o4 a")
}

// ── volume scaling ────────────────────────────────────────────────────────────

#[test]
fn volume_scaling_louder_volume_raises_rms() {
    // The VGM codegen maps MML v→PSG attenuation as (v >> 3); the emulator
    // turns attenuation into a linear amplitude. v56 → chip volume 7, v120 →
    // chip volume 15, an exact 2× amplitude step (8 vs 16 of 16 levels). So
    // RMS should roughly double. We assert a generous band around 2× rather
    // than an exact float, per the plan's "bounded ranges, never assert_eq! on
    // captured floats" rule.
    let quiet = rms(&render_for_seconds(&sn_tone(56), 0.5));
    let loud = rms(&render_for_seconds(&sn_tone(120), 0.5));

    assert!(quiet > 0.0, "quiet tone unexpectedly silent");
    assert!(loud > quiet, "louder volume must raise RMS (quiet={quiet}, loud={loud})");
    let ratio = loud / quiet;
    assert!(
        (1.6..=2.4).contains(&ratio),
        "doubling the chip volume step should ~double RMS; got ratio {ratio:.3} (quiet={quiet:.4}, loud={loud:.4})"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn rendering_is_deterministic() {
    // The emulator is a pure function of the VGM stream: two renders of the
    // same source must be bit-for-bit identical. A failure here points at
    // uninitialised state, time-dependent seeding, or iteration-order
    // nondeterminism in the chip mix.
    let mml = sn_tone(120);
    let a = render_stereo(&mml);
    let b = render_stereo(&mml);
    assert_eq!(a.len(), b.len(), "render length is nondeterministic");
    assert!(
        a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()),
        "two renders of identical source differ — emulator is nondeterministic"
    );
}

// ── numeric sanity ────────────────────────────────────────────────────────────

#[test]
fn rendered_samples_are_finite_and_bounded() {
    // No NaN/Inf, and output stays within a sane amplitude envelope. A chip
    // that overflows its accumulator or divides by zero shows up here before it
    // ever reaches an audio device.
    for mml in [
        sn_tone(120),
        format!(
            "{{\n  PartYM2612 = A\n}}\n'@ M 000\n'@ 031,000,000,000,000,000,000,001,000,000,000\n'@ 031,000,000,000,000,000,000,001,000,000,000\n'@ 031,000,000,000,000,000,000,001,000,000,000\n'@ 031,000,000,000,000,000,000,001,000,000,000\n'@ 007,000\n'A1 T120 @0 v127 l1 o4 a"
        ),
    ] {
        let pcm = render_stereo(&mml);
        assert!(!pcm.is_empty(), "render produced no samples");
        for (i, &s) in pcm.iter().enumerate() {
            assert!(s.is_finite(), "non-finite sample {s} at index {i}");
            assert!(
                s.abs() <= 4.0,
                "sample {s} at index {i} is implausibly large (accumulator overflow?)"
            );
        }
    }
}

// ── silence in → silence out ──────────────────────────────────────────────────

#[test]
fn rest_only_source_renders_silence() {
    // A part that only rests keys nothing on. The renderer must emit (near)
    // silence, not idle chip hum or a stuck DC level. This is the inverse of
    // the "not silent" anti-pattern: here silence is the *correct* answer, and
    // a spurious non-zero output is the bug.
    let mml = "{\n  PartSN76489 = A\n}\n'A1 T120 l1 r r";
    let mono = render_mono(mml);
    assert!(!mono.is_empty(), "rest-only render produced no frames");
    let peak = mono.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    assert!(
        peak < 1e-4,
        "rest-only source should be silent, but peak amplitude was {peak}"
    );
}
