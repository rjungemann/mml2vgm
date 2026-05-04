# MML Sample Files

This directory contains sample MML files for testing the Browser IDE.

## Available Samples

| File | Description | Parts | Chips |
|------|-------------|-------|-------|
| `hello_world.gwi` | Simple melody with bass and drums | 3 | OPNA |
| `arpeggio.gwi` | Arpeggio patterns | 3 | OPNA |
| `drum_pattern.gwi` | Drum patterns (BD, SN, HH, Toms) | 3 | OPNA |
| `chord_progression.gwi` | Chord progression with melody | 4 | OPNA |

## How to Use

1. Open the Browser IDE
2. Click File → Open
3. Select a sample file from this directory
4. Click Compile (F5) to test compilation
5. Click Play to test audio playback

## File Formats

- **`.gwi`** - mml2vgm native format
- All samples use the OPNA (YM2608) sound chip
- All samples are in GWI format which supports:
  - Multiple parts/channels
  - FM synthesis (OPNA)
  - SSG (Square wave) channels
  - Drum sounds

## Creating Your Own Samples

1. Create a new file with `.gwi` extension
2. Start with `@OPNA` or another chip directive
3. Define parts with `@0`, `@1`, `@2`, etc.
4. Use standard MML commands for notes, volume, octave, etc.

Example:
```
@OPNA
@0 v100 l4 o4
  c d e f g a b >c2.
```

## Testing

These samples are used for:
- Verifying MML compilation
- Testing syntax highlighting
- Testing audio playback
- Testing part counter functionality
- Regression testing

## License

These sample files are part of the mml2vgm project and are licensed under GPL-3.0.
