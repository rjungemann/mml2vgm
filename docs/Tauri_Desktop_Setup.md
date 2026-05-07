# Tauri Desktop App Setup Guide

> **⚠️ DEPRECATED** — The Tauri desktop app has been superseded by the native
> [egui desktop app](PLAN_egui_Desktop.md) (`egui-app/`).  
> Use `just egui-dev` (or `cd egui-app && cargo run`) to launch the new desktop app.  
> This document is retained for reference until `tauri-app/` is fully removed.

---

This guide explains how to set up and build the mml2vgm desktop application using [Tauri](https://tauri.app/).

## Overview

The **mml2vgm Desktop** app provides a native desktop experience for the mml2vgm Browser IDE. It uses:
- **Tauri 2.0** - Rust-based desktop framework
- **Vite + React** - Same frontend as the browser version
- **WASM** - WebAssembly for audio compilation

### Features

| Feature | Description |
|---------|-------------|
| Cross-platform | Windows, macOS, Linux |
| Small size | Typically < 10MB binary |
| Native feel | Native windows, menus, dialogs |
| File access | Open/save files with native dialogs |
| Offline support | Works without internet connection |
| File drop | Drag and drop MML files |
| Notifications | System notifications for events |

---

## Prerequisites

### 1. Node.js

**Required:** Node.js 18 or later

Check your version:
```bash
node --version
```

If you need to install/upgrade:
- Download from [https://nodejs.org/](https://nodejs.org/)
- Or use a version manager like `nvm`

### 2. Rust

**Required:** Rust 1.70 or later

Check your version:
```bash
rustc --version
```

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, add to your PATH:
```bash
source "$HOME/.cargo/env"
```

### 3. Tauri CLI

Install globally:
```bash
npm install -g @tauri-apps/cli
```

Verify:
```bash
tauri --version
```

---

## Project Structure

```
mml2vgm/
├── tauri-app/                    # Tauri desktop app
│   ├── src/                       # Frontend source
│   │   ├── main.tsx               # App entry point
│   │   └── index.css              # Global styles
│   ├── src-tauri/                 # Rust backend
│   │   ├── src/
│   │   │   └── main.rs            # Rust entry point
│   │   └── Cargo.toml             # Rust dependencies
│   ├── public/                    # Static assets (copied from browser-ide)
│   ├── icons/                     # App icons
│   ├── tauri.conf.json            # Tauri configuration
│   ├── vite.config.ts             # Vite configuration
│   ├── package.json               # Frontend dependencies
│   └── README.md                  # Tauri-specific docs
├── browser-ide/                   # Web IDE (shared code)
│   └── src/                       # React components, services, etc.
└── mml2vgm-wasm/                  # WASM module
    └── pkg/                        # Compiled WASM
```

The Tauri app **shares the frontend code** with the browser-ide, importing components directly from `../browser-ide/src/`.

---

## Setup

### Step 1: Clone the repository

```bash
git clone https://github.com/example/mml2vgm.git
cd mml2vgm
```

### Step 2: Install dependencies

```bash
# Install browser-ide dependencies
cd browser-ide
npm install

# Build the WASM module
cd ../mml2vgm-wasm
wasm-pack build --release

# Install Tauri app dependencies
cd ../tauri-app
npm install
```

### Step 3: (Optional) Run setup script

```bash
cd tauri-app
chmod +x setup.sh
./setup.sh
```

This will automatically check and install all prerequisites.

---

## Development

### Start the dev server

```bash
cd tauri-app
npm run tauri:dev
```

This will:
1. Start the Vite dev server on port 5173
2. Launch the Tauri app
3. Open a native window with hot-reload

### Run just the Vite dev server

```bash
npm run dev
```

Access at: `http://localhost:5173`

---

## Building

### Build for your current platform

```bash
npm run tauri:build
```

### Build for all platforms

```bash
npm run tauri:build -- --all-features
```

### Build for specific platforms

```bash
# Windows
npm run tauri:build -- --target x86_64-pc-windows-msvc

# macOS
npm run tauri:build -- --target x86_64-apple-darwin

# Linux
npm run tauri:build -- --target x86_64-unknown-linux-gnu
```

### Output locations

| Platform | Output Path |
|----------|-------------|
| Windows | `tauri-app/target/release/bundle/msvc/mml2vgm-desktop_0.1.0_x64_en-US.msi` |
| macOS | `tauri-app/target/release/bundle/dmg/mml2vgm.app` |
| Linux | `tauri-app/target/release/bundle/appimage/mml2vgm-desktop_0.1.0_amd64.AppImage` |

---

## Configuration

### tauri.conf.json

The main configuration file defines:

#### Build Settings

```json
{
  "build": {
    "frontends": [{
      "name": "mml2vgm-ide",
      "distDir": "dist",
      "devCommand": "vite dev",
      "devPath": "http://localhost:5173",
      "buildCommand": "vite build"
    }],
    "withGlobalTauri": true
  }
}
```

#### Bundle Settings

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "identifier": "com.mml2vgm.desktop",
    "icon": [{ "file": "icons/32x32.png", "sizes": [32] }]
  }
}
```

#### Window Settings

```json
{
  "tauri": {
    "windows": [{
      "title": "mml2vgm",
      "width": 1280,
      "height": 800,
      "minWidth": 800,
      "minHeight": 600,
      "visible": true,
      "resizable": true,
      "fileDropEnabled": true
    }]
  }
}
```

### Security Allowlist

The app requests the following permissions:

| Permission | Scope | Purpose |
|------------|-------|---------|
| `dialog` | All | File open/save dialogs |
| `fs:read` | `*.gwi`, `*.mml`, `*.muc`, `*.mdl`, `*.mus`, `*.txt` | Load MML source files |
| `fs:write` | `*.vgm`, `*.xgm`, `*.zgm`, `*.wav` | Save compiled output |
| `clipboard` | All | Copy/paste support |
| `notification` | All | System notifications |

---

## Desktop-Specific Features

### File System Access

The app has restricted file system access:

- **Read**: MML source files (`*.gwi`, `*.mml`, `*.muc`, `*.mdl`, `*.mus`, `*.txt`)
- **Write**: Compiled output (`*.vgm`, `*.xgm`, `*.zgm`, `*.wav`)

### File Dialogs

Native open/save dialogs are available:

```typescript
import { open, save } from '@tauri-apps/api/dialog';

// Open MML file
const filePath = await open({
  title: 'Open MML File',
  filters: [
    { name: 'MML Files', extensions: ['gwi', 'mml', 'muc', 'mdl', 'mus', 'txt'] }
  ]
});

// Save VGM file
const savePath = await save({
  title: 'Save VGM File',
  filters: [{ name: 'VGM Files', extensions: ['vgm'] }]
});
```

### File Reading/Writing

```typescript
import { readTextFile, writeTextFile, exists } from '@tauri-apps/api/fs';

// Check if file exists
const fileExists = await exists(filePath);

// Read file
const content = await readTextFile(filePath);

// Write file
await writeTextFile(savePath, vgmData);
```

### Notifications

```typescript
import { notify } from '@tauri-apps/api/notification';

await notify({
  title: 'Compilation Complete',
  body: 'Your file has been compiled successfully!'
});

await notify({
  title: 'Error',
  body: 'Compilation failed. Check the error list.'
});
```

### File Drop

File drop is enabled in `tauri.conf.json`:

```json
{
  "fileDropEnabled": true
}
```

Handle file drops in the frontend:

```typescript
import { listen } from '@tauri-apps/api/event';

listen('tauri://file-drop', (event) => {
  const { payload } = event as { payload: string[] };
  const filePath = payload[0];
  console.log('File dropped:', filePath);
  // Load the file...
});
```

---

## Icons

To generate icons from a source image:

```bash
npx @tauri-apps/cli icon path/to/source-icon.png
```

This generates:
- `icons/32x32.png`
- `icons/128x128.png`
- `icons/128x128@2x.png`

For Windows, you also need `icons/icon.ico`. You can create it with:

```bash
# Using ImageMagick
convert source-icon.png -define icon:auto-resize=16,32,48,64,128,256 icons/icon.ico
```

Or use an online ICO converter.

---

## Packaging

### Windows

Creates an MSI installer:
```
mml2vgm-desktop_0.1.0_x64_en-US.msi
```

### macOS

Creates a DMG with an app bundle:
```
mml2vgm.app
mml2vgm-desktop_0.1.0.aarch64.dmg
```

### Linux

Creates AppImage and .deb packages:
```
mml2vgm-desktop_0.1.0_amd64.AppImage
mml2vgm-desktop_0.1.0_amd64.deb
```

---

## Distribution

### GitHub Releases

1. Build all platforms:
   ```bash
   npm run tauri:build -- --all-features
   ```

2. Create a GitHub release with the artifacts

### Homebrew (macOS)

Create a tap:

```ruby
# mml2vgm.rb
class Mml2vgm < Formula
  version "0.1.0"
  homepage "https://github.com/example/mml2vgm"
  
  if OS.mac?
    url "https://github.com/example/mml2vgm/releases/download/v0.1.0/mml2vgm-desktop_0.1.0_amd64.dmg"
    sha256 "..."
    def install
      # Install logic
    end
  end
end
```

### Chocolatey (Windows)

Create a package:

```xml
<!-- mml2vgm.nuspec -->
<package>
  <metadata>
    <id>mml2vgm</id>
    <version>0.1.0</version>
    <title>mml2vgm Desktop</title>
  </metadata>
  <files>
    <file src="mml2vgm-desktop_0.1.0_x64_en-US.msi" />
  </files>
</package>
```

---

## Troubleshooting

### Error: "Rust not found"

Install Rust as described in the prerequisites section, then ensure it's in your PATH:

```bash
source "$HOME/.cargo/env"
```

### Error: "Tauri CLI not found"

Install the Tauri CLI globally:

```bash
npm install -g @tauri-apps/cli
```

### Build fails on macOS

Ensure you have Xcode command line tools:

```bash
xcode-select --install
```

### WASM not loading in production

Tauri 2.0 enables the required headers by default. If you still have issues:

1. Check the browser console for errors
2. Ensure `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` are set
3. In Tauri 2.0, these are enabled automatically for the webview

### "Module not found" errors

The Tauri app imports from `../browser-ide/src/`. Make sure:
1. You've run `npm install` in the browser-ide directory
2. The paths in `vite.config.ts` are correct
3. The path aliases match between tauri-app and browser-ide

---

## Updating Dependencies

To update Tauri and its dependencies:

```bash
# Update Tauri CLI
npm update -g @tauri-apps/cli

# Update Tauri in the project
cd tauri-app
npm update @tauri-apps/cli @tauri-apps/api

# Update Rust dependencies
cd src-tauri
cargo update
```

---

## License

GPL-3.0 - Same as the main mml2vgm project.

---

## Resources

- [Tauri Documentation](https://tauri.app/v2/)
- [Tauri GitHub](https://github.com/tauri-apps/tauri)
- [Tauri Discord](https://discord.gg/tauri)
- [Vite Documentation](https://vitejs.dev/)
