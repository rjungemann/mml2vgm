//! Layer-2 (spectral fingerprint) tests for chip emulator correctness.
//!
//! See `docs/dev/Golden_Master_Test_Plan.md` for the testing strategy.
//!
//! Each test compiles a small MML fixture, renders it through the
//! chip emulator, and asserts that the rendered audio has the
//! algorithm-implied harmonic structure. The assertions are
//! physically-motivated ranges, not captured float snapshots — when
//! one fails it should fail in a way that points at *which* spectral
//! property is wrong (presence of harmonics, modulation depth, etc).
//!
//! Coverage starts narrow (one FM-stub-class test per chip) and
//! grows alongside `docs/dev/Chip_Emulator_Coverage_Plan.md`. Pair
//! each new chip implementation with at least one fingerprint test
//! in this file.

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::player::vgm_player::VgmPlayer;
use mml2vgm::{CompileOptions, OutputFormat};

/// Compile MML, instantiate the VGM player, render `seconds` of audio.
/// Returns interleaved stereo samples.
fn render_mml(mml: &str, seconds: f32) -> Vec<f32> {
    let mut opts = CompileOptions::new();
    opts.format = OutputFormat::VGM;
    let compiler = MmlCompiler::new(opts);
    let result = compiler.compile_from_source(mml).expect("compile failed");
    let mut player = VgmPlayer::new();
    player.load(&result.data).expect("load failed");
    player.init_chips_from_header();
    player.play().expect("play failed");
    let n = (44100.0 * seconds) as usize;
    let mut buf = vec![0.0f32; n * 2];
    player
        .generate_samples(&mut buf, n)
        .expect("generate failed");
    buf
}

/// Crude high-frequency to low-frequency energy ratio. A pure sine has
/// a tiny ratio; a richly-modulated FM tone has substantially more.
///
/// Used as the cheapest possible "did the algorithm actually modulate"
/// check. Not a substitute for proper FFT bin assertions — but a stub
/// emulator that returns a single sine for every algorithm flunks it.
fn hf_lf_ratio(buf: &[f32]) -> f32 {
    let mut lo = 0.0f32;
    let mut hi = 0.0f32;
    for w in buf.windows(2) {
        let diff = w[1] - w[0];
        hi += diff * diff;
        lo += w[0] * w[0];
    }
    hi / (lo + 1e-9)
}

// ── YM2612 ────────────────────────────────────────────────────────────────────

/// ALG0 = four-operator cascade: op1 → op2 → op3 → op4 (out).
/// With staggered multiples (1, 2, 3, 1) and self-feedback FB=5 on op1
/// this produces a richly inharmonic spectrum.
const YM2612_ALG0_CASCADE: &str = "\
'{
    TitleName = ALG0 cascade
    Format = VGM
    ClockCount = 192
    PartYM2612 = A
}
'@ M 000
'@ 031,000,000,000,000,000,000,001,000,000,000
'@ 031,000,000,000,000,000,015,002,000,000,000
'@ 031,000,000,000,000,000,015,003,000,000,000
'@ 031,000,000,000,000,000,015,001,000,000,000
'@ 000,005
'A1 T120 @0 v100 l1 o4 a
";

/// ALG7 = four parallel carriers: op1 + op2 + op3 + op4 (out).
/// With identical operators this is literally 4 × sin(same phase) —
/// it sounds like a single sine. Used as the algorithm-discrimination
/// baseline; ALG0 should have substantially more harmonic content than ALG7.
const YM2612_ALG7_PARALLEL: &str = "\
'{
    TitleName = ALG7 parallel
    Format = VGM
    ClockCount = 192
    PartYM2612 = A
}
'@ M 000
'@ 031,000,000,000,000,000,000,001,000,000,000
'@ 031,000,000,000,000,000,000,001,000,000,000
'@ 031,000,000,000,000,000,000,001,000,000,000
'@ 031,000,000,000,000,000,000,001,000,000,000
'@ 007,000
'A1 T120 @0 v100 l1 o4 a
";

/// **Stub-detector.** If the YM2612 emulator ignores `algorithm` and
/// just outputs `sin(operators[0].phase)` for every channel — which it
/// did before the §Y rewrite — both ALG0 and ALG7 render as nearly
/// identical pure sines and the ratio is ~1.
///
/// A faithful FM emulator should produce dramatically richer HF
/// content for ALG0 than ALG7. We require at least 5× discrimination,
/// which corresponds to "the cascade actually modulates phase."
#[test]
fn ym2612_alg0_has_more_hf_content_than_alg7() {
    let alg0 = render_mml(YM2612_ALG0_CASCADE, 0.5);
    let alg7 = render_mml(YM2612_ALG7_PARALLEL, 0.5);

    let r0 = hf_lf_ratio(&alg0);
    let r7 = hf_lf_ratio(&alg7);

    eprintln!("ALG0 (cascade)  HF/LF ratio: {:.6}", r0);
    eprintln!("ALG7 (parallel) HF/LF ratio: {:.6}", r7);

    assert!(
        r0 > r7 * 5.0,
        "ALG0 should have substantially more HF content than ALG7. \
         got ALG0={}, ALG7={}. A sine-only stub fails this; correct \
         FM passes it by ~9×.",
        r0,
        r7,
    );
}

/// Cross-check that ALG7 with identical operators *is* close to a sine.
/// A correct emulator passes this; one that, say, accidentally summed
/// operators with wrong scaling would fail.
#[test]
fn ym2612_alg7_with_identical_ops_is_near_sine() {
    let alg7 = render_mml(YM2612_ALG7_PARALLEL, 0.5);
    let ratio = hf_lf_ratio(&alg7);
    assert!(
        ratio < 0.001,
        "ALG7 with identical ops should render as a sine (HF/LF < 0.001); \
         got {}",
        ratio,
    );
}

/// Smoke check: an FM channel that's been key-on'd must produce
/// nonzero audio within 0.5 s. Catches the §A class of bug where
/// register writes never reach the emulator (e.g. the VgmPlayer
/// forgetting to call `init_chips_from_header()`), as well as
/// envelope state machine bugs that leave attack-phase stuck at 0.
#[test]
fn ym2612_keyed_on_channel_produces_audio() {
    let buf = render_mml(YM2612_ALG7_PARALLEL, 0.5);
    let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.01, "expected audible output, got peak {}", peak);
}

// ── PSG / wavetable / square-wave chips ───────────────────────────────────────

/// Render a single tone via the part-letter convention for a given chip,
/// then assert audible peak. Intentionally generic — a stub that returns
/// silence (or all zeros) fails; anything actually generating sound passes.
fn assert_audible(label: &str, mml: &str) {
    let buf = render_mml(mml, 0.5);
    let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        peak > 0.005,
        "{}: expected audible output, got peak {} (chip likely returning silence)",
        label,
        peak,
    );
}

/// SN76489 tone: a quarter-note A. Catches any regression to the
/// SN76489 tone channel that drops the latch/data write sequence.
#[test]
fn sn76489_tone_channel_produces_audio() {
    assert_audible(
        "SN76489 tone",
        "'{
    TitleName = SN76489 tone
    Format = VGM
    ClockCount = 192
    PartSN76489 = B
}
'B1 T120 v100 l1 o4 a
",
    );
}

/// AY8910 / YM2149: tone channel. Has its own register layout
/// (tone period registers 0/2/4, mixer 0x07, channel volumes 0x08-0x0A).
/// A stub that doesn't initialise the mixer correctly produces silence.
#[test]
fn ay8910_tone_channel_produces_audio() {
    assert_audible(
        "AY8910 tone",
        "'{
    TitleName = AY8910 tone
    Format = VGM
    ClockCount = 192
    PartAY8910 = C
}
'C1 T120 v100 l1 o4 a
",
    );
}

/// K051649 (SCC) — uses 32-byte wavetable RAM per channel. A stub
/// that forgets to load a default waveform produces silence.
#[test]
fn k051649_scc_channel_produces_audio() {
    assert_audible(
        "K051649 (SCC)",
        "'{
    TitleName = SCC tone
    Format = VGM
    ClockCount = 192
    PartK051649 = D
}
'D1 T120 v100 l1 o4 a
",
    );
}

/// HuC6280 (PC Engine PSG): wavetable PSG like SCC. Has 6 channels;
/// channel 4-5 can also do noise. We just need any one of them audible.
#[test]
fn huc6280_pce_channel_produces_audio() {
    assert_audible(
        "HuC6280",
        "'{
    TitleName = HuC6280 tone
    Format = VGM
    ClockCount = 192
    PartHuC6280 = E
}
'E1 T120 v100 l1 o4 a
",
    );
}

/// NES APU pulse channel. The simplified envelope handling shouldn't
/// stop a quarter-note from producing sound.
#[test]
fn nes_apu_pulse_channel_produces_audio() {
    assert_audible(
        "NES APU pulse",
        "'{
    TitleName = NES pulse
    Format = VGM
    ClockCount = 192
    PartNES = F
}
'F1 T120 v100 l1 o4 a
",
    );
}

/// Game Boy DMG pulse channel.
#[test]
fn dmg_gameboy_pulse_channel_produces_audio() {
    assert_audible(
        "DMG pulse",
        "'{
    TitleName = DMG pulse
    Format = VGM
    ClockCount = 192
    PartDMG = G
}
'G1 T120 v100 l1 o4 a
",
    );
}

/// VRC6 (NES expansion): two pulse channels + one sawtooth.
#[test]
fn vrc6_channel_produces_audio() {
    assert_audible(
        "VRC6",
        "'{
    TitleName = VRC6
    Format = VGM
    ClockCount = 192
    PartVRC6 = H
}
'H1 T120 v100 l1 o4 a
",
    );
}

/// Atari POKEY.
///
/// Currently ignored: the chip emulator is correct (verified by inspection
/// of `chips/pokey.rs` — implements 4 channels with poly LFSR), but the
/// codegen has no `process_chip_note` arm for `Some("POKEY")`, so a
/// note-on emits zero VGM register writes. Adding `pokey_note_on`/
/// `pokey_note_off` + dispatch arm + per-state channel allocation is a
/// bigger change than the K051649/NES fixes in this commit. Tracked as
/// a known gap; un-ignore once the codegen path lands.
#[test]
#[ignore = "POKEY codegen path missing; chip emulator is fine"]
fn pokey_channel_produces_audio() {
    assert_audible(
        "POKEY",
        "'{
    TitleName = POKEY
    Format = VGM
    ClockCount = 192
    PartPOKEY = I
}
'I1 T120 v100 l1 o4 a
",
    );
}
