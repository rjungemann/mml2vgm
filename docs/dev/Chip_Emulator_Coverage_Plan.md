# Chip Emulator Coverage Plan

## Why this exists

User report: every FM instrument sounds like a sine wave, regardless of
algorithm or operator parameters. Confirmed: `mml2vgm-rs/src/chips/ym2612.rs`
`get_channel_output()` literally read:

```rust
// For now, simple sine wave output from operator 1
// Full FM synthesis implementation will come later
let output = self.get_operator_output(&channel.operators[0]);
```

i.e. operator 0's sine, no algorithm routing, no feedback, no
operator-to-operator modulation. A static audit found the same pattern in
**8 of 10 FM-family chips** (only `ym2612` is now fixed; `ymf271` still has
no implementation at all). The 25-test `vgm_codegen_accuracy` suite plus the
`vgm_player_smoke` "not all zero" assertion are the only emulator tests, and
neither one notices that a stub emulator produces audibly wrong audio — they
just verify the codegen byte stream and that the renderer produces nonzero
samples.

This document inventories every chip emulator, classifies what's missing,
and proposes a remediation path including the testing gap.

## Audit results — 2026-06-13

Static signals (algorithm-routing detection in `get_output`/`get_channel_output`,
per-operator references, placeholder/simplified comments):

| Chip       | File              | Status                       | Symptom |
|------------|-------------------|------------------------------|---------|
| YM2612     | `ym2612.rs`       | ✅ Fixed this pass            | Was sine-of-op1 only; full 8-algo routing + FB now implemented |
| YM2151     | `ym2151.rs`       | ❌ Stub                       | No algorithm routing; output references no operators at all |
| YM2203     | `ym2203.rs`       | ❌ Stub                       | No algorithm routing |
| YM2608     | `ym2608.rs`       | ❌ Stub                       | No algorithm routing; FM side identical to YM2612 needs |
| YM2413     | `ym2413.rs`       | ❌ Stub + placeholder banner   | No algorithm routing; "simplified FM" comment in file header |
| YM3526     | `ym3526.rs`       | ❌ Stub                       | 2-op output references op0+op1 but no OPL2 routing |
| YM3812     | `ym3812.rs`       | ❌ Stub                       | Same as YM3526 — 2-op stub |
| Y8950      | `y8950.rs`        | ❌ Stub                       | Same as YM3526 — 2-op stub |
| YMF262     | `ymf262.rs`       | ❌ Stub                       | OPL3 needs 2-op + 4-op modes; currently stub |
| YMF271     | `ymf271.rs`       | ❌ Skeleton only              | No feedback path, no algorithm routing — empty `generate_samples`? |
| SN76489    | `sn76489.rs`      | ⚠️ Audible (verified)         | Has note: "For now, simple noise implementation" — tone OK, noise approximate |
| AY8910     | `ay8910.rs`       | ❓ Untested                    | Channel-aware; no audible-quality verification on hand |
| K051649    | `k051649.rs`      | ❓ Untested                    | Wavetable SCC; channel-aware |
| HuC6280    | `huc6280.rs`      | ❓ Untested                    | Wavetable PSG; channel-aware |
| NES APU    | `nes_apu.rs`      | ⚠️ Placeholder note            | Pulse/triangle/noise/DPCM; "simplified" envelope code |
| DMG        | `dmg.rs`          | ⚠️ No `.channels` references   | GB pulse/wave/noise; suspicious — heuristic says "no channel access" |
| VRC6       | `vrc6.rs`         | ❓ Untested                    | NES extension; channel-aware? — heuristic flagged no channel access |
| POKEY      | `pokey.rs`        | ❓ Untested                    | Atari PSG; channel-aware |
| SegaPCM    | `segapcm.rs`      | ❓ Untested                    | Sample-data playback; needs verification |
| RF5C164    | `rf5c164.rs`      | ❌ Banner: "placeholder"       | Mega CD PCM; no PCM sample read path on heuristic |
| C140       | `c140.rs`         | ❓ Untested                    | Namco arcade PCM |
| C352       | `c352.rs`         | ⚠️ No PCM read path detected   | Namco System 22 PCM |
| K053260    | `k053260.rs`      | ❓ Untested                    | Konami arcade PCM |
| K054539    | `k054539.rs`      | ❓ Untested                    | Konami arcade PCM |
| QSound     | `qsound.rs`       | ❓ Untested (largest file)     | Capcom; 900 lines, channel-aware |

**Legend:** ✅ verified-correct or freshly-fixed. ⚠️ has implementation but
contains "simplified" / "for now" comments — likely partially-correct.
❌ confirmed stub (no algorithm routing in an FM chip, or explicit
placeholder banner). ❓ untested — needs an audible/golden-master test
before we can claim it's correct.

## Why the existing golden-master suite didn't catch this

`mml2vgm-rs/tests/vgm_codegen_accuracy.rs` (25 tests) checks:
- Correct VGM opcode emitted per MML construct.
- Correct register *byte* values written (e.g. SN76489 attenuation maps
  back to the right MIDI note's divider).
- Wait commands sum to the right total sample count.

It tests the **compiler**, not the **emulator**. The compiler always emitted
the correct register writes — the YM2612 stub correctly received the
algorithm/feedback values from the codegen, it just then ignored them when
rendering samples.

`mml2vgm-rs/tests/vgm_player_smoke.rs` does run the emulator, but the only
audio assertions are:
- `samples.len() > 0`
- `samples.iter().any(|s| s.abs() > some_epsilon)` (non-silent)

A stub that outputs a sine for every algorithm passes that bar.

**The missing test layer:** *audio correctness*. We need a reference
waveform per (chip, MML test fixture) pair and a similarity assertion
between our render and the reference. Options:
1. **External golden master** — render the test MML via a known-good
   emulator (libvgm, Genesis Plus GX, vgmplay, MAME) and commit the
   resulting WAV as a fixture. Our test renders the same MML and compares.
2. **Spectral fingerprint** — for each FM instrument and algorithm, capture
   the expected fundamental + first few harmonics; assert FFT bins land in
   the right buckets. Tolerant to small phase / envelope variations.
3. **Bit-accurate reference** — port a known-good open-source emulator
   (e.g. Nuked-OPN2 for YM2612, Nuked-OPM for YM2151) as the reference and
   compare sample-for-sample. Most ambitious.

The cheap practical first step is option 2: per-chip golden FFT
fingerprints. The fixtures are small (a handful of bins per test), they
don't require shipping WAV files, and they catch the precise failure mode
we just hit (stub outputs single sine → fundamental correct but all other
algorithm-driven harmonics absent).

## Remediation plan

### Phase 1: structural — get every chip "audibly plausible"

Per-chip task pattern (target ~1-2 hours each for the FM family; PCM and
PSG are smaller scope):

1. Read the chip's data sheet / vgmrips wiki page on register layout (most
   are already linked at the top of each `.rs` file).
2. Identify what kind of synthesis it is:
   - **FM-family** (YM2151/2203/2608/2413/2612/3526/3812/Y8950/YMF262/YMF271):
     N-operator per channel; algorithms route operators into carriers.
     Borrow the structure from the YM2612 fix in this branch as a template.
     OPL family (YM3526/3812/Y8950/YMF262) is 2-op (or 4-op in OPL3); fewer
     algorithms but same shape.
   - **PSG** (SN76489/AY8910/HuC6280/POKEY/etc.): tone divider + amplitude
     attenuation; sometimes wavetable (K051649, HuC6280) or noise channel.
   - **PCM** (SegaPCM/RF5C164/C140/C352/K053260/K054539/QSound):
     sample-data ROM/RAM read on key-on, advance read pointer at the
     configured pitch, multiply by per-voice volume.
3. Write the synthesis path. Aim for "audibly correct" (right algorithm,
   right rough timbre) rather than bit-accurate.
4. Add a focused test asserting the chip produces non-trivial harmonic
   content (see Phase 2).

**Suggested order** (by user-visible impact / sample coverage):
1. ✅ YM2612 — fixed in this branch (Hello World, Genesis FM).
2. ⏭ YMF262 / YM3812 / YM3526 / Y8950 (OPL2/OPL3 family — many samples
   reference these and they share the same shape).
3. ⏭ YM2151 (Sharp X68000, arcade) — distinct from YM2612 algorithm chart.
4. ⏭ YM2608 (PC-98) — adds SSG + ADPCM to YM2612-style FM.
5. ⏭ YM2203 / YM2413 (PC-88 / Master System / NES expansions).
6. ⏭ NES APU / DMG (verify channel mixing actually fires).
7. ⏭ RF5C164 and other PCM chips (verify PCM data read path).
8. ⏭ YMF271 (least-used; lowest priority).

### Phase 2: build the missing test layer

Add `mml2vgm-rs/tests/chip_audio_fingerprint.rs`:

```rust
#[test]
fn ym2612_alg_7_all_carrier_produces_four_harmonics() {
    let mml = include_str!("fixtures/ym2612_alg7_test.gwi");
    let pcm = compile_and_render(mml, /* duration */ 1.0);
    let mag = fft_magnitudes(&pcm, /* fundamental Hz */ 440.0);
    // ALG 7 with detuned operators: fundamental + 2x + 3x + 4x harmonics
    assert!(mag.fundamental > 0.1);
    assert!(mag.second_harmonic > 0.05);  // would have been ~0 in the stub
    assert!(mag.third_harmonic > 0.03);
}
```

Per FM chip + algorithm pair, write 1-2 such tests. Same pattern works for
PSG chips (assert noise channel has broadband content, not single sine) and
PCM chips (assert key-on triggers content matching the loaded sample).

These fixtures live in `mml2vgm-rs/tests/fixtures/` as small `.gwi` files.

### Phase 3: external reference (optional, longer-term)

If Phase 2 fingerprints prove insufficient (e.g. fingerprint passes but
audio still sounds wrong), bring in an external emulator for ground truth:
- For YM2612: Genesis Plus GX or Nuked-OPN2.
- For OPL: ymfm or Nuked-OPL3.
- For PCM: vgmplay's sample renderer.

Commit reference WAVs as test fixtures (max ~1 second per test, keeps
repo size bounded). Compare via spectrogram correlation or
cross-correlation peak strength rather than sample-for-sample (allows
small phase shifts).

## Out-of-scope here (separate concerns)

- **Bit-accurate emulation.** The Phase-1 fixes aim for audibly correct,
  not cycle-exact. Bit-accurate would require porting Nuked-OPN2 etc.,
  which is multi-thousand-line C → Rust per chip.
- **CPU-cost optimisation of the chip player.** Per the §X profile, FM
  emulation already dominates main-thread CPU. Adding more operators will
  scale linearly. A Web Worker offload (noted in §X) is the longer-term
  fix for that.
- **Pipeline issues.** All the work in this document assumes the producer
  pipeline and codegen are correct — both are now verified via §W/§X plus
  the existing 25-test vgm_codegen_accuracy suite.

## Quick reference: status by category

```
FM-family   (10):  fixed 1 / stub 8 / skeleton 1  →  fixed 10%
PSG-family  (8):   audible 2 / untested 6        →  unknown
PCM-family  (7):   stub 1 / suspect 1 / untested 5 →  unknown
```

Only ~10% of the chip emulators have been verified through real audio.
The other ~90% will need either Phase-1 implementation work or Phase-2
fingerprint tests before we can claim browser-IDE playback is faithful
across the supported chip library.
