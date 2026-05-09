# mml2vgm-rs Installation & CLI Usage Guide

## Installation

### From Source

```bash
cd mml2vgm-rs
cargo build --release
sudo install -Dm755 target/release/mml2vgm-rs /usr/local/bin/mml2vgm-rs
```

### Shell Completion Installation

#### Bash (version 4.1+)
```bash
sudo install -Dm644 completions/mml2vgm-rs.bash /usr/share/bash-completion/completions/mml2vgm-rs
```

After installation, reload bash:
```bash
exec bash
```

#### Zsh
```bash
sudo install -Dm644 completions/_mml2vgm-rs /usr/share/zsh/site-functions/_mml2vgm-rs
```

After installation, reload zsh:
```bash
exec zsh
```

### Man Page Installation

```bash
sudo install -Dm644 docs/mml2vgm-rs.1 /usr/share/man/man1/mml2vgm-rs.1
```

Update the man page database (optional but recommended):
```bash
sudo mandb
```

Now you can access the man page:
```bash
man mml2vgm-rs
```

## Environment Variables

The following environment variables can be used to set default behaviors:

### MML2VGM_COLORS
- **Type**: Boolean (0/1, true/false)
- **Default**: Auto-detect (TTY detection)
- **Effect**: Disable ANSI color codes in output
- **Example**: `export MML2VGM_COLORS=0`

### MML2VGM_QUIET
- **Type**: Boolean (0/1, true/false)
- **Default**: false
- **Effect**: Suppress all output except errors
- **Example**: `export MML2VGM_QUIET=1`

### MML2VGM_VERBOSE
- **Type**: Boolean (0/1, true/false)
- **Default**: false
- **Effect**: Enable verbose output by default
- **Example**: `export MML2VGM_VERBOSE=1`

### MML2VGM_PROGRESS
- **Type**: Boolean (0/1, true/false)
- **Default**: Auto-enabled for interactive TTY
- **Effect**: Show progress bars during compilation
- **Example**: `export MML2VGM_PROGRESS=1`

## Features

### 1. Color-Coded Output
The CLI automatically detects if your terminal supports ANSI colors and applies them. Disable with `--no-color` or `MML2VGM_COLORS=0`.

```bash
mml2vgm-rs song.gwi
```

### 2. Progress Indicators
Progress bars are shown during batch compilation and can be controlled with `--progress` or `MML2VGM_PROGRESS=1`.

```bash
mml2vgm-rs --batch ./mml_files --progress
```

### 3. Batch Compilation
Compile all .gwi files in a directory with a single command:

```bash
mml2vgm-rs --batch ./src_dir --format vgm --progress
```

### 4. Shell Completion
After installing completion scripts, use `Tab` to autocomplete options and filenames:

```bash
mml2vgm-rs --[TAB]              # Lists all options
mml2vgm-rs -c [TAB]              # Lists chip names
mml2vgm-rs song.[TAB]             # Lists .gwi files
```

### 5. Quiet Mode
Suppress output for clean scripting:

```bash
mml2vgm-rs input.gwi --quiet
echo $?  # Check exit code (0=success, 1=failure)
```

### 6. Verbose Output
See detailed compilation information:

```bash
mml2vgm-rs song.gwi --verbose
```

## Usage Examples

### Simple Compilation
```bash
mml2vgm-rs song.gwi
```

### Compile and Play
```bash
mml2vgm-rs song.gwi --play
```

### Batch Processing
```bash
mml2vgm-rs --batch ./songs --format vgm --progress
```

### Convert Multiple Formats
```bash
mml2vgm-rs song.gwi -f vgm -o song.vgm
mml2vgm-rs song.gwi -f xgm -o song.xgm
mml2vgm-rs song.gwi -f mid -o song.mid
```

### Target Specific Chips
```bash
mml2vgm-rs song.gwi -c YM2612 -c SN76489
```

### With Environment Variables
```bash
export MML2VGM_COLORS=0  # Disable colors
export MML2VGM_QUIET=1   # Quiet mode
mml2vgm-rs song.gwi
```

## Troubleshooting

### Colors not showing?
Try one of these:
```bash
unset MML2VGM_COLORS
mml2vgm-rs --no-color=false song.gwi
```

### Man page not found after installation?
Rebuild the man page database:
```bash
sudo mandb
```

### Shell completion not working?
1. Verify installation: `ls /usr/share/bash-completion/completions/mml2vgm-rs`
2. Restart shell: `exec bash` or `exec zsh`
3. Check shell configuration doesn't disable completions

## Architecture Notes

### Progress Bars
Progress bars are displayed during batch compilation with:
- Current/total file count
- Percentage completion
- Current file being compiled

### Color Support
Colors are detected via `atty::is(Stream::Stdout)` to ensure compatibility with:
- Terminals with color support
- Piped output (auto-disabled)
- Non-TTY environments

### Environment Variable Precedence
1. Command-line arguments (highest priority)
2. Environment variables
3. Built-in defaults (lowest priority)

## Future Enhancements

- [ ] Directory watching mode (--watch)
- [ ] Parallel compilation with rayon
- [ ] Configuration file support (~/.config/mml2vgm/mml2vgm.conf)
- [ ] Package distribution (DEB, RPM, AUR)
- [ ] TAB completion for include paths
