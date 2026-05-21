# Page 2 — Setting Up

← [Introduction](01_introduction.md) | [Next: Your First Song →](03_your_first_song.md)

---

## Choosing Your Tool

| Tool | Best For |
|------|----------|
| **Browser IDE** | First-timers, no install, instant feedback |
| **mml2vgm-rs CLI** | Automation, CI, scripting |
| **egui Desktop App** | Desktop use, live keyboard preview |

If you are new to mml2vgm, start with the **Browser IDE**. You can switch to
the CLI or desktop app later without changing anything about your `.gwi` files.

---

## Path A — Browser IDE

1. Open the Browser IDE URL in Chrome, Firefox, or Safari.
2. The editor opens with a sample song already loaded.
3. Click **Compile** (or press the keyboard shortcut shown in the toolbar) — a
   VGM file is generated entirely in the browser.
4. Click **Play** to hear it.

No account, no download, no install required.

**Offline use**: After your first visit the service worker caches the
application. Subsequent visits work without an internet connection.

---

## Path B — mml2vgm-rs (CLI)

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (`rustup install stable`)

### Build

```sh
git clone https://github.com/…/mml2vgm
cd mml2vgm/mml2vgm-rs
cargo build --release
```

The binary is placed at `target/release/mml2vgm-rs`.

### Basic Usage

```sh
# Compile a song and play immediately
./target/release/mml2vgm-rs examples/hello.gwi --play

# Compile to a VGM file
./target/release/mml2vgm-rs examples/hello.gwi -o hello.vgm

# Export to WAV
./target/release/mml2vgm-rs examples/hello.gwi --export-wav hello.wav
```

### Using Justfile shortcuts

The repository includes a `Justfile` with convenience targets:

```sh
just build        # build mml2vgm-rs in release mode
just play FILE    # compile and play a .gwi file
```

---

## Path C — egui Desktop App

### Prerequisites

- [Rust toolchain](https://rustup.rs/)

### Build

```sh
cd mml2vgm/egui-app
cargo build --release
./target/release/mml2vgm-egui
```

### Using the App

- Open a `.gwi` file with **File → Open** or drag-and-drop onto the window.
- Press **F5** (or the **Compile** button) to compile.
- Press **F9** (or **Play**) to hear the result.
- The **MIDI Keyboard** panel lets you preview notes on any channel without
  compiling — useful for auditioning FM patches while editing them.

---

## Verifying the Setup

Use `examples/hello.gwi` as a smoke test. It targets the Sega Genesis default
(YM2612 + SN76489) and plays a C major scale. If you hear the scale, your setup
is working correctly.

```sh
./target/release/mml2vgm-rs examples/hello.gwi --play
```

In the Browser IDE, click **Compile** then **Play** with the default sample
song.

---

## File Format Notes

- mml2vgm MML files use the **`.gwi`** extension.
- Encoding: **UTF-8** (with or without BOM; CRLF or LF are both accepted).
- Lines beginning with `'` (apostrophe) are interpreted as definitions or part
  sequences. All other lines are treated as comments.

---

← [Introduction](01_introduction.md) | [Next: Your First Song →](03_your_first_song.md)
