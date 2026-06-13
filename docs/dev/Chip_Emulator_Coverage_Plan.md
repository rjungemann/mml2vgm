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

## Audit results — 2026-06-13 (revised after manual code review)

**Initial static audit was wildly over-reporting.** My regex for
"algorithm routing detected" matched `match channel.algorithm` exactly
and missed every variation (`match self.channels[ch].algorithm`,
`match self.fm_channels[ch].algorithm`, the OPL `connection` toggle,
etc.). Manual code review of each "stub" file revealed real
implementations. Revised table below:

| Chip       | File              | Status                          | Notes |
|------------|-------------------|---------------------------------|-------|
| YM2612     | `ym2612.rs`       | ✅ Fixed this pass               | Was the only true FM stub — sine-of-op1 only. Full 8-algo routing + feedback now in place; verified via `chip_audio_fingerprint::ym2612_alg0_has_more_hf_content_than_alg7` (ALG0 cascade produces ~9× the HF content of ALG7 parallel) |
| YM2151     | `ym2151.rs`       | ✅ Implemented                   | Full 8-algorithm match in `get_channel_output` lines ~147-193; uses `op_output` + `op_feedback` |
| YM2203     | `ym2203.rs`       | ✅ Implemented                   | 8-algorithm match in `get_fm_output` lines ~148-160; OPN side; also has SSG side |
| YM2608     | `ym2608.rs`       | ✅ Implemented                   | Same OPNA-style 8-algorithm match in `get_fm_output` line ~161 |
| YM2413     | `ym2413.rs`       | ✅ Implemented                   | OPLL: only 1 algorithm (mod→carrier), proper feedback + phase modulation in `channel_sample` lines ~174-222 |
| YM3526     | `ym3526.rs`       | ✅ Implemented                   | OPL: 2-op with `connection` toggle (FM vs AM) — the OPL spec doesn't expose multiple algorithms |
| YM3812     | `ym3812.rs`       | ✅ Implemented                   | OPL2: same pattern as YM3526 |
| Y8950      | `y8950.rs`        | ✅ Implemented                   | MSX-AUDIO: same pattern, line ~125 has the `connection == 0` switch |
| YMF262     | `ymf262.rs`       | ⚠️ 2-op only                     | OPL3: implements the 2-op-per-channel `connection` toggle but does NOT implement the 4-op mode (which lets pairs of channels combine into one 4-op voice). Currently sounds like OPL2 when 4-op is enabled. Distinct from "stub" but incomplete. |
| YMF271     | `ymf271.rs`       | ✅ FFI to external               | Uses MAME's OPX emulator via FFI; substantially more accurate than anything we could write |
| SN76489    | `sn76489.rs`      | ✅ Audible / ⚠️ noise simplified  | Tone channels correct (Hello World bass verified). Noise channel uses a simplified LFSR — "For now, simple noise implementation" comment. Not a stub; just lower fidelity. |
| AY8910     | `ay8910.rs`       | ❓ Audibility-untested            | Has channel-aware generate; no fingerprint test yet |
| K051649    | `k051649.rs`      | ❓ Audibility-untested            | SCC wavetable; reads from waveform RAM |
| HuC6280    | `huc6280.rs`      | ❓ Audibility-untested            | PCE wavetable PSG |
| NES APU    | `nes_apu.rs`      | ⚠️ Audible / simplified envelope | "Constant volume from envelope (simplified: use volume field)" — works but envelope handling shortcut |
| DMG        | `dmg.rs`          | ❓ Audibility-untested            | Static heuristic falsely flagged "no channel access" — re-read needed |
| VRC6       | `vrc6.rs`         | ❓ Audibility-untested            | NES expansion; static heuristic flagged "no channel access" |
| POKEY      | `pokey.rs`        | ❓ Audibility-untested            | Atari |
| SegaPCM    | `segapcm.rs`      | ❓ Audibility-untested            | Sample-data playback |
| RF5C164    | `rf5c164.rs`      | ⚠️ Misleading banner              | Module doc says "placeholder" but has functional PCM playback (`current_addr` advance, volume + pan in `generate_samples` line ~228). The "placeholder" note is stale documentation, not real code state. |
| C140       | `c140.rs`         | ❓ Audibility-untested            | Namco arcade PCM |
| C352       | `c352.rs`         | ❓ Audibility-untested            | Namco System 22 PCM — heuristic flagged "no PCM read path", needs verification |
| K053260    | `k053260.rs`      | ❓ Audibility-untested            | Konami arcade PCM |
| K054539    | `k054539.rs`      | ❓ Audibility-untested            | Konami arcade PCM |
| QSound     | `qsound.rs`       | ❓ Audibility-untested            | Capcom; 900 lines, complex |

**Legend (revised):** ✅ implementation present and verified-correct or
known-good. ⚠️ implementation present but has known simplifications /
misleading docs / incomplete features (not a stub). ❓ audibility-untested
— no fingerprint test exists yet. **No chips are confirmed stubs after
the YM2612 fix.**

**Lesson learned about audit methodology:** static signal scans
("does this file mention `match algorithm`?") gave 9 false positives
out of 10 FM chips. The actual stub was caught only because the user
heard sine waves and reported them. **Static heuristics are not a
substitute for either reading the code or running fingerprint tests
that produce audible audio and assert spectral content.**

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

**Suggested order** (revised — most FM chips already work, audibility
testing is what's missing):

1. ✅ YM2612 stub fixed and fingerprint-tested.
2. ⏭ Add fingerprint tests for every chip that's currently in the
   "❓ Audibility-untested" column. One test per chip would have
   caught the YM2612 stub; one test per (chip, algorithm) gives
   broader coverage. **This is the highest-leverage next step** —
   it converts "we *think* it works" into "we *know* it works."
3. ⏭ YMF262 4-op mode (real feature gap; OPL3 4-op pairs aren't
   implemented).
4. ⏭ SN76489 noise channel — the simplified LFSR is audible but
   not pitch-accurate against silicon.
5. ⏭ NES APU envelope simplification — affects sustained tones.
6. ⏭ Stale "placeholder" doc-comments (RF5C164) — clean up the
   text so audits don't keep mis-firing.

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

## Quick reference: status by category (revised)

```
FM-family   (10):  fixed 1 (YM2612) / implemented 8 / FFI-backed 1
                   + 1 known gap (YMF262 4-op mode)
PSG-family  (8):   audible+verified 2 (SN76489 tone, Hello World bass)
                   / audible-but-simplified 1 (NES APU envelope)
                   / audibility-untested 5
PCM-family  (7):   stale-placeholder-banner 1 (RF5C164 works in practice)
                   / audibility-untested 6
```

After the YM2612 fix, **no chip is a confirmed stub.** What's
missing is *audibility verification* for ~17 of the 25 emulators —
the same gap that let the YM2612 sine-stub live. Phase 2 of this
plan (fingerprint tests) directly closes that gap; the Phase-1
implementation work is much smaller than the original audit
suggested. Spending the next iteration on writing Layer-2 fingerprint
tests for every untested chip is higher leverage than rewriting any
single emulator from scratch.
