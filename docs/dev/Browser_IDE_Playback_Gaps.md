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

### X. Producer pacing re-tuned from Firefox profile
User-supplied Firefox performance profile of `01_fm_basics.gwi`
playback. Analysis of `audioService` self-time and `setTimeout
callback` marker durations on the COOP+COEP renderer thread:

- **Self-time hot spot:** `wasm-function[69]`
  (`chip_player_generate_samples`) at 31 % of main-thread samples —
  the FM emulator itself is the dominant CPU consumer. Generation
  is otherwise just float copies + ring writes.
- **Cycle duration:** mean 27.4 ms, p95 28.6 ms, p99 31.7 ms,
  **max 38.2 ms** per `generateSamples` setTimeout callback.
- **Inter-cycle gap (start→start):** median 63 ms, p95 95 ms,
  **max 127 ms** — and `127 ms > bufferMs (92.8 ms)`.
- **Main-thread `eventDelay`:** max 38 ms; no classical jank.

The 127 ms gap was the smoking gun. When a gap exceeds one
buffer's worth of audio, the ring drains more than the producer
refills on that iteration. Previous tuning (`TARGET_FILL_MS=150`,
sleep cap=bufferMs) wasn't tight enough.

Re-derived steady-state model: each cycle adds `bufferMs` of audio
in the gen step and drains `cycleDur + sleep`. For the buffer to
hold level, **sleep ≈ bufferMs − cycleDur ≈ 65–70 ms**.

New parameters:
- `TARGET_FILL_MS = 250` — gives 250 ms of slack ahead of the
  consumer, so a worst-case 127 ms gap still leaves ~120 ms in the
  ring (above the bufferMs floor) and the next cycle refills
  without underrun.
- `SLEEP_CAP_MS = 70` — set to match the steady-state sleep value.
  A slow cycle (38 ms) adapts itself: the pacer computes a shorter
  sleep that turn, but never sleeps longer than 70 ms, so the gap
  caps at `worstCycle + 70 ≈ 108 ms < bufferMs (92.8 ms ... well,
  almost; the buffer's existing 250 ms slack absorbs the last 15 ms).

Path B (postMessage worklet) cadence-paces at `SLEEP_CAP_MS`; Path
C (ScriptProcessor) reads the in-process queue directly. 146 tests
pass.

**Note:** the FM emulator's main-thread CPU cost remains the
underlying constraint. Offloading chip emulation to a Web Worker
would eliminate the cyclic main-thread blocking entirely; not in
scope here, but worth filing if hitches return on heavier
multi-chip samples.

### W. Producer hot-path desync & overrun: choppy / halting playback
User reported `01_fm_basics.gwi` "starts fairly clear and then quickly
gets more garbled until it seems to halt." Same class of artefact on
other examples.

Two root causes, in series:

**(1) Async overhead inside the inner loop.** `applyPendingVgmCommands`
was async and every `wasmService.writeChipRegister` call was `await`ed.
The underlying WASM call is synchronous; the await chain only existed
because the wrapper did `await this.ensureInitialized()` at every entry
point. With FM init bursts of 30-60 register writes at time 0 plus the
producer's own outer `await wasmService.generateSamples`, each buffer
cycle inserted dozens of microtask hops where React renders / other
main-thread tasks could interleave between our register writes and our
sample generation. The chip emulator effectively saw register state
change in the middle of producing a buffer.

**(2) Producer racing past wall-clock.** The cycle finished with
`setTimeout(generateSamples, 0)`. Browsers clamp nested-timeout zeros
to ~4 ms, so the producer fired ~250 cycles/s, generating ~93 ms of
audio per call → ~23× real-time. `vgmSampleCursor` advanced by
`bufferSize` every cycle regardless of how much wall-clock had
elapsed, so the cursor reached `vgmTotalSamples` after ~0.4 s of
wall-clock for a 10 s song. `applyPendingVgmCommands` then emitted
`onEnd`, flipped `_isPlaying = false`, and the producer stopped. The
ring buffer drained whatever was queued (≤ 1 s) and then went silent
— exactly the "starts clear then halts" pattern the user described.
The intermediate "garbled" period is the over-generation feedback:
samples were being thrown away when the ring filled, but cursor and
register-write timing still advanced, so the surviving samples
represented increasingly stale chip state.

**Fixes:**

1. `wasmService.writeChipRegisterSync(playerId, chip, addr, data)` and
   `wasmService.generateSamplesSync(playerId, numSamples)` — bypass the
   `await ensureInitialized()` chain. Both are safe to call after a
   successful `createChipPlayer`, which already awaited init.

2. `applyPendingVgmCommands` is now synchronous and uses
   `writeChipRegisterSync`. `startSampleGeneration`'s producer body is
   also fully synchronous; `await` removed.

3. `computeProducerDelayMs()` paces the next cycle to the ring-buffer
   fill level. Targets ~150 ms steady-state fill: sleep until the
   consumer drains to that level, then fire to refill. Cap the sleep
   at one bufferSize (~93 ms) so a brief consumer speed-up never starves
   us. Underfilled → fire immediately. Path B (postMessage worklet)
   has no fill back-channel so it cadence-paces at `bufferMs - 5 ms`;
   Path C reads the in-process queue directly.

Net effect: producer advances `vgmSampleCursor` at roughly wall-clock
rate, end-of-stream fires when the song genuinely ends rather than
0.4 s in, and the chip emulator sees register writes applied between
buffers instead of mid-buffer.

**Test fixtures:** `audioService.playback.test.ts` mock service got
`writeChipRegisterSync` and `generateSamplesSync` entries; the
`mockImplementation` calls now stamp both the async and sync variants.
146 tests pass.

### V. Parser allocation deferral (measurement-gated)
Measured against every `browser-ide/public/samples/*.gwi` file: the
largest emits 1,087 register-write commands (`35_ensemble`), with most
samples in the 100-500 range. JS object-per-write allocation at that
scale is sub-millisecond per parse and well below any audible
threshold — there's no profiler signal to justify the typed-array
rewrite.

Added a comment on `parseVgmStream` documenting:
- the current scale (so a future reader doesn't redo the same
  measurement),
- the realistic pessimal case to watch for (minutes-long YM2608
  ADPCM with 30k+ DAC events),
- the mechanical conversion path if the threshold is ever crossed
  (`applyPendingVgmCommands` is the only hot reader and would
  switch from object-field dereference to stride-4 typed-array lane
  indexing).

This is a deliberate non-optimisation, not an oversight; revisit
only when a profiler shows GC pressure on real content.

### U. Spurious chip-clock defaults zeroed
Follow-up surfaced during §R. `VgmHeader::default()` in
`mml2vgm-rs/src/compiler/codegen/mod.rs` was initialising
`sn76489_clock`, `ym2413_clock`, `ym2612_clock`, and `ym2151_clock` to
their nominal nominal rates (3.58 MHz / 7.67 MHz). The
`extract_chips` post-processing only ever **overwrites** the clocks for
chips actually referenced — it never clears clocks that weren't, so
files like Hello World ended up emitting non-zero YM2413 + YM2151
clocks even though no part used those chips. Downstream consumers
(both the browser-side `detectChipsFromVgmHeader` and the new Rust
`chips_from_vgm_header` from §R) read those clock fields verbatim and
faithfully reported the spurious chips as "in use", which then
instantiated emulators on the chip player that did nothing.

Fix: all chip-clock fields now default to 0. `extract_chips` populates
only the clocks for chips that are referenced. The SN76489/YM2612
nonzero defaults were also harmless redundancy — the `extract_chips`
empty-chips fallback (`self.chips = vec![YM2612, SN76489]`) sets them
to identical values anyway. The test `test_vgm_header_default` was
updated to assert all chip clocks zero by default.

**Hello World after the fix:**
- Header `0x10` (YM2413 clock): now `00 00 00 00`, was `99 9E 36 00`.
- Header `0x30` (YM2151 clock): now `00 00 00 00`, was `99 9E 36 00`.
- `CompileInfo.chips_used`: now `[SN76489, YM2612]`, was
  `[SN76489, YM2413, YM2612, YM2151]`.
- Browser-side detection sees the same `[SN76489, YM2612]` and creates
  exactly two chip emulators.

All 581 integration tests + 25 vgm_codegen_accuracy tests pass.

### T. Octave-Rev header flag implemented at parse time
The `Octave-Rev = TRUE` header in the C# MML dialect swaps the meaning of
`>` and `<` so that `>` shifts the octave down. The Rust compiler was
parsing the value into `ast.metadata` but no codepath read it, so
`Octave-Rev = FALSE` and `Octave-Rev = TRUE` produced byte-identical
output.

**Where the flip has to happen:** the parser, not the codegen. Notes
bake their octave at parse time (`Note::new(letter, accidental,
self.current_octave)`), so codegen sees a fully-resolved octave per
note — by the time `MmlNode::OctaveShift` reaches the codegen handler,
the surrounding `Note` octaves have already been decided. A
codegen-side flip would only desync `state.octave` from the
already-resolved `Note.octave` values.

**Implementation:**
- `Parser` gained an `octave_reversed: bool` field + a
  `set_octave_reversed(bool)` setter.
- `Token::GreaterThan` and `Token::LessThan` arms now branch on
  `octave_reversed` and emit `OctaveShift::Up` / `OctaveShift::Down`
  with directionally-correct `current_octave` updates either way.
- `MmlCompiler::parse_with_metadata(tokens, &metadata)` reads
  `metadata["Octave-Rev"]` (`TRUE` case-insensitive) and configures the
  parser before parsing.
- All three `compile_from_source*` paths switched to
  `parse_with_metadata`; `validate_from_source` left on the old
  `parse()` since it doesn't render notes.

**Verified against a hand-written `'B1 o4 c >c <c`:**
- `Octave-Rev = FALSE` → middle note at octave 5 (`50 86 50 0d ...`,
  smaller SN76489 divider).
- `Octave-Rev = TRUE` → middle note at octave 3 (`50 87 50 35 ...`,
  larger divider).

Hello World's compiled output is byte-identical to before the change
(it declares `Octave-Rev = FALSE`, which matches the default). 25
`vgm_codegen_accuracy` and 37 `parser_regression` tests still pass; the
pre-existing failing live_player doctest is unrelated.

### S. Dead encoding option dropped
`CompileOptions.encoding` defaulted to `"utf-8-bom"` and was never read
anywhere in the compile path. The lexer accepted both BOMed and
non-BOMed sources via `normalize_source`'s unconditional
`source.strip_prefix('\u{FEFF}')`, so the option value had zero
behavioural effect. Removed the field from:

- `mml2vgm-rs/src/lib.rs` — struct field + `Default` impl + the
  `default_encoding()` helper.
- `browser-ide/src/types/index.ts` — TS `CompileOptions.encoding?` shape.

`#[serde(default)]` on neighbouring fields keeps options JSON
backward-compatible: any existing caller that still includes
`"encoding": "utf-8-bom"` deserialises fine (serde just ignores the
unknown key now). BOM handling stays — `normalize_source` keeps its
`strip_prefix('\u{FEFF}')` since that's the actual desired behaviour;
it just doesn't need an option string to switch on.

The `Document.encoding` field (UI display: "UTF-8" in the status bar)
is a separate concept and stays unchanged.

### R. CompileInfo.chips_used populated from VGM header
`info_from_vgm` filled the rest of `CompileInfo` (part count, command
count, duration) but left `chips_used` at `Vec::default()` (empty), so the
WASM `chips_used()` getter always returned `[]` and the browser's log
line `chips_used(): []` was reporting truthfully.

Added `chips_from_vgm_header(data)` that walks the VGM 1.71 chip-clock
table (25 entries) and returns every chip whose clock field is nonzero
(top bit masked, since VGM uses it for the dual-chip flag). The entry
table is gated by header `version` so older VGMs don't misread reserved
bytes. `info_from_vgm` now calls it to populate `chips_used` whenever
the header magic checks out.

The browser-side `detectChipsFromVgmHeader` already runs the same scan
in `audioService.ts` — the two layers are now in sync and both report
exactly what the binary header declares. For Hello World that's
`[SN76489, YM2413, YM2612, YM2151]` instead of `[]`. The YM2413 and
YM2151 entries reflect a separate codegen-side bug (those clocks are
non-zero in the emitted header even though the parts don't use those
chips) — that's a Rust codegen cleanup, not a CompileInfo issue.

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

### 10. ✅ chips_used populated from VGM header — DONE (see resolved §R)
### 10a. ✅ Spurious YM2413/YM2151 clock defaults zeroed — DONE (see resolved §U)

### 11. ✅ Dead encoding option dropped — DONE (see resolved §S)

---

### 14. ✅ Rust codegen emits VGM loop offsets — DONE (see resolved §K)

## 📝 Lower priority / cosmetic

### 12. ✅ Octave-Rev applied at parse time — DONE (see resolved §T)

### 13. ✅ Parser allocation deliberately deferred — DONE (see resolved §V)

---

## Working order

The next three items by user-visible impact:

1. **#1 Per-chip volume / mute / solo** — already wired into the UI, currently
   no-op. Picking this first means visible control changes start mattering
   immediately.
2. **#2 VGM loop point** — needed before any "loop" UI control feels honest.
3. **#3 VGM parser coverage** — required for sample-driven content
   (`PCM` instrument definitions, YM2608 ADPCM).
