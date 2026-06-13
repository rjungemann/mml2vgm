# Golden Master Test Plan

## Why we need this

The existing `vgm_codegen_accuracy.rs` (25 tests) and `vgm_player_smoke.rs`
suites are mis-labelled as "golden master" tests. They are not. Together they
have two functions and one demonstrated failure mode:

- **Function 1 (`vgm_codegen_accuracy.rs`):** compare the VGM compiler's
  emitted bytes (register addresses, register values, wait sample counts)
  against hand-derived expected values. Legitimate **compiler** unit tests.
- **Function 2 (`vgm_player_smoke.rs`):** assert format magic bytes are
  correct, and assert "the renderer produced *some* non-zero output."
- **Demonstrated failure mode:** they passed for years while
  `YM2612::get_channel_output` literally returned `sin(operators[0].phase)`
  regardless of algorithm — every FM instrument sounded like a sine wave.
  The codegen tests didn't care (codegen was correct; the bug was in the
  emulator); the smoke tests didn't care (a sine is non-silent).

So the bar this document sets:

> A "golden master" test passes only when the rendered audio is *audibly
> correct*. "Audibly correct" must be a falsifiable, machine-checkable
> claim — not "the renderer produced some output."

This document defines what falsifiable claims to make, in what layers, with
what regeneration / maintenance workflow.

## Anti-patterns to retire

1. **"Not silent" assertions.** `samples.iter().any(|s| s != 0.0)` is
   green for any stub that produces a single click. Delete every variant.
2. **Format-magic checks.** `data[0..4] == b"Vgm "` adds no signal — your
   eyes catch this in the first second of manual testing. Move to a
   `binary_header_smoke` file if you want it at all; don't conflate with
   audio correctness.
3. **Misnamed test files.** `vgm_codegen_accuracy.rs` should be
   `compiler_codegen_tests.rs`. The "golden master" framing implied
   end-to-end coverage that didn't exist and let the FM stub live there.

## Layered test strategy

Four test layers, each catching a distinct class of bug, each with
different fixture cost and runtime budget.

### Layer 1 — Codegen unit tests (already exist; rename + keep)

**What:** for each MML construct (note, rest, instrument, octave shift,
tempo, `(...)N` loop, `$` loop marker, etc.) verify the compiler emits the
expected VGM bytes.

**Catches:** parser bugs, lexer bugs, register-write codegen bugs, header
field bugs, source-map line-number drift.

**Doesn't catch:** anything about the renderer.

**Fixture cost:** zero (inline `assert_eq!` against hand-derived numbers).

**Action:** rename `vgm_codegen_accuracy.rs` →
`compiler_codegen_tests.rs`, update the module doc-comment to make clear
these are compiler-side tests, not emulator-side. Keep all 25 tests.

### Layer 2 — Spectral fingerprint tests (new, primary recommendation)

**What:** for each (chip, algorithm, instrument-shape) combination, render
~0.5 s of audio, FFT it, and assert specific frequency bins meet specific
magnitude thresholds.

The FM stub failure mode is exactly this shape: it produces a sine, so the
fundamental bin has signal but every harmonic bin is empty. A correct
4-operator FM voice with algorithm 4 (two parallel pairs) produces a
fundamental plus harmonics determined by the operator multiples — typically
2×, 3×, 4× ratios easily visible in an FFT.

**Catches:**
- Stub emulators that synthesize a single sine.
- Algorithm-routing bugs (wrong harmonic content for a given algorithm).
- Feedback bugs (FB > 0 adds distinctive harmonics; FB = 0 should be
  pure carrier).
- Per-operator parameter bugs (DT, ML control harmonic ratios; if ML=2
  doesn't produce a 2× harmonic, ML is broken).

**Doesn't catch:** envelope timing bugs (Layer 3), panning / stereo bugs
(Layer 3), PCM playback bugs (need sample-data tests; see below).

**Fixture cost:** zero (each test is a constant inline tuple of expected
bin magnitudes). No WAV files in the repo.

**Runtime cost:** each test renders 0.5 s × 44.1 kHz = 22,050 stereo
frames, then an FFT. With `realfft` or hand-rolled DFT for only the bins
of interest, each test is single-digit milliseconds. A full suite of
~50 fingerprint tests runs in well under a second.

**Sketch:**

```rust
// mml2vgm-rs/tests/fixtures/ym2612_alg4_440hz.gwi
'{
    TitleName  = ALG4 fingerprint
    Format     = VGM
    PartYM2612 = A
}

; Two parallel pairs: op1 modulates op2, op3 modulates op4.
; Detuning op1 and op3 produces a recognisable inharmonic spectrum.
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,000,000,000,000,000,000,002,000,000,000  ; op1: ML=2 (2× modulator)
'@ 031,000,000,000,000,000,000,001,000,000,000  ; op2: ML=1 (carrier)
'@ 031,000,000,000,000,000,000,002,000,000,000  ; op3: ML=2
'@ 031,000,000,000,000,000,000,001,000,000,000  ; op4: ML=1
'@ 004,000  ; ALG=4 FB=0

'A1 T120 @0 v100 l1 o4 a  ; A=440 Hz, whole note
```

```rust
// mml2vgm-rs/tests/chip_audio_fingerprint.rs

#[test]
fn ym2612_alg4_440hz_has_modulated_spectrum() {
    let mml = include_str!("fixtures/ym2612_alg4_440hz.gwi");
    let pcm = render_for_seconds(mml, 0.5);

    let mag = magnitude_at(&pcm, &[
        220.0,   // half — should be quiet (no subharmonics)
        440.0,   // fundamental — loud (carrier)
        880.0,   // 2× — present (modulator ML=2 creates this sideband)
        1760.0,  // 4× — present (second-order FM sideband)
        3520.0,  // 8× — quiet (FB=0 limits high harmonics)
    ]);

    assert!(mag[0] < 0.01, "expected no subharmonic at 220 Hz, got {}", mag[0]);
    assert!(mag[1] > 0.1, "fundamental missing");
    assert!(mag[2] > 0.05, "expected 2× sideband (modulator ML=2)");
    assert!(mag[3] > 0.02, "expected 4× harmonic from FM");
    assert!(mag[4] < 0.02, "didn't expect strong 8× without feedback");
}
```

**Action:** add `mml2vgm-rs/tests/chip_audio_fingerprint.rs`. Start with
one test per FM chip + one or two algorithm-discrimination tests per FM
chip. Add PSG tests (assert noise channel has broadband content; tone
channel has a clean fundamental + odd harmonics). Add PCM key-on tests
(assert any signal at all in the output spectrum, distinct from baseline
noise).

Target: ~50 tests covering every chip the compiler exercises. Each catches
a class of failure (algorithm routing, feedback presence, ML mapping,
detune sign, etc.).

### Layer 3 — Reference-audio similarity tests (longer-term)

**What:** for a curated set of MML fixtures, commit short reference WAV
files generated from a known-good external emulator
(`vgmplay`, libvgm, Genesis Plus GX). The test renders the same MML via
our emulator and compares against the reference.

**Catches:**
- Envelope timing drift (AR/DR/SR/RR phases off by samples).
- Panning bugs.
- Subtle phase-modulation distortion shape mismatches.
- Anything the fingerprint tests' bin-based assertions are too coarse to
  catch.

**Doesn't catch:** anything the external reference is itself wrong about
(some chip behaviours are still disputed in the emulator community).

**Fixture cost:** non-trivial. Each ref WAV at 44.1 kHz mono = ~88 KB per
second. 20 fixtures × 1 second × 2 channels × 2 bytes = ~7 MB. Compress
to FLAC: ~30-50 % size reduction. Or limit to 0.5 s clips. Or commit
fixtures to `git lfs` if the repo can stomach it; otherwise keep WAVs
small and in-tree.

**Comparison metric:** **not** sample-for-sample equality. Use:
- **Cross-correlation peak** — robust to small phase shifts.
- **Spectrogram log-magnitude L2 distance** — robust to small timing
  drift, but catches harmonic content errors.
- **Peak-aligned mean-squared error** — for short clips with one
  attack, align peaks and then compare.

A test that asserts "spectrogram L2 distance < 0.1" gives a meaningful
tolerance.

**Action (deferred):** add a Layer-3 file only after Layer 2 stabilises
and starts surfacing regressions Layer 2 misses. The fingerprint tests
catch the vast majority of stub-class bugs without WAV fixtures; build
the WAV infrastructure when you have a concrete bug Layer 2 can't catch.

### Layer 4 — Property / invariant tests

**What:** generic claims that should hold across every emulator regardless
of chip type. Examples:

- **Envelope monotonicity:** during the release phase, `|sample[i+1]|`
  must trend downward (averaged over a few samples to smooth oscillation).
- **Silence after key-off + RR samples:** at `release_samples + slop`
  after key-off, output magnitude must be below 1% of the attack peak.
- **Volume scaling:** doubling the part's `v` command from 50 to 100
  must roughly double the RMS of the rendered audio. (Trivially testable
  by rendering the same MML twice.)
- **Channel independence:** two parts on different channels of the same
  chip must produce a sum-of-sines-style output, not interfere. Render
  one part alone, render both together, subtract — residual should
  approximate the second part rendered alone.

**Catches:** envelope state machine bugs (Phase 1 of the chip plan keeps
the envelope generic; bugs there would show up here), volume/attenuation
bugs, channel-state-leak bugs.

**Fixture cost:** zero.

**Action:** add as a third file `tests/chip_audio_invariants.rs`
once Layer 2 is in place.

## Regeneration workflow

The biggest risk with golden-master tests is **accidentally blessing a
buggy state.** Once a test is regenerated to match the current (broken)
output, future runs all "pass." Mitigations:

### For Layer 1 (codegen)

No regeneration — expected values are hand-derived. If a test asserts
"register 0x28 byte == 0xF0" and the codegen changes, the test fails;
update the expected value only after manually verifying the new byte is
correct per the VGM spec / chip datasheet.

### For Layer 2 (fingerprints)

Each fingerprint test holds **explicit, numerical, physically-motivated
thresholds** (e.g., "expect 2× harmonic > 0.05 because ML=2 should
produce a 2× sideband"). When the test fails, fixing it requires
explaining *why* the threshold changed in terms of the chip's behaviour —
not just observing a new number.

Avoid `assert_eq!` against captured floats. Always use bounded ranges.

### For Layer 3 (reference WAVs)

Two-tier regeneration:
1. **External regen** (default): re-render the fixture MML through
   vgmplay/libvgm and commit the new WAV. Requires the external tool +
   a documented command in `tests/fixtures/README.md`.
2. **Internal regen** (gated): `MML2VGM_REBLESS_FIXTURES=1 cargo test`
   re-captures fixtures from the current build. **Must require an
   environment-variable opt-in** so it never runs by accident. The
   commit that re-blesses should explain in its message *why* the
   reference changed.

## Phased rollout

**Phase A — Cleanup (today, low risk):**
1. Rename `vgm_codegen_accuracy.rs` → `compiler_codegen_tests.rs`.
   Update doc-comment to clarify scope.
2. Delete `vgm_player_smoke.rs` audio assertions entirely (or
   delete the whole file). Keep the format-magic checks only if they
   ever caught a real bug — they probably didn't.

**Phase B — Layer 2 foundation (1-2 days):**
1. Add `tests/util/audio_fingerprint.rs` with `magnitude_at(pcm, freqs)`,
   `render_for_seconds(mml, seconds)`, FFT helper.
2. Add `tests/chip_audio_fingerprint.rs` with ~3 starter tests:
   one YM2612 alg-0 (deep cascade), one YM2612 alg-7 (parallel sum),
   one SN76489 tone. Verify they fail loudly on a regression injected
   in a scratch branch.
3. Add fixture .gwi files under `tests/fixtures/`.

**Phase C — Layer 2 coverage expansion (per chip, as that chip is
fixed):**
   - For each chip in `docs/dev/Chip_Emulator_Coverage_Plan.md`,
     write 2-5 fingerprint tests as the chip's implementation lands.
   - Pair the implementation PR with its fingerprint tests. A chip
     emulator change without an accompanying test is the failure mode
     we just lived through.

**Phase D — Layer 4 invariants (parallel with Phase C):**
   - Add `tests/chip_audio_invariants.rs` with envelope monotonicity,
     post-keyoff silence, volume linearity, channel independence.
   - These run against every chip without per-chip fixture work.

**Phase E — Layer 3 reference WAVs (only if Phase B/D find gaps):**
   - Set up the external-emulator regeneration tooling.
   - Commit short WAVs (≤ 1 s) for specific bugs Layer 2 couldn't catch.

## Anti-goal: bit-accurate emulation tests

Some upstream emulator projects (Nuked-OPN2, ymfm) aim for sample-exact
match against the real silicon. **That is not the goal here.** Trying to
hit bit-accuracy through fingerprint tests will produce false failures
on every floating-point reordering and tolerance tuning. Reserve
bit-accuracy aspirations for a separate effort — likely a port of
Nuked-OPN2 / ymfm as the reference, with WAV diff tolerance set to the
~−96 dB level you can reasonably claim.

## Concretely: what each layer would have caught today

| Bug | Layer 1 | Layer 2 | Layer 3 | Layer 4 |
|-----|---------|---------|---------|---------|
| FM `get_channel_output` returns op0 sine | ❌ | ✅ | ✅ | ⚠️ partial |
| Spurious YM2413/YM2151 clock defaults (§U) | ✅ | ✅ | ✅ | ❌ |
| `Octave-Rev` flag ignored (§T) | ⚠️ partial | ✅ | ✅ | ❌ |
| Stereo de-interleave bug (§B browser) | ❌ | ✅ | ✅ | ⚠️ partial |
| Source-map line numbers shifted (§Q) | ✅ | ❌ | ❌ | ❌ |
| Codegen drops VGM loop point (§K) | ✅ | ✅ | ✅ | ❌ |
| AY8910 (if it has the same stub pattern) | ❌ | ✅ | ✅ | ⚠️ partial |

Layer 2 (FFT fingerprints) is the highest leverage of the four — it
catches every emulator-stub-class bug without requiring external WAV
fixtures, and its assertions remain physically meaningful (each
threshold answers "*why* should this harmonic exist").

## Summary

- **Today's "golden master" suite is half compiler tests in disguise
  (`vgm_codegen_accuracy`) and half audio coverage theatre
  (`vgm_player_smoke`).** The theatre half deserves to be deleted; the
  compiler half deserves to be renamed.
- **Real audio correctness needs Layer 2 (spectral fingerprints).** Tiny
  fixtures, fast, physically-grounded thresholds, catches every stub
  emulator we currently ship.
- **Reference WAVs (Layer 3) come later**, only if Layer 2 misses
  something concrete.
- **Property tests (Layer 4) are free coverage** — write them once,
  apply to every chip.
- **Pair every chip implementation with its fingerprint tests**, the
  same way you pair business logic with unit tests. The FM stub lived
  for years because nobody had to write a test alongside it.
