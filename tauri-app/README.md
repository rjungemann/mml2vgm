# mml2vgm Desktop

A desktop application for the mml2vgm Browser IDE, built with [Tauri](https://tauri.app/).

## Features

- Full mml2vgm Browser IDE experience as a native desktop app
- Cross-platform: Windows, macOS, Linux
- Tiny binary size (typically < 10MB)
- Native file dialogs for opening/saving files
- System tray integration (optional)
- File drop support
- Offline-capable with service worker caching

## Prerequisites

- Node.js 18+ 
- Rust 1.70+
- Tauri CLI: `npm install -g @tauri-apps/cli`

## Installation

```bash
# Clone the mml2vgm repository
# cd to the tauri-app directory

# Install dependencies
npm install

# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
npm install -g @tauri-apps/cli
```

## Development

```bash
# Run the development server
npm run tauri:dev

# Or for the Vite dev server only
npm run dev
```

## Building

```bash
# Build for development
npm run tauri:build

# Build for production (all platforms)
npm run tauri:build -- --all-features
```

## Running

```bash
# Run the production build
npm run tauri:preview
```

## Project Structure

```
tauri-app/
├── src/                    # Frontend source (TypeScript/React)
│   └── main.tsx            # App entry point (loads browser-ide)
├── src-tauri/              # Rust backend source
│   ├── src/
│   │   └── main.rs         # Tauri app entry point
│   └── Cargo.toml          # Rust package configuration
├── tauri.conf.json         # Tauri configuration
├── vite.config.ts          # Vite configuration
├── package.json            # Frontend dependencies
└── icons/                  # App icons
```

## Configuration

### tauri.conf.json

The main Tauri configuration file defines:
- App metadata (name, version, description)
- Build settings (frontend dist directory, dev command)
- Bundle settings (targets, identifier, icons)
- Security allowlist (file system access, dialogs, etc.)

### Capabilities

The app has the following permissions:
- **Dialog**: Open and save file dialogs
- **File System**: Read MML files, write VGM/XGM/ZGM/WAV files
- **Clipboard**: Read/write text for copy/paste
- **Notification**: Show system notifications

## Browser IDE Integration

The Tauri app serves the browser-ide from `../browser-ide/`. The frontend:
1. Uses the same React components as the web version
2. Has access to Tauri APIs for desktop features
3. Maintains full compatibility with the web version

## Desktop-Specific Features

### File System Access

```typescript
import { open, save } from '@tauri-apps/api/dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';

// Open a file
const filePath = await open({
  filters: [
    { name: 'MML Files', extensions: ['gwi', 'mml', 'muc', 'mdl', 'mus'] }
  ]
});

// Read a file
const content = await readTextFile(filePath);

// Save a file
const savePath = await save({
  filters: [{ name: 'VGM Files', extensions: ['vgm'] }]
});
await writeTextFile(savePath, vgmData);
```

### Notifications

```typescript
import { notify } from '@tauri-apps/api/notification';

await notify({
  title: 'Compilation Complete',
  body: 'Your MML file has been compiled successfully!'
});
```

## Icons

To generate icons from a source image:

```bash
npx @tauri-apps/cli icon path/to/source-icon.png
```

This will generate icons for all platforms:
- `icons/32x32.png` - 32x32
- `icons/128x128.png` - 128x128
- `icons/128x128@2x.png` - 256x256 (for macOS Retina)

For Windows, also add `icons/icon.ico`.

## Troubleshooting

### Build fails with Rust errors

Make sure you have Rust 1.70+ installed:

```bash
rustc --version
```

If not, update Rust:

```bash
rustup update
```

### Tauri commands not found

Install the Tauri CLI globally:

```bash
npm install -g @tauri-apps/cli
```

### WASM not loading

Ensure the following headers are set in your HTML:

```html
<meta http-equiv="Cross-Origin-Opener-Policy" content="same-origin">
<meta http-equiv="Cross-Origin-Embedder-Policy" content="require-corp">
```

These are required for SharedArrayBuffer, which Tauri enables by default.

## License

GPL-3.0 - See the LICENSE file in the root of the repository.
