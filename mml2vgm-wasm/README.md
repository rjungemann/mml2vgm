# mml2vgm-wasm

WebAssembly bindings for the mml2vgm compiler, enabling MML to VGM/XGM/ZGM compilation in the browser.

## Overview

This crate provides WebAssembly bindings for the [mml2vgm-rs](../mml2vgm-rs/) Rust library, allowing you to:

- Compile MML (Music Macro Language) to VGM/XGM/ZGM formats
- Validate MML syntax
- Tokenize MML for syntax highlighting
- Emulate sound chips in real-time for audio playback
- Play VGM files directly

All from within a web browser using WebAssembly.

## Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- Node.js (for using the generated bindings)

## Installation

```bash
# Install wasm-pack
cargo install wasm-pack

# Build the WASM module
wasm-pack build

# For production build with optimizations
wasm-pack build --release
```

## Usage

### In JavaScript

```javascript
import init, { compile_mml, get_supported_chips } from './pkg/mml2vgm_wasm.js';

// Initialize WASM
await init();

// Get supported chips
const chips = JSON.parse(get_supported_chips());
console.log(chips);

// Compile MML to VGM
const mml = `@0 v10 o4 l4 c4 d4 e4 f4 g4`;
const options = { format: 'vgm' };
const vgmData = compile_mml(mml, JSON.stringify(options));
console.log(`Compiled to ${vgmData.length} bytes`);
```

### In TypeScript

```typescript
import init, { 
    compile_mml, 
    validate_mml,
    tokenize,
    CompileOptions 
} from './pkg/mml2vgm_wasm';

await init();

const mml = `@0 v10 o4 l4 c4 d4 e4 f4`;
const options: CompileOptions = {
    format: 'vgm',
    clock_count: 192,
    verbose: false
};

const vgmData = compile_mml(mml, JSON.stringify(options));
```

## API Reference

### Compilation

| Function | Description |
|----------|-------------|
| `compile_mml(mml: string, options_json: string): Uint8Array` | Compile MML to binary format |
| `validate_mml(mml: string): void` | Validate MML syntax |
| `tokenize(mml: string): string` | Tokenize MML for syntax highlighting |

### Utilities

| Function | Description |
|----------|-------------|
| `get_supported_chips(): string` | Get JSON array of supported chips |
| `get_supported_formats(): string` | Get JSON array of supported formats |
| `parse_sound_chip(chip_name: string): object` | Parse chip name to object |
| `parse_output_format(format_name: string): object` | Parse format name to object |
| `default_compile_options(): string` | Get default options as JSON |
| `compile_options_for_format(format: string): string` | Get options for specific format |

### Chip Player (Real-time Audio)

| Function | Description |
|----------|-------------|
| `create_chip_player(sample_rate: number): JsChipPlayer` | Create a new chip player |
| `chip_player_add_chip(player, chip_name)` | Add a sound chip to the player |
| `chip_player_write_register(player, chip_name, addr, data)` | Write to a chip register |
| `chip_player_generate_samples(player, num_samples): Float32Array` | Generate audio samples |
| `chip_player_reset(player)` | Reset all chips |
| `chip_player_state(player): string` | Get player state ("stopped", "playing", "paused") |

### VGM Player

| Function | Description |
|----------|-------------|
| `create_vgm_player(): JsVgmPlayer` | Create a new VGM player |
| `vgm_player_load(player, data)` | Load VGM binary data |
| `vgm_player_play(player)` | Start playback |
| `vgm_player_stop(player)` | Stop playback |
| `vgm_player_state(player): string` | Get player state |
| `vgm_player_get_info(player): string` | Get VGM header info as JSON |

## Example: Compiling and Playing MML

```javascript
import init, { 
    compile_mml, 
    create_chip_player,
    chip_player_add_chip,
    chip_player_generate_samples,
    create_vgm_player,
    vgm_player_load,
    vgm_player_play
} from './pkg/mml2vgm_wasm.js';

await init();

// Compile MML to VGM
const mml = `{Title=Test}
'FM o4 v10 l4 c4 d4 e4 f4`;
const vgmData = compile_mml(mml, JSON.stringify({ format: 'vgm' }));

// Create and load VGM player
const vgmPlayer = create_vgm_player();
vgm_player_load(vgmPlayer, vgmData);

// Play the VGM
vgm_player_play(vgmPlayer);

// Or use chip player for real-time emulation
const chipPlayer = create_chip_player(44100);
chip_player_add_chip(chipPlayer, 'YM2612');

// Generate samples
const samples = chip_player_generate_samples(chipPlayer, 4096);
// Send samples to AudioContext for playback
```

## Compile Options

The `CompileOptions` object accepts the following properties:

```typescript
interface CompileOptions {
    format: 'vgm' | 'xgm' | 'xgm2' | 'zgm';
    target_chips?: string[];  // e.g., ['YM2612', 'SN76489']
    verbose?: boolean;
    debug?: boolean;
    output_trace?: boolean;
    compression?: number;  // 0-9
    encoding?: string;     // e.g., 'utf-8-bom'
    include_paths?: string[];
    clock_count?: number;
}
```

## Supported Sound Chips

The following sound chips are supported:

- YM2612 (OPN2) - Mega Drive/Genesis
- YM2612X, YM2612X2 - Extended variants
- SN76489 (DCSG) - PSG
- SN76489X2 - Dual PSG
- YM2608 (OPNA) - PC Engine/TurboGrafx-16
- YM2609 (OPNA2)
- YM2610B (OPNB)
- YM2151 (OPM)
- YM3526 (OPL)
- Y8950
- YM3812 (OPL2)
- YMF262 (OPL3)
- YM2413 (OPLL)
- YM2203 (OPN)
- RF5C164 - Sega CD PCM
- SegaPCM
- HuC6280
- C140
- C352
- AY8910
- K051649
- K053260
- K054539
- QSound
- NES APU
- DMG (Game Boy)
- VRC6
- POKEY (Atari)
- MIDI

## Supported Output Formats

- VGM - Standard Video Game Music format
- XGM - Extended Game Music format (Mega Drive)
- XGM2 - XGM with extended features
- ZGM - ZunG Music format

## Token Types

The `tokenize()` function returns tokens with the following types:

- `number` - Numeric literal
- `string` - String literal
- `identifier` - Identifier
- `note` - Musical note (C, D, E, F, G, A, B)
- `sharp` - Sharp (#)
- `flat` - Flat (b)
- `rest` - Rest (r)
- `duration` - Duration number
- `dot` - Dotted note modifier
- `tie` - Tie modifier (_)
- `octave_up` - Octave up (>)
- `octave_down` - Octave down (<)
- `octave_cmd` - Octave command (o)
- `volume_cmd` - Volume command (v)
- `tempo_cmd` - Tempo command (t)
- `length_cmd` - Length command (l)
- `part_cmd` - Part/Instrument command (@)
- `bar` - Bar line (|)
- `comment` - Comment
- `whitespace` - Whitespace
- `eof` - End of file
- Structure tokens: `left_brace`, `right_brace`, `apostrophe`, `equals`, `comma`, `left_bracket`, `right_bracket`, `left_paren`, `right_paren`

## Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| WebAssembly | ✅ | ✅ | ✅ | ✅ |
| AudioWorklet | ✅ 66+ | ✅ 65+ | ❌ | ✅ 79+ |
| Web MIDI API | ✅ 43+ | ✅ 46+ | ❌ | ✅ 79+ |
| File System Access API | ✅ 86+ | ✅ 111+ | ❌ | ✅ 86+ |

## Performance Considerations

### WASM Bundle Size

The initial WASM bundle will be large because it includes all chip emulators. To reduce size:

1. **Use feature flags:**
   ```bash
   wasm-pack build --features all-chips  # Only if you need all chips
   ```

2. **Optimize with wasm-opt:**
   ```bash
   wasm-opt -Oz -o output.wasm input.wasm
   ```

3. **Compress with gzip/brotli:**
   - WASM files compress very well (70-80% reduction)

### Audio Latency

For real-time audio playback:
- Use AudioWorklet for low-latency processing
- Implement double-buffering between WASM and JavaScript
- Use SharedArrayBuffer for zero-copy data transfer

## Development

### Testing

Run the test HTML page:

```bash
# Build WASM
wasm-pack build

# Start a local server
python3 -m http.server 8000

# Open test.html in browser
open http://localhost:8000/mml2vgm-wasm/test.html
```

### Debugging

Enable console logging in WASM:

```rust
#[wasm_bindgen(start)]
pub fn init_console() {
    console_log::init_with_level(log::Level::Trace).unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
```

Then use `console.log!` macros in Rust code, which will appear in the browser console.

## License

This project is licensed under the GPL-3.0 license. See [LICENSE.txt](../LICENSE.txt) for details.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
