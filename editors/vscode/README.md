# mml2vgm VS Code Extension

Adds syntax highlighting for mml2vgm score files in Visual Studio Code and
compatible editors (Cursor, VSCodium, etc.).

## Supported file types

| Extension | Format |
|-----------|--------|
| `.gwi` | mml2vgm native format |
| `.muc` | Multi-channel MML |
| `.mdl` | MDL format |
| `.mus` | MUS format |

## Installation

### Manual (development copy)

Copy this directory into your VS Code extensions folder:

```sh
# macOS / Linux
cp -r editors/vscode/ ~/.vscode/extensions/mml2vgm-syntax/

# Windows (PowerShell)
Copy-Item -Recurse editors\vscode\ "$env:USERPROFILE\.vscode\extensions\mml2vgm-syntax\"
```

Reload VS Code (`Ctrl+Shift+P` → "Developer: Reload Window").

### Via `code --install-extension` (if packaged as `.vsix`)

```sh
code --install-extension mml2vgm-syntax-0.1.0.vsix
```

## Highlighted syntax

| Element | Colour class |
|---------|-------------|
| `;` line comments | Comment |
| Song info block `'{ … }` | Special / Keyword |
| Instrument headers `'@ M/F/X4 NNN` | Function / StorageClass |
| Instrument parameter rows `'@ 031,010,…` | Constant |
| Part declarations `'A1`, `'Vf01` | Statement |
| Tempo `T120`, volume `v100` | Keyword |
| Notes `c d e f g a b`, rests `r` | Type |
| Octave shifts `< >`, octave `o4` | Constant |
| Repeat blocks `[ … ]4` | Delimiter |
