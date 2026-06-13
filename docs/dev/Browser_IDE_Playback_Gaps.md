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

### 3. VGM parser bails on unhandled commands
`default: return commands;` truncates the command stream on any unknown opcode.
Hello World only uses `0x50`, `0x52`, `0x61`, but real files will hit:

| Opcode  | Bytes | Meaning                                  | Frequency |
|---------|-------|------------------------------------------|-----------|
| `0x4F`  | 1     | Game Gear PSG stereo                     | rare      |
| `0x80-0x8F` | 0   | YM2612 DAC write + short wait (0-15)     | **any FM PCM** |
| `0xE0`  | 4     | PCM data block seek                      | **any sample using PCM** |
| `0x90-0x95` | varies | DAC stream control                    | sample-heavy songs |
| `0x30-0x3F` | 1   | Reserved single-byte (consume 1 data)    | future-proofing |
| `0x68`  | 11    | PCM RAM write                            | uncommon |
| `0xA0-0xAF` | 2   | AY8910 write                             | only AY files |

The `0x80-0x8F` family hides a wait of `n` samples (where `n = cmd & 0x0F`)
plus a DAC write of `2A` (YM2612 register `0x2A`, port 0), which is what
makes 8-bit DAC samples on the Genesis sing. Forgetting it makes PCM-heavy
demos sound like glitchy silence.

### 4. Stale "Generated samples remain silent" warning
Now misleading. The warning fires when 25+ buffers in a row have peak
amplitude < 1e-6. Text should point at the real causes today:
- Chip detected from header isn't in the player set,
- All YM2612 writes routed to a chip that isn't `'YM2612'`,
- VGM stream truncated at byte 0,
- AudioContext suspended without user gesture.

### 5. Dead `vgmPlayerId` branch in `startSampleGeneration`
```ts
if (this.vgmPlayerId) {
  samples = new Float32Array(this.bufferSize * this.outputChannels);  // ← silence
} else if (this.chipPlayerId) { ... }
```
Either delete (we only use the chip player path) or wire to a real VGM
player. Today: foot-gun waiting for someone to set `vgmPlayerId`.

### 6. `generateMoreSamples(count)` is dead
Only called by the ScriptProcessor fallback; `count` ignored; just kicks the
already-self-rescheduling `startSampleGeneration`. Remove.

### 7. ScriptProcessor fallback is broken end-to-end
`setupScriptProcessorNode` uses `createScriptProcessor` and writes into
`this.sampleBuffer`, but the producer path posts samples via
`port.postMessage({type:'samples', samples})` to the worklet only when
`!usingSharedArrayBuffer`. The two halves disagree on the transport.
On a browser without SharedArrayBuffer, playback would silently produce
nothing.

### 8. `outputChannels` configurable but not robust
`AudioServiceOptions.outputChannels` defaults to 2, and the chip player
always emits stereo. Setting `outputChannels: 1` shrinks the ring buffer
(`sampleRate * outputChannels`) by half but the chip player still produces
2× the floats per call — guarantees underruns. Worklet downmix is now
correct, but ring-buffer sizing should track *chip output* (stereo, fixed),
not output channels.

### 9. Empty source-map / trace events
Log: `TraceService] Initialized with 2 parts and 0 source map events`. The
WASM compile result returns an empty source map, so the editor's active-note
highlighting never fires even when register writes work. Probably a
`mml2vgm-wasm` binding gap.

### 10. `chips_used` returns `[]` from the WASM result
Per the log: `chips_used(): []`. Informational since we now detect from the
header, but it's a regression in the WASM bindings worth tracking. Likely a
metadata-extraction bug, not a codegen bug.

### 11. Compile encoding always sends `"utf-8-bom"`
The C# tool emitted BOMs; the Rust parser ought to consume them transparently.
Confirm `"utf-8"` produces identical output and drop the BOM dance.

---

### 14. Rust codegen never emits VGM loop offsets
The browser-side loop-back path from §J/§2 is dormant because
`mml2vgm-rs/src/compiler/codegen/vgm.rs` emits header bytes `0x1C` (loop
offset) and `0x20` (loop sample count) as zero for every output. C# MML's
`(...)N` finite repeats get fully unrolled into the command stream.

For looping playback to actually trigger, the compiler needs an "infinite
loop"-style directive (or a `L` / `/` / `:` marker per common MML
conventions) that maps to:
1. A non-zero `0x1C` set to `(loop_body_first_byte - 0x1C)`,
2. `0x20` set to the loop body's sample length,
3. Optionally `0x18` (total samples) recalculated.

Trivially testable end-to-end once added: a Hello World variant with a
two-bar loop should keep playing past the 6-second mark in the browser.

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
