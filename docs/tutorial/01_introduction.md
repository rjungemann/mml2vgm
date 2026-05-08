# Page 1 — Introduction

← [Index](README.md) | [Next: Setting Up →](02_setting_up.md)

---

## What Is mml2vgm?

mml2vgm is an open-source MML-to-VGM compiler and IDE for composing chiptune
music targeting classic sound hardware: Sega Genesis / Mega Drive (YM2612 +
SN76489), PC-88 / PC-98 (YM2608 / OPN family), arcade boards, and many other
chips.

It originated as a Japanese Windows application (mml2vgmIDE by kuma4649) and
has since grown into a multi-platform ecosystem:

- **mml2vgm-rs** — a Rust command-line compiler
- **Browser IDE** — a zero-install web editor
- **egui desktop app** — a native GUI with live keyboard preview

## What Is MML?

MML stands for **Music Macro Language** — a text-based notation where notes,
lengths, octaves, and effects are expressed as short commands that are compiled
into binary data a sound chip can play back.

A simple melody looks like this:

```
'A1 T120 v100 l4 o4 c d e f g a b >c r1
```

That one line sets the tempo to 120 BPM, volume to 100, default note length to
a quarter note, octave to 4, and plays a C major scale followed by a rest.

## What Is VGM?

VGM (Video Game Music) is a register-dump format that stores the exact writes
sent to sound chips, playable in any VGM player with bit-perfect accuracy.
mml2vgm compiles your MML source into a `.vgm` file you can play, share, or
archive.

## Supported Sound Chips

| Family | Example Chips | Sound Type |
|--------|---------------|------------|
| OPN/OPN2 | YM2203, YM2612, YM2608 | 4-op FM |
| OPM | YM2151 | 4-op FM |
| OPL / OPL3 | YM3812, YMF262 | 2-op FM |
| PSG / DCSG | AY8910, SN76489 | Square wave |
| PCM | RF5C164, SegaPCM, C140 | Sampled audio |
| Wavetable | HuC6280, K051649 | Waveform memory |

## The Three Tools

### Browser IDE

Zero-install. Open the URL in a modern browser, type MML, click **Compile**,
click **Play**. The service worker caches the app for offline use.

### mml2vgm-rs (CLI)

A Rust binary that compiles `.gwi` files to VGM on the command line. Ideal for
automation, scripting, and CI pipelines.

```sh
mml2vgm-rs examples/hello.gwi --play
```

### egui Desktop App

A native desktop application built with Rust and egui. Includes a file browser,
compilation log, and a MIDI keyboard panel for live note preview without
compiling.

## Who This Tutorial Is For

This tutorial is written for newcomers to MML who want to write chiptune music
targeting Genesis / PC-98-era hardware. Prior music theory knowledge is helpful
but not required. If you know what a quarter note is and can name the notes
C through B, you have everything you need to get started.

## Acknowledgments

mml2vgm-rs is based on the original Japanese mml2vgm IDE by **kuma4649**. The
MML syntax derives from the MUSIC LALF / PMD family of MML compilers common in
the Japanese PC-98 demoscene. Many thanks to contributors who have extended the
codebase, added chip support, and written documentation.

---

← [Index](README.md) | [Next: Setting Up →](02_setting_up.md)
