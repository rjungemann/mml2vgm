# Browser IDE Playback Gaps

Audit of `browser-ide/src/services/audioService.ts` and adjacent code, focused
on what stops compiled VGM streams from playing back faithfully in the browser.
Compiled fresh on 2026-06-13 against `main` (post commit 6355611e).

The reference test case throughout is `browser-ide/public/samples/hello_world.gwi`,
which exercises one YM2612 FM voice and one SN76489 PSG voice.

## Status legend
- ✅ Fixed in current branch
- 🔧 Tracked here, not yet implemented
- 📝 Hygiene / non-blocking

---

## ✅ Resolved this pass

### A. Chip detection from the VGM header
`playVGM` defaulted to `['YM2608','SN76489']` whenever the caller didn't pass
chips. Hello World's parts are `PartYM2612`/`PartSN76489`, the parser tags
every `0x52` write as `'YM2612'`, and `applyPendingVgmCommands` skipped any
command whose chip wasn't in `this.chips`. Net result: every FM register write
dropped on the floor.

Fix: `detectChipsFromVgmHeader()` walks the VGM 1.71 chip-clock table (24
entries) and returns every chip whose clock field is nonzero (top bit masked,
since it marks dual-chip use). `playVGM` now uses that as the primary set,
merged with any caller-supplied chips.

### B. Stereo de-interleave bug in the AudioWorklet
`chip_player_generate_samples(N)` returns `2*N` floats — interleaved
`L,R,L,R,...`. The worklet was reading 1 float per output frame and
replicating it across all output channels.

Effect: pitch dropped one octave (half rate) AND stereo collapsed to alternating
L/R samples. Worklet now reads two floats per output frame; producer aligns
its write index to even offsets and reserves a 2-float gap.

### C. AudioWorklet constructor `ReferenceError`
`thisbufferSize = 0;` (missing `.`) crashed the worklet constructor, so
playback was silent regardless of what the producer wrote. Now `this.bufferSize`.

### D. `Atomics.wait` on main thread / render thread
Both producer (`writeSamplesToRingBuffer`) and consumer (worklet `process()`)
called `Atomics.wait`, which throws on both thread types. Replaced with
non-blocking read-what-fits / write-what-fits loops; back-pressure handled by
the producer's `setTimeout(generateSamples, 0)` and the audio thread's
~3 ms `process()` cadence.

### E. MonacoEditor render storm
Three decoration effects stored Monaco decoration ID arrays in `useState` and
listed them in their own dependency arrays — every decoration update
re-triggered the effect. Moved to `useRef`; warnings cleared.

### F. MonacoEditor `editor._monaco` crashes
`editor._monaco` was never a Monaco API. Three call sites used it for
constructing `new Range(...)`. Replaced with the `monaco` instance from the
already-imported `useMonaco()` hook.

### H. Per-chip volume / mute / solo wired end-to-end
Added `ChipPlayer::set_chip_gain` / `get_chip_gain` (Rust), exposed as
`chip_player_set_chip_gain` (WASM), wrapped as `wasmService.setChipGain`,
and `AudioService.setChipVolume`/`setChipMuted`/`setChipSolo` now push the
combined effective gain (mute & solo collapse to gain=0) through to the
chip player on every change and on chip-player creation.

### Q. Source-map plumbed end-to-end + line numbers corrected
The Rust codegen has been pushing `NoteEvent`s into a `SourceMap` per note
all along — the browser never saw any because nothing read
`source_map_json` between the WASM boundary and the trace service.

**End-to-end wiring:**
- `wasmWrapper.extractResultData` now reads `source_map_json` from the
  `JsCompileResult` (defaults to empty string if the WASM build is older
  and doesn't expose it).
- `compilerWorker.compileMml` parses the JSON once on the worker thread,
  attaches the resulting `{events: [...]}` to the message under
  `source_map`, and logs the event count so the gap will be visible in
  the console if it ever reappears.
- `wasmService.compile` (the non-worker path used by some test fixtures)
  parses the same JSON and includes `source_map` in its `CompileResult`.
- `StoreCompileResult` gained the optional `source_map?: SourceMap` field;
  the store forwarder copies it from the worker payload. `App.tsx` was
  already reading `result.source_map` — the previously-flagged TS2339
  error on that access disappears now that the field exists.

**Line-number fix:** `preprocess_song_info` was stripping the
`'{ ... }` header block from the source before tokenisation, which
shifted every subsequent line up by the header height (13 lines for
Hello World). Source-map note events therefore pointed at e.g. line 20
(the FM instrument definition) instead of line 33 (the actual part
track). Fix: stripped lines are replaced with empty strings in the
preprocessed source rather than dropped, so line numbering is preserved
end-to-end. Verified with a Rust integration test that dumps source-map
events for `hello_world.gwi`: now reports `line=33` for the A1 melody
and `line=38` for the B1 bass, matching the editor positions.

### P. Ring-buffer capacity decoupled from outputChannels
`initSharedRingBuffer` was sizing capacity as
`sampleRate * outputChannels`. The chip player always emits stereo
interleaved (2 floats per frame) regardless of how many channels the
consumer renders, so the multiplier should track the *producer* format,
not the consumer's channel count.

Effect of the old formula at the configurable extremes:
- `outputChannels = 1` → capacity 44,100 floats = 22,050 frames =
  ~0.5 s of stereo audio. The producer still emits 2 floats per frame
  but the consumer indexed each as a sample, so a single producer tick
  of 4,096 frames (8,192 floats) would consume 8,192 of the 44,100
  capacity (~18.5%); workable but tight when 8 buffers are queued.
- `outputChannels = 4` → capacity 176,400 floats = 88,200 frames = 2 s.
  Just wasteful, but harmless.
- `outputChannels = 2` (default) → 88,200 floats = 1 s. Worked by
  accident because the stereo math happens to balance.

New formula: `capacityFrames = sampleRate * 1.0; capacity =
capacityFrames * 2` — explicit `STRIDE = 2` for chip-player stereo,
1 s of headroom regardless of consumer channel count. Log line now
reports both float count and frame count for clarity.

**Companion fix:** `captureWaveformSamples` had the same misuse — it
stepped through `samples` in increments of `outputChannels` and averaged
that many floats into one mono waveform point. Stride should be 2
(chip-player frame size). At `outputChannels=1` it would have stored
alternating L and R floats as separate waveform points (visual phase
artefact); at `outputChannels=4` it would have averaged across frame
boundaries (garbage).

Consumers that legitimately care about render channel count
(`AudioWorkletNode outputChannelCount`, `ScriptProcessor channelCount`,
the worklet's downmix logic) all still use `outputChannels` correctly.

### O. ScriptProcessor + non-SAB AudioWorklet fallback paths fixed
Three audio output paths now coexist and are individually correct:

- **Path A** (AudioWorklet + SharedArrayBuffer): lock-free SAB ring buffer,
  stereo de-interleaved by the worklet reading frame pairs. Unchanged
  from §B but stays the preferred path.
- **Path B** (AudioWorklet + postMessage): producer ships interleaved
  stereo `Float32Array` via `port.postMessage({type:'samples', samples})`;
  worklet maintains a small queue, reads frame pairs, silence-pads when
  the queue runs dry, caps at 8 buffers to bound memory if the audio
  thread stalls. Same stereo bug §B fixed for Path A had been latent
  here; now patched.
- **Path C** (ScriptProcessorNode): producer pushes onto an in-process
  queue (`fallbackSampleQueue`); `onaudioprocess` drains it identically
  to Path B's worklet logic. Caps at 8 buffers.

The producer-side dispatcher now distinguishes the three paths
explicitly: SAB ring → `writeSamplesToRingBuffer`; AudioWorkletNode (has
a `.port`) → `port.postMessage`; ScriptProcessor (no `.port`) →
`fallbackSampleQueue.push`. Previously it called `.port.postMessage`
unconditionally, which would throw `TypeError` the moment Path C
activated (ScriptProcessorNode has no port property).

Two bugs killed Path C end-to-end before:
- The producer always called `this.audioWorkletNode.port.postMessage`,
  but Path C's `audioWorkletNode` was a ScriptProcessorNode with no
  `.port` — synchronous TypeError per buffer.
- The audio callback read `this.sampleBuffer[bufferIndex++]` as if mono
  (replicating one float to all output channels) while the producer fed
  it interleaved stereo — same half-rate / phase-shifted pseudo-mono as
  §B in Path A.

Dropped now-orphaned state: the `sampleBuffer`/`bufferIndex` field pair
(replaced by the queue) and the never-declared `sampleQueue =` assignments
in `stop`/`destroy` (TS was already flagging them as TS2339).

Also surfaced: `YMF271` wasn't in the `SoundChip` type union, so the
parser case I added in §L (`push('YMF271', ...)`) had been a TypeScript
error my last typecheck filter was hiding. Added to the union.

### N. Dead vgmPlayerId / generateMoreSamples paths removed
- `vgmPlayerId` field deleted along with all three `if (this.vgmPlayerId)`
  branches in `startSampleGeneration`, `stop`, and `destroy`. The field was
  declared null and never assigned anything else, so all three checks
  were unreachable — the producer's "generate a silent buffer if VGM
  player exists" branch in particular would have been a foot-gun the
  moment anyone wired up an actual VGM player.
- `generateMoreSamples(count)` method removed. It ignored its `count` arg
  and just re-triggered the already-self-rescheduling
  `startSampleGeneration`. Its two call sites — the worklet's
  `needSamples` message handler and the ScriptProcessor fallback's
  per-buffer trigger — were also removed. The producer's
  `setTimeout(generateSamples, 0)` keeps the loop alive without help.

**Side fixes surfaced by `audioService.playback.test.ts`:**
- The `wasmService` mock didn't have `setChipGain` (added in §H), so the
  `createChipPlayer → pushEffectiveChipGains` call synchronously threw,
  killing `playMML` before `compile` ran. Added the mock entry.
- `createMinimalAudibleVgm()` built a stream with two register writes;
  the §J end-of-stream detection (added two passes ago) correctly stopped
  playback after one buffer, breaking the "stays audible for >0.5s"
  assertions. Extended the helper to emit 256 writes spaced 4000 samples
  apart — well over the producer's 130-buffer churn during the test's
  fake-timer window. Test now passes.

### M. Silence warning now reports the actual cause
Replaced the misleading "VGM command rendering to chip registers is likely
not implemented yet" text (which had been wrong since §A landed and is now
wrong for a different reason: every layer it indicted is working). The
warning still fires after 25 consecutive silent buffers, but now prints:

- the active chip set,
- applied vs. skipped register-write counts,
- VGM command count and sample-cursor / total,
- AudioContext state,
- a one-line `likely cause` derived from those observables.

`diagnoseSilence()` picks the highest-priority matching cause from a
checked-in-order list:
1. no AudioContext,
2. AudioContext suspended / interrupted (needs user gesture),
3. zero register-write commands in the parsed stream,
4. no chips registered with the player,
5. every command skipped because chip-detection didn't include the
   needed chip (lists the skipped chip names and what IS active),
6. reached end-of-stream with loop disabled (the "expected silence" case),
7. writes are landing but chip is rendering silence (instrument /
   key-on / volume problem inside the chip emulator).

The fallback "unable to attribute" branch only fires when the observable
state doesn't match any of the known patterns — which should be the cue
to add a new pattern rather than to ignore the message.

### L. VGM parser handles all VGM 1.71 opcodes
`parseVgmStream` no longer truncates the stream on unknown opcodes. Added:

- A `vgmOperandSize(cmd)` helper that returns the operand byte count for
  every command in the VGM 1.71 opcode table (single-byte commands → 0,
  PSG/FM family by sub-range, DAC stream control 0x90-0x95 by exact code,
  2-byte 0xA0-0xBF, 3-byte 0xC0-0xDF, 4-byte 0xE0-0xFF, `0x67` data block
  flagged as variable-length so the caller's existing branch handles it).
- Explicit register-write emission for the chips the Rust codegen actually
  produces opcodes for: `0xA0` AY8910, `0xB1` RF5C164, `0xB3` DMG, `0xB4`
  NES, `0xB6` VRC6 (mirroring the codegen's deviation from strict
  spec — strict 0xB6 is uPD7759), `0xB9` HuC6280, `0xBA` K053260, `0xBB`
  POKEY, `0xC4` QSound, `0xD1` YMF271, `0xD2` K051649, `0xD3` K054539,
  `0xD4` C140. 3-byte forms drop the high-byte/port info to fit the
  chip-player's 8-bit `(addr, data)` API; single-port songs play
  correctly, multi-port/wide-addr files lose precision (tracked separately
  if it ever becomes an audible issue).
- `0x80-0x8F` (YM2612 DAC + short wait) now advances `currentTime` by
  `cmd & 0x0F` samples. The DAC byte itself still isn't pushed (we'd need
  PCM-data-block cursor tracking for that), but the stream stays in sync
  so non-DAC writes after a DAC burst no longer mistime.
- Default branch consumes `vgmOperandSize(cmd)` bytes silently instead of
  bailing. Only a truly out-of-range value (which can't happen for a
  valid VGM byte but is possible on a corrupted stream) returns the
  partial result.

Off-by-one fix surfaced while testing: the size table initially had
`0x50-0x5F → 2` but per spec `0x50` (SN76489) is single-operand. Caught
by a synthetic parser pass over `hello_world.vgm`, `05_loops.vgm`, and
the `$`-using `loop_test.vgm` — all three now parse end-to-end with no
bail (verified opcode histograms match the codegen's reported command
counts).

### K. Rust codegen emits VGM loop offsets from `$` markers
Pairs with §J/§2. Added `Token::LoopMarker` to the lexer (recognises `$`),
`MmlNode::LoopMarker` to the AST, and a parser arm inside the part-body
dispatcher. VGM codegen records the latest `$`-marker time across all
parts (`VgmGenerator::loop_time`), seeds `time_checkpoints` with that
value so the wait-splitter creates a clean event boundary, then in
`calculate_header` scans the merged command stream for the first non-wait
command at or after `loop_time` and writes:
- `0x1C` (loop_offset) = `byte_offset - 0x1C` per VGM 1.71 §3,
- `0x20` (loop_samples) = `total_samples - loop_time`.

Files without a `$` marker keep both fields at zero (verified
byte-identical to pre-change output for `hello_world.gwi` and
`05_loops.gwi`). All 25 existing `vgm_codegen_accuracy` tests pass.

Editor highlighting: Monarch rule + theme entry for `keyword.loopMarker`
so `$` shows up distinctly in the part-track lane (purple in both dark
and light themes).

End-to-end check on a hand-written `'B1 T120 v100 l4 o4 c d $ e f`:
total_samples = 88200 (2 s), loop_offset = 0xF6 (loop body starts at file
byte 0x112, which is the first SN76489 register write after the
intro wait), loop_samples = 44100 (1 s). The browser playback path from
§J consumes those fields and wraps correctly.

### J. VGM loop-point handling on the browser side
`parseVgmCommands` (renamed `parseVgmStream`) now reads header fields 0x18
(total samples), 0x1C (loop offset relative to 0x1C), and 0x20 (loop
samples), and records which command index marks the loop body's first
write. A loop offset that lands past the last command is collapsed to "no
loop" so a misaligned marker can't spin the playback loop.

`applyPendingVgmCommands` now tracks an accumulated `vgmLoopShift` so
`targetSample` (driven by wall-clock audio time) stays monotonic while the
VGM cursor wraps. When the stream cursor reaches end-of-commands or
`vgmTotalSamples`:
- if `loop=true` and a loop offset is present: rewind
  `nextVgmCommandIndex` to the loop entry and bump `vgmLoopShift` by
  `vgmLoopSamples`, then resume applying register writes;
- otherwise: emit `onEnd` and stop the sample loop.

The outer loop runs up to 8 wraps per call so a loop body shorter than one
audio quantum still advances, but a degenerate zero-sample loop can't
freeze the JS thread. Chip register state is intentionally not reset on
wrap — the loop body inherits whatever instrument/volume state the
pre-loop intro left behind, matching VGM playback semantics.

**Caveat:** the Rust codegen does not currently populate header 0x1C/0x20.
`(...)N` finite repeats in C# MML get fully unrolled into the command
stream, so the browser-side loop-back path is dormant until a codegen
change emits a real loop offset. See open item #14.

### I. Multi-chip mix was dropping all but the last chip
Discovered while implementing §H. Every chip's `generate_samples`
implementation (verified across all 27 chips: `ym2612`, `sn76489`,
`ym2608`, `ym2151`, `ym2203`, `ym3526`, `y8950`, `ym3812`, `ymf262`,
`ymf271`, `ym2413`, `segapcm`, `rf5c164`, `huc6280`, `c140`, `c352`,
`ay8910`, `k051649`, `k053260`, `k054539`, `qsound`, `nes_apu`, `pokey`,
`dmg`, `vrc6`, plus `SilentChip`) writes its output with `frame[0] = left`
/ `frame[1] = right` — an **assignment**, not an accumulator. `ChipPlayer`
was calling them in a loop over the same shared `sample_buffer`, so each
chip silently clobbered whatever the previous chip wrote. Net audible
behaviour: whichever chip the `HashMap` happened to iterate last won; all
other chips were inaudible regardless of register state.

For Hello World specifically, with the chip-detection fix from §A enabled,
this meant the user would hear *either* the FM melody *or* the PSG bass
but never both — depending on hash order from one process startup to the
next.

Fix is part of §H: `ChipPlayer::generate_samples` now renders each chip
into a reusable `chip_scratch` buffer, then mixes that into
`sample_buffer` (with the chip's gain applied). Final mix is clamped to
`[-1.0, 1.0]` to avoid clipping when multiple loud chips overlap.

This bug had been latent since multi-chip support landed; it was masked
by §A (every multi-chip Hello-World test silently dropped the YM2612
writes, leaving only the SN76489 audible — which is exactly the scenario
where a single-chip-wins mixer is indistinguishable from a correct one).

### G. MML Monarch tokenizer mismatch with C# format
Earlier tokenizer assumed a hallucinated `@OPNA`/`@0` dialect, marked every
`'`-prefixed line as `invalid`, ignored case (collapsing global vs. per-part
commands), and didn't recognise header blocks, instrument-definition lines,
part-track labels, `>`/`<` octave shifts, or `+`/`-` accidentals. Rewrote
around the real C# dialect with a state-based tokenizer; theme extended.

---

## 🔧 Open gaps (in rough priority order)

### 1. ✅ Per-chip volume / mute / solo — DONE (see resolved §H)

### 2. ✅ VGM loop-point handling (browser side) — DONE (see resolved §J)

### 3. ✅ VGM parser handles all 1.71 opcodes — DONE (see resolved §L)

### 4. ✅ Silence warning copy reflects current causes — DONE (see resolved §M)

### 5. ✅ Dead vgmPlayerId branch removed — DONE (see resolved §N)

### 6. ✅ Dead generateMoreSamples removed — DONE (see resolved §N)

### 7. ✅ ScriptProcessor + non-SAB fallbacks repaired — DONE (see resolved §O)

### 8. ✅ Ring-buffer capacity decoupled from outputChannels — DONE (see resolved §P)

### 9. ✅ Source-map plumbed end-to-end + line numbers correct — DONE (see resolved §Q)

### 10. `chips_used` returns `[]` from the WASM result
Per the log: `chips_used(): []`. Informational since we now detect from the
header, but it's a regression in the WASM bindings worth tracking. Likely a
metadata-extraction bug, not a codegen bug.

### 11. Compile encoding always sends `"utf-8-bom"`
The C# tool emitted BOMs; the Rust parser ought to consume them transparently.
Confirm `"utf-8"` produces identical output and drop the BOM dance.

---

### 14. ✅ Rust codegen emits VGM loop offsets — DONE (see resolved §K)

## 📝 Lower priority / cosmetic

### 12. Verify `Octave-Rev` semantics
`hello_world.gwi` sets `Octave-Rev = FALSE`. Confirm the Rust parser actually
applies the flag (`>`/`<` swap) and doesn't silently ignore unknown options.

### 13. `parseVgmCommands` allocation profile
One `ParsedVgmCommand` object per write. Fine for Hello World (111
commands). A 30-second YM2608+ADPCM piece could be tens of thousands. A
typed-array layout (`Int32Array` of `[time, chipIdx, addr, data]`) would
cut both parse time and GC pressure if it ever shows up in profiles.

---

## Working order

The next three items by user-visible impact:

1. **#1 Per-chip volume / mute / solo** — already wired into the UI, currently
   no-op. Picking this first means visible control changes start mattering
   immediately.
2. **#2 VGM loop point** — needed before any "loop" UI control feels honest.
3. **#3 VGM parser coverage** — required for sample-driven content
   (`PCM` instrument definitions, YM2608 ADPCM).
