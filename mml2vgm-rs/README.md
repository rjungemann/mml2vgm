# mml2vgm-rs

**Cross-platform CLI compiler for MML → VGM / XGM / XGM2 / ZGM**

`mml2vgm-rs` takes Music Macro Language (`.gwi`) score files and compiles them
into audio data files for Sega Mega Drive/Genesis and other retro hardware.

---

## Installation

### Homebrew (macOS / Linux)

```sh
brew tap rjungemann/maltese
brew install mml2vgm-rs --HEAD
```

### From source (all platforms)

```sh
git clone https://github.com/rjungemann/maltese
cd maltese/mml2vgm-rs
cargo build --release
```

The compiled binary is at `target/release/mml2vgm-rs`. Copy it anywhere on
your `$PATH`:

```sh
sudo install -Dm755 target/release/mml2vgm-rs /usr/local/bin/mml2vgm-rs
```

For shell completions and the man page see [`INSTALL.md`](INSTALL.md).

---

## Quick Start

```sh
# Compile a score to VGM (output defaults to song.vgm)
mml2vgm-rs song.gwi

# Play back immediately after compiling
mml2vgm-rs song.gwi --play

# Check syntax without producing output
mml2vgm-rs song.gwi --check
```

---

## Usage

```
mml2vgm-rs [OPTIONS] <INPUT>
```

### Options

| Flag | Short | Description |
|------|-------|-------------|
| `--output <PATH>` | `-o` | Output file path (default: `<input-stem>.<format>`) |
| `--format <FMT>` | `-f` | Output format: `vgm`, `xgm`, `xgm2`, `zgm` (default: `vgm`) |
| `--check` | | Validate only — no output file written |
| `--play` | | Compile then play the result |
| `--verbose` | `-v` | Print compilation details |
| `--debug` | | Very verbose internal tracing |
| `--quiet` | | Suppress all output except errors |
| `--no-color` | | Disable ANSI colours |
| `--chips <CHIP>` | `-c` | Restrict to specific chip(s), e.g. `-c YM2612 -c SN76489` |
| `--include <DIR>` | `-I` | Add an include search path |
| `--clock-count <N>` | | Override VGM clock count per frame |
| `--list-chips` | | Print all supported sound chips and exit |
| `--list-formats` | | Print all supported output formats and exit |
| `--version` | | Print version and exit |
| `--help` | `-h` | Print help and exit |

---

## Examples

### Compile to VGM

```sh
mml2vgm-rs my_song.gwi
# Writes my_song.vgm
```

### Compile to XGM (Mega Drive ROM-ready)

```sh
mml2vgm-rs my_song.gwi -f xgm -o my_song.xgm
```

### Validate syntax

```sh
mml2vgm-rs my_song.gwi --check -v
```

### Batch compile all `.gwi` files in a directory

```sh
for f in songs/*.gwi; do
  mml2vgm-rs "$f" -f vgm -o "out/$(basename "${f%.gwi}.vgm")"
done
```

### Target specific chips only

```sh
mml2vgm-rs my_song.gwi -c YM2612 -c SN76489
```

### Pipe-friendly quiet mode

```sh
mml2vgm-rs input.gwi --quiet && echo "ok" || echo "failed"
```

---

## Supported Output Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| `vgm` | `.vgm` | Video Game Music — universal retro audio container |
| `xgm` | `.xgm` | Mega Drive ROM-embedded music (SGDK / Blast16) |
| `xgm2` | `.xgm` | Extended XGM with additional features |
| `zgm` | `.zgm` | ZGM extended format |

---

## Environment Variables

| Variable | Default | Effect |
|----------|---------|--------|
| `MML2VGM_COLORS` | auto | `0` / `false` to disable ANSI colour output |
| `MML2VGM_QUIET` | false | `1` / `true` to suppress all non-error output |
| `MML2VGM_VERBOSE` | false | `1` / `true` to enable verbose output by default |
| `MML2VGM_PROGRESS` | auto (TTY) | `1` / `true` to force progress bars |

Command-line flags override environment variables.

---

## Editor Support

Syntax highlighting for `.gwi` / `.mml` files is available in
[`editors/`](../editors/):

| Editor | Location | Install |
|--------|----------|---------|
| **VS Code** | `editors/vscode/` | Copy folder to `~/.vscode/extensions/` or run `code --install-extension editors/vscode/` |
| **Vim / Neovim** | `editors/vim/` | Copy `syntax/` and `ftdetect/` files into `~/.vim/` |

---

## Troubleshooting

**`error: could not find Cargo.toml`** — Run from the `mml2vgm-rs/` subdirectory,
not the repository root.

**Colours not showing** — Try `unset MML2VGM_COLORS` or pass `--no-color=false`.

**Man page not found after install** — Run `sudo mandb` to rebuild the man page
database.

**Shell completion not working** — Verify the completion file is in the right
location and restart your shell (`exec bash` / `exec zsh`).

---

## Further Documentation

| Document | Content |
|----------|---------|
| [`INSTALL.md`](INSTALL.md) | Detailed install steps, shell completions, man page |
| [`docs/USAGE.md`](docs/USAGE.md) | Annotated usage examples |
| [`../docs/user/User_Manual.md`](../docs/user/User_Manual.md) | End-user MML guide |
| [`../docs/user/MML_Commands.md`](../docs/user/MML_Commands.md) | MML command reference |
| [`../docs/user/tutorial/`](../docs/user/tutorial/) | Step-by-step tutorial series |
