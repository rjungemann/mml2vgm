# Golden Master Silence Remediation Plan

**Date**: 2026-05-20  
**Status**: In Progress  
**Scope**: 31 silent WAV files + 1 corrupt file found in `tests/golden_master/references/`

Run `just test-silence` to reproduce the current state.

---

## Summary

45 golden master reference WAVs were checked for silence with `scripts/detect_silence.mjs`.
The generation script (`tools/validation/run_golden_master_tests.py`) declared all 43 files
"OK" based on `vgm2wav` exit code alone — it does not check whether the rendered audio is
non-zero. That validation gap means 31 files silently (pun intended) contain all-zero samples.

**Working (13):** NES ×3, C140 ×2, C352 basic ×1, DMG noise + wave ×2, YM2151 ×4, brandish2_fm ×1  
**Silent (31):** see categories below  
**Corrupt (1):** `tier2/c352/test_c352_filter.wav` — chunk size exceeds file length

---

## Category A — FM chips that compile but render silent (18 files)

These are synthesis chips; they have no dependency on sample ROM. Silence here means the
compiler is producing incorrect or incomplete VGM register sequences.

| File | Chip | VGM opcode |
|------|------|------------|
| `opl/test_opl2_basic.wav` | YM3812 | 0x5A |
| `opl/test_opl3_4op.wav` | YMF262 | 0x5E/0x5F |
| `opl/test_opl_envelope.wav` | YM3812 | 0x5A |
| `ym2203/test_ym2203_fm.wav` | YM2203 | 0x55 |
| `ym2203/test_ym2203_mixed.wav` | YM2203 | 0x55 |
| `ym2203/test_ym2203_ssg.wav` | YM2203 | 0x55 |
| `ym2608/test_ym2608_adpcm.wav` | YM2608 | 0x56/0x57 |
| `ym2608/test_ym2608_fm.wav` | YM2608 | 0x56/0x57 |
| `ym2608/test_ym2608_ssg.wav` | YM2608 | 0x56/0x57 |
| `tier2/y8950/test_y8950_adpcm.wav` | Y8950 | 0x5C |
| `tier2/y8950/test_y8950_opl.wav` | Y8950 | 0x5C |
| `tier2/ym2413/test_ym2413_custom.wav` | YM2413 | 0x51 |
| `tier2/ym2413/test_ym2413_patches.wav` | YM2413 | 0x51 |
| `tier2/ym2413/test_ym2413_rhythm.wav` | YM2413 | 0x51 |
| `tier2/ay8910/test_ay8910_envelope.wav` | AY8910 | 0xA0 |
| `tier2/ay8910/test_ay8910_wavetable.wav` | AY8910 | 0xA0 |
| `tier2/huc6280/test_huc6280_wavetable.wav` | HuC6280 | 0xB9 |
| `ym2151/sf2_envelope.wav` | YM2151 | 0x54 |

**Note on AY8910 and HuC6280**: `vgm2wav` reports only 4 default devices for these VGMs
(SN76489, YM2612, RF5C164 stereo, SN76489). The target chip does not appear in the device
list, which means the VGM header clock field is 0 at render time. Either the `ay8910_clock`
/ `huc6280_clock` header field is not being written, or it is overwritten before serialization.

**Note on sf2_envelope**: This test was designed for SoundFont-driven synthesis; the
YM2151 codegen may not implement SF2 envelope mapping. Treat as a separate investigation
from the standard FM tests.

### Investigation steps (per chip)

1. Compile the `.gwi` to VGM:
   ```
   cargo run --manifest-path mml2vgm-rs/Cargo.toml -- \
     tests/golden_master/tier1/test_opl2_basic.gwi -o /tmp/test.vgm
   ```
2. Confirm the target chip appears in `vgm2wav` device list:
   ```
   /path/to/vgm2wav /tmp/test.vgm /tmp/test.wav 2>&1 | grep "Dev "
   ```
   If the chip is absent, the VGM header clock is 0 — fix header serialization.
3. If the chip is present, hexdump the VGM and verify:
   - At least one write to the note-on register (e.g., OPL2 reg 0xB0 bit 5 set)
   - Operator volume registers are not all 0x3F (full attenuation)
4. Listen to the rendered WAV.

### Fix approach

- **Header clock missing**: Trace `SoundChip::YM3812 => self.header.ym3812_clock = ...` in
  `codegen/vgm.rs` through to the `VgmHeader` serializer. Confirm the field offset matches
  the VGM 1.71 spec (offsets differ between version 1.50 and 1.71 for extended chip fields).
- **Note-on never written**: Add key-on emit to `opl_note_on` / `ym2203_note_on` etc.
  Verify `TL` (total level) for all 4 operators is non-0x3F on init.
- **AY8910 mixer not enabled**: Mixer register (0x07) must have tone bits clear for the
  active channels. Verify `opl_global_init` / equivalent sets this.

---

## Category B — PSG / wavetable / pulse chips that should produce audio (5 files)

Simple digital chips with no ROM dependency. Silence here is a codegen bug.

| File | Chip | VGM opcode | Working sibling |
|------|------|------------|-----------------|
| `tier3/dmg/test_dmg_pulse.wav` | DMG CH1/CH2 | 0xB3 | `test_dmg_noise`, `test_dmg_wave` pass |
| `tier3/vrc6/test_vrc6_pulse.wav` | VRC6 | 0xB6 | — |
| `tier3/k051649/test_k051649_wavetable.wav` | K051649 SCC | 0xD2 | — |
| `tier2/ay8910/test_ay8910_envelope.wav` | AY8910 | 0xA0 | — |
| `tier2/ay8910/test_ay8910_wavetable.wav` | AY8910 | 0xA0 | — |

**Note on DMG pulse**: `test_dmg_noise` and `test_dmg_wave` already pass, so the DMG
dispatch path is partially working. The pulse channel (CH1/CH2, 0xFF11/0xFF16) is likely
not being dispatched in the note-on handler when `dmg_ch` == 0 or 1.

**Note on K053260**: Listed here because K053260 is used for PCM *and* as an on-chip
ROM-less oscillator mode. Test files may be exercising only the PCM mode; revisit after
Category C is addressed.

### Investigation steps

1. Compile the `.gwi` and hexdump `0xB3` / `0xB6` / `0xD2` writes:
   ```
   xxd /tmp/test.vgm | grep -A1 "b3\|b6\|d2"
   ```
2. For DMG pulse: confirm `dmg_ch` is 0 or 1 and that registers 0xFF11, 0xFF12, 0xFF13,
   0xFF14 are written with non-zero volume (NR12 bits 7:4 ≠ 0).
3. For VRC6: confirm 0xB006 (pulse 1 freq hi + gate) has bit 7 set to enable the channel.
4. For K051649: confirm wavetable RAM is loaded before key-on.

---

## Category C — PCM sample-playback chips (10 files)

These chips require host-side sample ROM data loaded via a VGM data block. Without real
sample data they will always render silent. C140 and C352 basic produce audio because their
codegen writes synthetic register patterns that happen to trigger internal test tone logic.
The five chips below need actual sample data embedded in the VGM data block (opcode 0x67).

| File | Chip | VGM opcode |
|------|------|------------|
| `qsound/test_qsound_basic.wav` | QSound | 0xC4 |
| `qsound/test_qsound_echo.wav` | QSound | 0xC4 |
| `qsound/test_qsound_phase.wav` | QSound | 0xC4 |
| `segapcm/test_segapcm_basic.wav` | SegaPCM | 0xA4 |
| `segapcm/test_segapcm_pitch_sweep.wav` | SegaPCM | 0xA4 |
| `tier2/rf5c164/test_rf5c164_basic.wav` | RF5C164 | 0x68 |
| `tier2/rf5c164/test_rf5c164_pitch.wav` | RF5C164 | 0x68 |
| `tier2/k053260/test_k053260_basic.wav` | K053260 | 0xBA |
| `tier2/k053260/test_konami_pcm_pitch.wav` | K053260 | 0xBA |
| `tier2/k054539/test_k054539_basic.wav` | K054539 | 0xD3 |

### Fix approach

Two options, in order of preference:

**Option 1 — Embed a synthetic sample in the VGM data block**

Emit a VGM data block (0x67) containing a short, known-good PCM waveform (e.g., a single
cycle of a 440 Hz sine at the chip's native format: 8-bit signed for RF5C164, 8-bit unsigned
for SegaPCM, 16-bit signed for K054539). Point the channel start/end registers at this
block. This makes the test self-contained and does not require external ROM files.

This is the right approach for a golden master: the test encodes exactly the data it expects
to reproduce, independent of any external asset.

**Option 2 — Mark PCM tests as "register-write-only" in metadata**

If embedding synthetic samples is out of scope, update `tests/golden_master/metadata.json`
to flag these tests as `"validation_method": "register_writes_only"`. The golden master
runner already distinguishes WAV-comparison tests from register-count tests; these files
would be excluded from the silence check.

`just test-silence` should then accept an optional `--ignore-pcm` flag (or a manifest file
listing known-PCM-silent paths) so the test passes while the Category A/B bugs are the
actionable failures.

---

## Category D — Corrupt / edge-case files (2 files)

| File | Status | Note |
|------|--------|------|
| `tier2/c352/test_c352_filter.wav` | CORRUPT | data chunk size in header exceeds file length |
| `ym2151/sf2_envelope.wav` | SILENT | YM2151 + SoundFont mapping unimplemented |

**c352_filter**: The VGM likely compiled correctly (the VGM itself is fine); the corruption
is in the WAV written by `vgm2wav`. Regenerate it:
```
just test-golden   # re-runs vgm2wav for all tier1/2/3 tests
```
If the WAV is still corrupt after regeneration, the VGM itself has a bad data block that
causes `vgm2wav` to overrun the output buffer. Inspect with `xxd`.

**sf2_envelope**: Needs a design decision — either implement SF2 → YM2151 envelope mapping
in the compiler, or remove/replace this test with a standard YM2151 patch test.

---

## Remediation priority

| Priority | Group | Files | Complexity | Value |
|----------|-------|-------|------------|-------|
| 1 | VGM header clock fields (Cat A) | ~8 files | Low — header field write | Unblocks all FM chips |
| 2 | OPL note-on / operator init (Cat A) | 6 files | Medium | High-visibility chips |
| 3 | DMG pulse channel dispatch (Cat B) | 1 file | Low — known sibling works | Quick win |
| 4 | VRC6 / K051649 channel enable (Cat B) | 2 files | Low | Tier 3 completeness |
| 5 | Synthetic PCM sample data (Cat C) | 10 files | Medium — requires sample blocks | Completes tier 1/2 |
| 6 | c352_filter regeneration (Cat D) | 1 file | Trivial | Clean up |
| 7 | sf2_envelope design decision (Cat D) | 1 file | Unknown | Backlog |

---

## Immediate next step

Add a silence guard to the generation script so the "OK" verdict requires non-zero audio:

```python
# In tools/validation/run_golden_master_tests.py, after vgm2wav completes:
wav_data = open(wav_path, 'rb').read()
# Find data chunk and verify at least one non-zero sample
if all(b == 0 for b in wav_data[44:]):  # simplified; use proper chunk parsing
    result['ok'] = False
    result['error'] = 'rendered WAV is silent'
```

Or simply integrate `scripts/detect_silence.mjs` into the CI pipeline by adding
`just test-silence` to the `ci:` recipe in the Justfile.
